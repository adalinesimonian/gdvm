// SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of gdvm.
//
// gdvm is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

use crate::artifact_cache::ArtifactCache;
use crate::config::Config;
use crate::date_utils::{modified_unix_secs, now_unix_secs};
use crate::host::{HostPlatform, detect_host};
use crate::metadata_cache::{CacheStore, GdvmCache};
#[cfg(test)]
use crate::metadata_cache::{RegistryReleasesCache, ReleaseCache, filter_cached_releases};
use crate::paths::GdvmPaths;
use crate::registry::{self, BinarySelectionError};
use crate::releases::{CatalogSet, ReleaseCatalog};
use crate::run_version_resolver::RunVersionSource;
use anyhow::{Result, anyhow, bail};
#[cfg(target_family = "unix")]
use daemonize::Daemonize;
use digest_io::IoWrapper;
use i18n::I18n;
use indicatif::{ProgressBar, ProgressStyle};
use semver::{Version, VersionReq};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashSet;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use crate::download_utils::download_file;
use crate::migrations;
use crate::registry_version_resolver::RegistryVersionResolver;
use crate::usage_tracker::{UsageState, UsageTracker};
use crate::version_utils::GodotVersion;
use crate::version_utils::{DeterminateSelection, Variant, VersionSelection};
use crate::zip_utils;
use crate::{eprintln_i18n, println_i18n};
use crate::{i18n, project_version_detector, t};

use crate::version_utils::GodotVersionDeterminate;

#[derive(Debug)]
pub enum InstallOutcome {
    Installed,
    AlreadyInstalled,
}

#[derive(Debug, Clone)]
pub struct InstalledVersion {
    pub version: GodotVersionDeterminate,
    pub variant: Variant,
    pub registry: Option<String>,
}

/// Options controlling how `GodotManager::prune` behaves.
#[derive(Debug, Clone, Copy, Default)]
pub struct PruneOptions {
    /// Remove all installs and cached archives regardless of age. Installs that
    /// still have an active link are preserved unless `force` is also set.
    pub all: bool,
    /// Ignore links entirely, allowing linked installs to be removed.
    pub force: bool,
    /// Report what would be removed without deleting anything.
    pub dry_run: bool,
}

/// A single asset removed by prune.
#[derive(Debug, Clone)]
pub struct PrunedItem {
    /// A user-friendly label for the asset.
    pub label: String,
    /// Approximate bytes freed by removing the asset.
    pub freed_bytes: u64,
}

/// The outcome of a prune operation.
#[derive(Debug, Clone, Default)]
pub struct PruneReport {
    /// Installs that were removed.
    pub installs: Vec<PrunedItem>,
    /// Cached archives that were removed.
    pub archives: Vec<PrunedItem>,
    /// Number of installs preserved because they still have an active link.
    pub preserved_by_link: usize,
    /// Total approximate bytes freed.
    pub freed_bytes: u64,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

impl PruneReport {
    /// True when nothing was removed.
    pub fn is_empty(&self) -> bool {
        self.installs.is_empty() && self.archives.is_empty()
    }
}

#[derive(Debug)]
enum GithubJsonError {
    Network(reqwest::Error),
    Api(anyhow::Error),
    Other(anyhow::Error),
}

impl std::fmt::Display for GithubJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "{e}"),
            Self::Api(e) | Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for GithubJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(e) => Some(e),
            Self::Api(e) | Self::Other(e) => Some(e.root_cause()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShaType {
    Sha256,
    Sha512,
}

impl ShaType {
    /// Attempts to detect the SHA type based on the expected hash length.
    /// SHA256 produces 64 hex characters, SHA512 produces 128 hex characters.
    fn from_hash_length(hash: &str) -> Option<Self> {
        match hash.len() {
            64 => Some(ShaType::Sha256),
            128 => Some(ShaType::Sha512),
            _ => None,
        }
    }
}

/// GodotManager is a struct that manages the installation and running of Godot versions.
pub struct GodotManager<'a> {
    /// Paths helper for GDVM directories
    paths: GdvmPaths,
    /// Cache for downloaded artifacts
    artifact_cache: ArtifactCache,
    /// Metadata cache.
    cache_store: CacheStore,
    /// Tracks when prunable assets were last used.
    usage_tracker: UsageTracker,
    /// Release catalogs for fetching Godot versions.
    catalogs: CatalogSet,
    /// Client for GitHub API requests
    client: reqwest::Client,
    /// Host platform
    host: HostPlatform,
    i18n: &'a I18n,
}

/// Verifies the SHA of a file against an expected hash.
fn verify_sha(file_path: &Path, expected: &str, i18n: &I18n) -> Result<()> {
    let sha_type = ShaType::from_hash_length(expected).ok_or_else(|| {
        anyhow!(t!(
            i18n,
            "error-invalid-sha-length",
            length = expected.len()
        ))
    })?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(t!(i18n, "verifying-checksum"));

    let mut f = fs::File::open(file_path)?;

    let local_hash = match sha_type {
        ShaType::Sha256 => {
            let mut hasher = IoWrapper(Sha256::new());
            std::io::copy(&mut f, &mut hasher)?;
            hasher
                .0
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        }
        ShaType::Sha512 => {
            let mut hasher = IoWrapper(Sha512::new());
            std::io::copy(&mut f, &mut hasher)?;
            hasher
                .0
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        }
    };

    if local_hash == expected.to_lowercase() {
        pb.finish_with_message(t!(i18n, "checksum-verified"));
        Ok(())
    } else {
        pb.finish_and_clear();
        Err(anyhow!(t!(
            i18n,
            "error-checksum-mismatch",
            file = file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        )))
    }
}

/// Searches for the Godot executable within the given directory.
///
/// ## Arguments
///
/// - `version_dir` - A reference to the directory path where the search is performed.
/// - `console` - A boolean indicating whether to search for the console version (relevant for Windows).
///
/// ## Returns
///
/// - `Ok(Some(PathBuf))` containing the path to the Godot executable if found.
/// - `Ok(None)` if no executable is found.
/// - `Err(io::Error)` if there is an error reading the directory.
#[allow(unused_variables)]
pub fn find_godot_executable(version_dir: &Path, console: bool) -> Result<Option<PathBuf>> {
    // Collect all entries (files/folders) under version_dir
    let entries: Vec<_> = fs::read_dir(version_dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    #[cfg(target_os = "windows")]
    {
        // If console is requested, try to find a "_console" executable first
        if console {
            let console_candidate = entries.iter().find_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("_console.exe") {
                    Some(entry.path())
                } else {
                    None
                }
            });

            // If found, return it.
            if console_candidate.is_some() {
                return Ok(console_candidate);
            }
        }

        // Prefer the non-console executable when available.
        let gui_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".exe") && !name.ends_with("_console.exe") {
                Some(entry.path())
            } else {
                None
            }
        });

        if gui_candidate.is_some() {
            return Ok(gui_candidate);
        }

        // Fall back to any .exe if nothing else matches.
        let exe_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".exe") {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(exe_candidate)
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, prefer an app bundle but return the executable inside it.
        let app_candidate = entries.iter().find_map(|entry| {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.ends_with(".app") {
                Some(entry.path())
            } else {
                None
            }
        });

        if let Some(app_path) = app_candidate
            && let Some(exe) = find_macos_app_executable(&app_path)
        {
            return Ok(Some(exe));
        }

        // Fall back to a Godot binary directly under the install dir.
        let binary_candidate = entries.iter().find_map(|entry| {
            let Ok(file_type) = entry.file_type() else {
                return None;
            };
            if !file_type.is_file() {
                return None;
            }

            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.starts_with("Godot") {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(binary_candidate)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // For Linux or other Unix-likes
        // Look for a few known suffixes or naming patterns
        let unix_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("Godot_v")
                || name.ends_with(".x86_64")
                || name.ends_with(".x86_32")
                || name.ends_with(".arm64")
            {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(unix_candidate)
    }
}

#[cfg(target_os = "macos")]
fn find_macos_app_executable(app_path: &Path) -> Option<PathBuf> {
    let macos_dir = app_path.join("Contents/MacOS");

    // Prefer known Godot binaries.
    let preferred = ["Godot", "Godot_mono"];
    for name in preferred {
        let candidate = macos_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    // Fall back to first regular file in Contents/MacOS.
    let entries = fs::read_dir(&macos_dir).ok()?;
    for entry in entries.flatten() {
        if entry.file_type().ok()?.is_file() {
            return Some(entry.path());
        }
    }

    None
}

/// Compute the approximate size of a file or directory in bytes.
fn dir_size(path: &Path) -> u64 {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.file_type().is_symlink() {
        return 0;
    }
    if meta.is_file() {
        return meta.len();
    }

    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            total += dir_size(&entry.path());
        }
    }
    total
}

/// Get registry information from the nearest gdvm.toml in the current directory or its parents.
fn project_registry_pairs(i18n: &I18n) -> Vec<(String, String)> {
    let Ok(mut current) = std::env::current_dir() else {
        return Vec::new();
    };

    loop {
        let candidate = current.join("gdvm.toml");
        if candidate.is_file() {
            let parsed = fs::read_to_string(&candidate)
                .map_err(|e| e.to_string())
                .and_then(|contents| {
                    crate::version_utils::deserialize_gdvm_toml(&contents)
                        .map_err(|e| e.to_string())
                });
            match parsed {
                Ok(toml) => {
                    if let Some(registries) = toml.registries {
                        return registries
                            .into_iter()
                            .filter(|(name, _)| crate::config::validate_registry_name(name).is_ok())
                            .map(|(name, registry)| (name, registry.url))
                            .collect();
                    }
                }
                Err(error) => {
                    let path = candidate.display().to_string();
                    eprintln_i18n!(
                        i18n,
                        "gdvm-toml-malformed",
                        path = path.as_str(),
                        error = error.as_str()
                    );
                }
            }
        }

        if !current.pop() {
            break;
        }
    }

    Vec::new()
}

/// A project-level registry alias that shadows a machine-level one of the same
/// name but points at a different URL.
struct RegistryOverrideConflict {
    name: String,
    machine_url: String,
    project_url: String,
}

/// Find aliases that appear in both the machine-level and project-level
/// registry sets but resolve to different URLs.
fn registry_override_conflicts(
    machine: &[(String, String)],
    project: &[(String, String)],
) -> Vec<RegistryOverrideConflict> {
    let machine_urls: std::collections::HashMap<&str, &str> = machine
        .iter()
        .map(|(name, url)| (name.as_str(), url.as_str()))
        .collect();

    project
        .iter()
        .filter_map(|(name, project_url)| {
            let machine_url = machine_urls.get(name.as_str())?;
            (*machine_url != project_url.as_str()).then(|| RegistryOverrideConflict {
                name: name.clone(),
                machine_url: (*machine_url).to_string(),
                project_url: project_url.clone(),
            })
        })
        .collect()
}

impl<'a> GodotManager<'a> {
    /// Create a new GodotManager instance and set up the installation and cache paths
    pub async fn new(i18n: &'a I18n) -> Result<Self> {
        let paths = GdvmPaths::new(i18n)?;
        let artifact_cache = ArtifactCache::new(paths.cache_dir().to_path_buf());
        artifact_cache.ensure_dir()?;

        let client = GodotManager::get_github_client(i18n)?;

        let config = Config::load(i18n).unwrap_or_default();
        let mut registries = config.registry_pairs();
        let project = project_registry_pairs(i18n);
        for conflict in registry_override_conflicts(&registries, &project) {
            eprintln_i18n!(
                i18n,
                "registry-project-override-conflict",
                registry = conflict.name.as_str(),
                machine_url = conflict.machine_url.as_str(),
                project_url = conflict.project_url.as_str(),
            );
        }
        registries.extend(project);
        let catalogs = CatalogSet::new(paths.cache_index(), &registries)?;
        let cache_store = CacheStore::new(paths.cache_index().to_path_buf());
        let usage_tracker = UsageTracker::new(paths.usage_index().to_path_buf());
        let host = detect_host(i18n)?;

        let manager = GodotManager {
            paths,
            artifact_cache,
            cache_store,
            usage_tracker,
            catalogs,
            client,
            host,
            i18n,
        };

        migrations::run_migrations(manager.paths.base(), i18n)?;

        // Don't fail if update check fails, since it isn't critical
        manager.check_for_upgrades().await.ok();

        Ok(manager)
    }

    /// Gets the path to the GodotManager's base directory
    /// (e.g. `~/.gdvm` on Unix-like systems)
    pub fn get_base_path(&self) -> &Path {
        self.paths.base()
    }

    /// Select a release catalog by registry name. If `registry` is `None`, the
    /// official catalog is returned.
    fn catalog(&self, registry: Option<&str>) -> Result<&ReleaseCatalog> {
        self.catalogs.catalog(registry)
    }

    /// The install store directory name for a registry.
    fn install_store_key(&self, registry: Option<&str>) -> Result<String> {
        let base_url = self.catalog(registry)?.registry_base_url();
        Ok(crate::registry::store_dir_name(&base_url))
    }

    /// A unique key for an install path.
    pub fn install_key(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<String> {
        Ok(crate::version_utils::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        ))
    }

    /// Record that an install was run or referenced.
    fn track_install_use(&self, install_key: &str) -> Result<()> {
        self.usage_tracker.record_install(install_key)
    }

    /// Record that a cached archive was used.
    fn track_archive_use(&self, archive_path: &Path) -> Result<()> {
        if let Some(name) = archive_path.file_name().and_then(|n| n.to_str()) {
            self.usage_tracker.record_archive(name)?;
        }
        Ok(())
    }

    /// Record that a link was created at `link_path` pointing into the install
    /// identified by `install_key`.
    pub fn record_link(&self, link_path: &Path, install_key: &str) -> Result<()> {
        self.usage_tracker.record_link(link_path, install_key)
    }

    /// The label to display for a registry.
    fn display_registry_for_url(&self, url: &str, display_name: Option<&str>) -> Option<String> {
        let normalized = crate::registry::normalize_url(url);
        if normalized == crate::registry::normalize_url(crate::registry::OFFICIAL_BASE_URL) {
            return None;
        }
        for name in self.catalogs.names() {
            if name == crate::registry::OFFICIAL_REGISTRY {
                continue;
            }
            if let Ok(cat) = self.catalogs.catalog(Some(name))
                && crate::registry::normalize_url(&cat.registry_base_url()) == normalized
            {
                return Some(name.to_string());
            }
        }
        Some(display_name.unwrap_or(url).to_string())
    }

    /// Select the correct binary for the current host and `variant`.
    fn select_platform_binary<'r>(
        &self,
        meta: &'r registry::ReleaseMetadata,
        variant: &Variant,
    ) -> Result<&'r registry::BinaryInfo> {
        registry::select_binary(meta, self.host, variant).map_err(|err| match err {
            BinarySelectionError::UnsupportedPlatform => {
                anyhow!(t!(self.i18n, "unsupported-platform"))
            }
            BinarySelectionError::UnsupportedArch => {
                anyhow!(t!(self.i18n, "unsupported-architecture"))
            }
            BinarySelectionError::MissingUrl => {
                anyhow!(t!(self.i18n, "error-file-not-found"))
            }
        })
    }

    /// Resolve the path to the cached download archive for a release.
    pub async fn cached_archive_path(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<PathBuf> {
        let meta = self.catalog(registry)?.metadata_for(gv, self.i18n).await?;
        let binary = self.select_platform_binary(&meta, variant)?;
        let path = self.artifact_cache.cached_zip_path(&binary.sha512);
        if !path.exists() {
            bail!(t!(
                self.i18n,
                "error-archive-not-cached",
                version = gv.to_display_str()
            ));
        }
        self.track_archive_use(&path)?;
        Ok(path)
    }

    /// Install a specified Godot version
    ///
    /// - `variant`: Optional variant, e.g. `Some("csharp")`.
    /// - `registry`: Optional registry name, `None` uses gdvm's official registry.
    /// - `force`: If true, reinstall the version even if it's already installed.
    /// - `redownload`: If true, ignore cached zip files and download fresh ones.
    pub async fn install(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
        force: bool,
        redownload: bool,
    ) -> Result<InstallOutcome> {
        let store_key = self.install_store_key(registry)?;
        let install_str =
            crate::version_utils::install_dir_subpath(&store_key, &gv.to_remote_str(), variant);
        let version_path = self.paths.installs().join(&install_str);

        if version_path.exists() {
            if force {
                self.remove(gv, variant, registry)?;
                eprintln_i18n!(
                    self.i18n,
                    "force-reinstalling-version",
                    version = gv.to_display_str(),
                );
            } else {
                return Ok(InstallOutcome::AlreadyInstalled);
            }
        }

        if !gv.is_stable() {
            eprintln_i18n!(self.i18n, "warning-prerelease", branch = &gv.release_type);
        }

        self.artifact_cache.ensure_dir()?;

        let meta = self.catalog(registry)?.metadata_for(gv, self.i18n).await?;

        let binary = self.select_platform_binary(&meta, variant)?;

        let download_url = binary.urls.first().unwrap();
        let cache_zip_path = self.artifact_cache.cached_zip_path(&binary.sha512);

        let cache_hit = !redownload
            && cache_zip_path.exists()
            && verify_sha(&cache_zip_path, &binary.sha512, self.i18n).is_ok();

        if cache_hit {
            eprintln_i18n!(self.i18n, "using-cached-zip");
        } else {
            if redownload && cache_zip_path.exists() {
                eprintln_i18n!(self.i18n, "force-redownload", version = gv.to_display_str());
            }

            let tmp_file = self.artifact_cache.partial_path(&binary.sha512);

            // Download the archive.
            let staged = async {
                download_file(download_url, &tmp_file, self.i18n).await?;
                verify_sha(&tmp_file, &binary.sha512, self.i18n)?;
                fs::rename(&tmp_file, &cache_zip_path)?;
                anyhow::Ok(())
            }
            .await;

            if let Err(err) = staged {
                let _ = fs::remove_file(&tmp_file);
                return Err(err);
            }

            eprintln_i18n!(self.i18n, "cached-zip-stored");
        }

        fs::create_dir_all(&version_path)?;

        self.track_archive_use(&cache_zip_path)?;

        // Extract from cache_zip_path
        zip_utils::extract_zip(&cache_zip_path, &version_path, self.i18n)?;

        let base_url = self.catalog(registry)?.registry_base_url();
        let store_dir = self.paths.installs().join(&store_key);
        crate::registry_store::upsert(&store_dir, &base_url, registry, None)?;

        self.track_install_use(&install_str)?;

        Ok(InstallOutcome::Installed)
    }

    /// List all installed Godot versions.
    pub fn list_installed(&self) -> Result<Vec<InstalledVersion>> {
        let mut versions = vec![];
        let installs_dir = self.paths.installs();

        for entry in fs::read_dir(installs_dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() || !file_type.is_dir() {
                continue;
            }
            let top_dir = entry.path();

            match crate::registry_store::read(&top_dir)? {
                Some(meta) => {
                    let registry =
                        self.display_registry_for_url(&meta.url, meta.display_name.as_deref());
                    self.collect_store_installs(&top_dir, &registry, &mut versions);
                }
                None => {
                    // Legacy layout.
                    self.collect_legacy_installs(&entry, &mut versions);
                }
            }
        }
        versions.sort_by(|a, b| {
            use crate::version_utils::GodotVersionDeterminateVecExt;
            let mut v = vec![a.version.clone(), b.version.clone()];
            v.sort_by_version();
            if v[0] == a.version {
                std::cmp::Ordering::Greater // Newest first.
            } else {
                std::cmp::Ordering::Less
            }
        });
        Ok(versions)
    }

    /// Collect installs from a URL-keyed store directory, whose layout is
    /// `{store}/{variant}/{version}`.
    fn collect_store_installs(
        &self,
        store_dir: &Path,
        registry: &Option<String>,
        out: &mut Vec<InstalledVersion>,
    ) {
        let Ok(variants) = fs::read_dir(store_dir) else {
            return;
        };
        for variant_entry in variants.flatten() {
            if !variant_entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                continue;
            }
            let variant_name = variant_entry.file_name().to_string_lossy().to_string();
            let Ok(leaves) = fs::read_dir(variant_entry.path()) else {
                continue;
            };
            for leaf in leaves.flatten() {
                if leaf.file_type().is_ok_and(|ft| ft.is_dir())
                    && let Ok(gv) =
                        GodotVersion::from_install_str(&leaf.file_name().to_string_lossy())
                {
                    out.push(InstalledVersion {
                        version: gv.to_determinate(),
                        variant: Variant::from_option(Some(variant_name.as_str())),
                        registry: registry.clone(),
                    });
                }
            }
        }
    }

    /// Collect installs from a legacy layout.
    fn collect_legacy_installs(&self, entry: &fs::DirEntry, out: &mut Vec<InstalledVersion>) {
        let top_name = entry.file_name().to_string_lossy().to_string();
        let top_dir = entry.path();
        let Ok(sub_entries) = fs::read_dir(&top_dir) else {
            return;
        };
        for sub_entry in sub_entries.flatten() {
            if !sub_entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                continue;
            }
            let sub_name = sub_entry.file_name().to_string_lossy().to_string();

            if let Ok(gv) = GodotVersion::from_install_str(&sub_name) {
                out.push(InstalledVersion {
                    version: gv.to_determinate(),
                    variant: Variant::from_option(Some(top_name.as_str())),
                    registry: None,
                });
            } else {
                let variant_dir = top_dir.join(&sub_name);
                let Ok(leaves) = fs::read_dir(&variant_dir) else {
                    continue;
                };
                for leaf in leaves.flatten() {
                    if leaf.file_type().is_ok_and(|ft| ft.is_dir())
                        && let Ok(gv) =
                            GodotVersion::from_install_str(&leaf.file_name().to_string_lossy())
                    {
                        out.push(InstalledVersion {
                            version: gv.to_determinate(),
                            variant: Variant::from_option(Some(sub_name.as_str())),
                            registry: Some(top_name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Remove a specified Godot version
    pub fn remove(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<()> {
        let install_name = crate::version_utils::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        );
        let path = self.paths.installs().join(&install_name);

        if path.exists() {
            // If this version is the default, unset it
            if let Some(def) = self.get_default()?
                && def.version.to_remote_str() == gv.to_remote_str()
                && def.variant == *variant
                && crate::version_utils::normalize_registry(def.registry.as_deref())
                    == crate::version_utils::normalize_registry(registry)
            {
                self.unset_default()?;
            }
            fs::remove_dir_all(path)?;
            self.usage_tracker.forget_install(&install_name)?;
            Ok(())
        } else {
            Err(anyhow!(t!(self.i18n, "error-version-not-found")))
        }
    }

    /// Run a specified Godot version
    pub fn run(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
        console: bool,
        godot_args: &[String],
    ) -> Result<()> {
        let path = self.get_executable_path(gv, variant, registry, console)?;

        if console {
            // Run the process attached to the terminal and wait for it to exit
            std::process::Command::new(&path)
                .args(godot_args)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;
        } else {
            // Detached process configuration
            #[cfg(target_family = "unix")]
            {
                Daemonize::new().start().map_err(|e| {
                    anyhow!(t!(self.i18n, "error-starting-godot", error = e.to_string(),))
                })?;
                std::process::Command::new(&path).args(godot_args).spawn()?;
            }

            #[cfg(target_family = "windows")]
            {
                use std::os::windows::process::CommandExt;
                use winapi::um::winbase::DETACHED_PROCESS;
                std::process::Command::new(&path)
                    .args(godot_args)
                    .creation_flags(DETACHED_PROCESS)
                    .spawn()?;
            }
        }

        Ok(())
    }

    /// Resolve the path to the Godot executable for the given version and console preference.
    pub fn get_executable_path(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
        console: bool,
    ) -> Result<std::path::PathBuf> {
        let install_name = crate::version_utils::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        );
        let version_dir = self.paths.installs().join(&install_name);
        if !version_dir.exists() {
            return Err(anyhow!(t!(
                self.i18n,
                "error-version-not-found",
                version = &gv.to_display_str(),
            )));
        }

        let godot_executable = find_godot_executable(&version_dir, console)?;

        let godot_executable = godot_executable.ok_or_else(|| {
            anyhow!(t!(
                self.i18n,
                "godot-executable-not-found",
                version = &gv.to_display_str(),
            ))
        })?;

        self.track_install_use(&install_name)?;

        Ok(godot_executable)
    }

    /// Fetch available releases with caching.
    pub async fn fetch_available_releases(
        &self,
        registry: Option<&str>,
        filter: &Option<GodotVersion>,
        use_cache_only: bool,
    ) -> Result<Vec<GodotVersionDeterminate>> {
        self.catalog(registry)?
            .list_releases(filter.as_ref(), use_cache_only, self.i18n)
            .await
    }

    /// Gets a reqwest client with the GitHub token if available
    fn get_github_client(i18n: &I18n) -> Result<reqwest::Client> {
        let token = env::var("GITHUB_TOKEN").ok().or_else(|| {
            let config = Config::load(i18n).ok()?;
            config.github_token
        });
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(token) = token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("token {token}").parse()?,
            );
        }
        Ok(reqwest::ClientBuilder::new()
            .default_headers(headers)
            .user_agent("gdvm")
            .build()?)
    }

    /// Tries to query the GitHub API with a GET at a given URL. If it fails due
    /// to a rate-limit, it will return an error after printing a message.
    async fn get_github_json(&self, url: &str) -> Result<serde_json::Value, GithubJsonError> {
        // Rate limits are 403s with a JSON object that has a "message" key that
        // starts with "API rate limit exceeded".

        let resp = self
            .client
            .get(url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map_err(GithubJsonError::Network)?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| GithubJsonError::Other(e.into()))?;
            return Ok(json);
        }

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| GithubJsonError::Other(e.into()))?;
        if status == reqwest::StatusCode::FORBIDDEN
            && let Some(message) = json.get("message").and_then(|m| m.as_str())
            && message.starts_with("API rate limit exceeded")
        {
            return Err(GithubJsonError::Api(anyhow!(t!(
                self.i18n,
                "error-github-rate-limit"
            ))));
        }

        let error_message = json.get("message").and_then(|m| m.as_str());

        Err(GithubJsonError::Api(anyhow!(t!(
            self.i18n,
            "error-github-api",
            error = error_message,
        ))))
    }

    /// Clears the release cache by deleting the cache file and all cached zip files
    pub fn clear_cache(&self) -> Result<()> {
        let cache_index = self.cache_store.index_path();
        if cache_index.exists() {
            fs::remove_file(cache_index)?;
            println_i18n!(self.i18n, "cache-metadata-removed");
        } else {
            println_i18n!(self.i18n, "no-cache-metadata-found");
        }

        if self.artifact_cache.exists() {
            self.artifact_cache.clear_files()?;
            println_i18n!(self.i18n, "cache-files-removed");
        } else {
            println_i18n!(self.i18n, "no-cache-files-found");
        }
        Ok(())
    }

    /// Remove installs and cached archives that are no longer needed.
    pub fn prune(&self, max_age_secs: u64, opts: PruneOptions) -> Result<PruneReport> {
        let now = now_unix_secs();
        let mut state = self.usage_tracker.load()?;

        let default_install_key: Option<String> =
            self.get_default().ok().flatten().and_then(|def| {
                self.install_key(&def.version, &def.variant, def.registry.as_deref())
                    .ok()
            });

        let protected: HashSet<String> = if opts.force {
            HashSet::new()
        } else {
            self.live_link_install_keys(&state)
        };

        let mut report = PruneReport {
            dry_run: opts.dry_run,
            ..Default::default()
        };

        for (key, path) in self.collect_prunable_installs() {
            if default_install_key.as_deref() == Some(key.as_str()) {
                continue;
            }

            if !opts.force && protected.contains(&key) {
                report.preserved_by_link += 1;
                continue;
            }

            let should_remove = if opts.all {
                true
            } else {
                let last_used = self.effective_install_last_used(&key, &path, &state);
                now.saturating_sub(last_used) >= max_age_secs
            };

            if !should_remove {
                continue;
            }

            let freed = dir_size(&path);
            if !opts.dry_run {
                fs::remove_dir_all(&path)?;
            }
            report.freed_bytes += freed;
            report.installs.push(PrunedItem {
                label: self.install_label(&key),
                freed_bytes: freed,
            });
        }

        for path in self.collect_cached_archives() {
            let should_remove = if opts.all {
                true
            } else {
                let last_used = self.effective_archive_last_used(&path, &state);
                now.saturating_sub(last_used) >= max_age_secs
            };

            if !should_remove {
                continue;
            }

            let freed = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if !opts.dry_run {
                fs::remove_file(&path)?;
            }
            report.freed_bytes += freed;
            report.archives.push(PrunedItem {
                label: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                freed_bytes: freed,
            });
        }

        if opts.dry_run {
            return Ok(report);
        }

        let installs_dir = self.paths.installs().to_path_buf();
        let cache_dir = self.artifact_cache.dir().to_path_buf();
        state.installs.retain(|k, _| installs_dir.join(k).exists());
        state
            .archives
            .retain(|name, _| cache_dir.join(name).exists());
        state
            .links
            .retain(|path_str, rec| self.link_is_live(Path::new(path_str), &rec.install_key));
        self.usage_tracker.save(&state)?;

        Ok(report)
    }

    /// The set of install keys that still have at least one symlink.
    fn live_link_install_keys(&self, state: &UsageState) -> HashSet<String> {
        let mut protected = HashSet::new();
        for (path_str, rec) in &state.links {
            if self.link_is_live(Path::new(path_str), &rec.install_key) {
                protected.insert(rec.install_key.clone());
            }
        }
        protected
    }

    /// True when `link_path` is a symlink that resolves into the install
    /// directory identified by `install_key`.
    fn link_is_live(&self, link_path: &Path, install_key: &str) -> bool {
        let Ok(meta) = fs::symlink_metadata(link_path) else {
            return false;
        };
        if !meta.file_type().is_symlink() {
            return false;
        }
        let install_dir = self.paths.installs().join(install_key);
        let (Ok(target), Ok(install_canon)) =
            (fs::canonicalize(link_path), fs::canonicalize(&install_dir))
        else {
            return false;
        };
        target.starts_with(install_canon)
    }

    /// Enumerate every install directory as `(install_key, path)`.
    fn collect_prunable_installs(&self) -> Vec<(String, PathBuf)> {
        let installs = self.paths.installs();
        let mut out = Vec::new();
        let Ok(tops) = fs::read_dir(installs) else {
            return out;
        };
        for top in tops.flatten() {
            if !top.file_type().is_ok_and(|ft| ft.is_dir()) {
                continue;
            }
            let top_name = top.file_name().to_string_lossy().to_string();
            let Ok(mids) = fs::read_dir(top.path()) else {
                continue;
            };
            for mid in mids.flatten() {
                if !mid.file_type().is_ok_and(|ft| ft.is_dir()) {
                    continue;
                }
                let mid_name = mid.file_name().to_string_lossy().to_string();
                if GodotVersion::from_install_str(&mid_name).is_ok() {
                    // Legacy variant/version layout.
                    out.push((format!("{top_name}/{mid_name}"), mid.path()));
                    continue;
                }
                let Ok(leaves) = fs::read_dir(mid.path()) else {
                    continue;
                };
                for leaf in leaves.flatten() {
                    if !leaf.file_type().is_ok_and(|ft| ft.is_dir()) {
                        continue;
                    }
                    let leaf_name = leaf.file_name().to_string_lossy().to_string();
                    if GodotVersion::from_install_str(&leaf_name).is_ok() {
                        out.push((format!("{top_name}/{mid_name}/{leaf_name}"), leaf.path()));
                    }
                }
            }
        }
        out
    }

    /// Enumerate every file in the artifact cache directory.
    fn collect_cached_archives(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(self.artifact_cache.dir()) else {
            return out;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|ft| ft.is_file()) {
                out.push(entry.path());
            }
        }
        out
    }

    /// The most recent recorded use of an install, falling back to the
    /// directory's modification time when no usage is tracked.
    fn effective_install_last_used(&self, key: &str, path: &Path, state: &UsageState) -> u64 {
        if let Some(usage) = state.installs.get(key) {
            usage.last_used
        } else {
            modified_unix_secs(path).unwrap_or(0)
        }
    }

    /// The most recent recorded use of a cached archive, falling back to the
    /// file's modification time when no usage is tracked.
    fn effective_archive_last_used(&self, path: &Path, state: &UsageState) -> u64 {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(usage) = state.archives.get(&name) {
            usage.last_used
        } else {
            modified_unix_secs(path).unwrap_or(0)
        }
    }

    /// Get a user-friendly label for an install key.
    fn install_label(&self, key: &str) -> String {
        let parts: Vec<&str> = key.split('/').collect();
        let (registry, variant, version) = match parts.as_slice() {
            [store, variant, version] => {
                let registry = crate::registry_store::read(&self.paths.installs().join(store))
                    .ok()
                    .flatten()
                    .and_then(|m| self.display_registry_for_url(&m.url, m.display_name.as_deref()));
                (registry, (*variant).to_string(), (*version).to_string())
            }
            [variant, version] => (None, (*variant).to_string(), (*version).to_string()),
            _ => return key.to_string(),
        };

        match GodotVersion::from_install_str(&version) {
            Ok(gv) => crate::version_utils::display_version(
                &gv.to_determinate().to_display_str(),
                &Variant::from_option(Some(&variant)),
                registry.as_deref(),
            ),
            Err(_) => key.to_string(),
        }
    }

    /// Refresh the gdvm release cache by re-downloading the registry index.
    pub async fn refresh_cache(&self) -> Result<()> {
        self.catalog(None)?.refresh_cache(self.i18n).await
    }

    /// Refresh the cache for a single registry.
    pub async fn refresh_registry_cache(&self, registry: Option<&str>) -> Result<()> {
        self.catalog(registry)?.refresh_cache(self.i18n).await
    }

    /// Refresh the caches of every configured registry. Stops at the first failure.
    pub async fn refresh_all_registry_caches(&self) -> Result<()> {
        for name in self.catalogs.names() {
            self.catalog(Some(name))?.refresh_cache(self.i18n).await?;
        }
        Ok(())
    }

    /// True when `registry` is the official registry. Also true when `registry` is `None`, since
    /// the official registry is the default.
    pub fn is_official_registry(&self, registry: Option<&str>) -> bool {
        self.catalogs.is_official(registry)
    }

    /// The base URL configured for a registry.
    pub fn registry_base_url(&self, registry: &str) -> Result<String> {
        Ok(self.catalog(Some(registry))?.registry_base_url())
    }

    /// Summaries of all configured registries, official first, marking the default.
    pub fn registry_list(&self) -> Vec<crate::releases::RegistryInfo> {
        self.catalogs.list()
    }

    /// Resolve the Godot version from a string, for an installed version
    /// Returns a list of possible versions. If the input is ambiguous, the list
    /// will have more than one element. Otherwise, it will have one element,
    /// unless of course the version is not found, in which case the list will
    /// be empty.
    /// Accepts full and partial versions.
    pub async fn resolve_installed_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> Result<Vec<InstalledVersion>>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let installed = self.list_installed()?;
        // Filter by registry and variant, then filter by version match.
        let target_variant = Variant::from_option(variant);
        let matches: Vec<_> = installed
            .into_iter()
            .filter(|v| {
                v.variant == target_variant
                    && crate::version_utils::normalize_registry(v.registry.as_deref())
                        == crate::version_utils::normalize_registry(registry)
                    && gv.matches(&v.version)
            })
            .collect();
        Ok(matches)
    }

    /// Resolve the Godot version from a string, for an available version
    /// Returns a single version, whichever is the latest that matches the
    /// input.
    /// Accepts full and partial versions.
    pub async fn resolve_available_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
        use_cache_only: bool,
    ) -> Result<Option<GodotVersionDeterminate>>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let resolver = RegistryVersionResolver::new(self.catalog(registry)?, self.i18n, self.host);
        resolver
            .resolve_available(&gv, variant, include_pre, use_cache_only)
            .await
    }

    pub fn set_default(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<()> {
        // Check if the version exists
        let install_name = crate::version_utils::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        );
        let version_path = self.paths.installs().join(&install_name);
        if !version_path.exists() {
            return Err(anyhow!(t!(self.i18n, "error-version-not-found")));
        }

        self.track_install_use(&install_name)?;

        // Write pinned-format string to .gdvm/default
        let default_path = self.paths.default_file();
        let default_str = crate::version_utils::pinned_str(registry, &gv.to_remote_str(), variant);
        fs::write(&default_path, &default_str)?;

        // Create directory symlink .gdvm/bin/current_godot -> .gdvm/<install_name>/
        let symlink_dir = self.paths.current_godot_symlink();
        let target_dir = self.paths.installs().join(&install_name);

        // Make sure bin directory exists
        fs::create_dir_all(symlink_dir.parent().unwrap())?;

        if symlink_dir.exists() {
            fs::remove_dir_all(&symlink_dir)?;
        }
        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(&target_dir, &symlink_dir)?;
        #[cfg(target_family = "windows")]
        if let Err(e) = std::os::windows::fs::symlink_dir(&target_dir, &symlink_dir) {
            if e.raw_os_error() == Some(1314) {
                return Err(anyhow!(t!(self.i18n, "error-create-symlink-windows")));
            }
            return Err(anyhow!(e));
        }

        Ok(())
    }

    pub fn unset_default(&self) -> Result<()> {
        // Remove default file and symlink
        let default_file = self.paths.default_file();
        if default_file.exists() {
            fs::remove_file(default_file)?;
        }

        let symlink_dir = self.paths.current_godot_symlink();
        if symlink_dir.exists() {
            fs::remove_dir_all(symlink_dir)?;
        }

        Ok(())
    }

    pub fn get_default(&self) -> Result<Option<DeterminateSelection>> {
        let default_file = self.paths.default_file();
        if default_file.exists() {
            let contents = fs::read_to_string(&default_file)?;
            let (registry, variant, version_str) =
                crate::version_utils::parse_pinned_str(contents.trim());
            let version = GodotVersion::from_install_str(&version_str)?.to_determinate();
            Ok(Some(DeterminateSelection {
                version,
                variant: Variant::from_option(variant.as_deref()),
                registry,
            }))
        } else {
            Ok(None)
        }
    }

    /// Recursively search upward for gdvm.toml, return the pinned version if found.
    pub fn get_pinned_version(&self) -> Option<VersionSelection> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let toml_candidate = current.join("gdvm.toml");
            if toml_candidate.is_file()
                && let Ok(contents) = fs::read_to_string(&toml_candidate)
                && let Ok(gdvm_toml) = crate::version_utils::deserialize_gdvm_toml(&contents)
                && let Some(godot) = gdvm_toml.godot
            {
                let (registry, variant, version_str) =
                    crate::version_utils::parse_pinned_str(godot.version.trim());
                if let Ok(version) = GodotVersion::from_install_str(&version_str) {
                    return Some(VersionSelection {
                        version,
                        variant,
                        registry,
                    });
                }
            }

            // Fall back to deprecated .gdvmrc.
            let rc_candidate = current.join(".gdvmrc");
            if rc_candidate.is_file()
                && let Ok(contents) = fs::read_to_string(&rc_candidate)
            {
                let (registry, variant, version_str) =
                    crate::version_utils::parse_pinned_str(contents.trim());
                if let Ok(version) = GodotVersion::from_install_str(&version_str) {
                    return Some(VersionSelection {
                        version,
                        variant,
                        registry,
                    });
                }
            }

            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Try to determine the version to use based on the current Godot project
    pub fn determine_version<P: AsRef<Path>>(
        &self,
        path: Option<P>,
    ) -> Option<(GodotVersion, Option<String>)> {
        let current_dir = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => std::env::current_dir().ok()?,
        };

        project_version_detector::detect_godot_version_in_path(self.i18n, &current_dir)
    }

    /// Pin a version to gdvm.toml in the current directory.
    pub fn pin_version(
        &self,
        gv: &GodotVersionDeterminate,
        variant: &Variant,
        registry: Option<&str>,
        gdvm_toml_only: bool,
    ) -> Result<()> {
        let path = std::env::current_dir()?;

        let specifier = crate::version_utils::pinned_str(registry, &gv.to_remote_str(), variant);
        let toml_content = crate::version_utils::serialize_gdvm_toml(&specifier);
        fs::write(path.join("gdvm.toml"), toml_content)?;

        // Write deprecated .gdvmrc for backward compatibility with older versions of gdvm.
        // The legacy format predates registries, so we skip writing it for builds from custom
        // registries.
        if !gdvm_toml_only && crate::version_utils::is_official_registry(registry) {
            let legacy = crate::version_utils::legacy_pinned_str(gv, variant);
            fs::write(path.join(".gdvmrc"), legacy)?;
        }

        Ok(())
    }

    pub async fn auto_install_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
    ) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let resolver = RegistryVersionResolver::new(self.catalog(registry)?, self.i18n, self.host);

        let actual_version = resolver
            .resolve_for_auto_install(&gv, variant, include_pre)
            .await?;

        // Check if version is installed, if not, install
        if !self.is_version_installed(&actual_version, variant, registry)? {
            eprintln_i18n!(
                self.i18n,
                "auto-installing-version",
                version = &actual_version.to_display_str(),
            );
            self.install(
                &actual_version,
                &Variant::from_option(variant),
                registry,
                false,
                false,
            )
            .await?;
        }
        Ok(actual_version)
    }

    fn is_version_installed<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> Result<bool>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();

        let installed_versions = self.list_installed()?;
        let target_variant = Variant::from_option(variant);
        Ok(installed_versions.iter().any(|v| {
            v.variant == target_variant
                && crate::version_utils::normalize_registry(v.registry.as_deref())
                    == crate::version_utils::normalize_registry(registry)
                && gv.matches(&v.version)
        }))
    }

    /// Find the latest stable release matching a semver requirement from a list of GitHub releases
    fn find_latest_stable_release(
        &self,
        releases: &serde_json::Value,
        version_req: &str,
    ) -> Result<Option<String>> {
        let req = VersionReq::parse(version_req)
            .map_err(|e| anyhow!("Invalid version requirement '{version_req}': {e}"))?; // Should never fail.

        let releases_array = releases
            .as_array()
            .ok_or_else(|| anyhow!("Expected releases to be an array"))?; // Should never fail.

        let mut matching_versions = Vec::new();

        for release in releases_array {
            // Skip drafts and prereleases.
            if release
                .get("draft")
                .and_then(|d| d.as_bool())
                .unwrap_or(false)
                || release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false)
            {
                continue;
            }

            let tag_name = release
                .get("tag_name")
                .and_then(|t| t.as_str())
                .ok_or_else(|| anyhow!("Release missing tag_name"))?; // Should never fail.

            // Parse version, expecting "vM.m.p" format.
            if let Ok(version) = Version::parse(tag_name.trim_start_matches('v')) {
                // Only consider stable versions that match the requirement.
                if version.pre.is_empty() && req.matches(&version) {
                    matching_versions.push((version, tag_name.to_string()));
                }
            }
        }

        // Sort by version, newest first, and return the tag name of the latest match.
        matching_versions.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(matching_versions.first().map(|(_, tag)| tag.clone()))
    }

    pub async fn check_for_upgrades(&self) -> Result<()> {
        // Load or initialize gdvm cache
        let gdvm_cache = self.cache_store.load_gdvm_cache()?;

        // Check for updates
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))? // Should never fail.
            .as_secs();
        let cache_duration = Duration::from_secs(48 * 3600); // 48 hours
        let cache_age = now - gdvm_cache.last_update_check;

        if cache_age > cache_duration.as_secs() {
            let progress = ProgressBar::new_spinner();
            progress
                .set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
            progress.enable_steady_tick(Duration::from_millis(100));
            progress.set_message(t!(self.i18n, "checking-updates"));

            let mut new_version = None;
            let mut new_major_version = None;

            let releases = match self
                .get_github_json("https://api.github.com/repos/adalinesimonian/gdvm/releases")
                .await
            {
                Ok(json) => json,
                Err(e) => {
                    progress.finish_and_clear();
                    if matches!(e, GithubJsonError::Api(_)) {
                        eprintln!("{e}");
                    } else {
                        self.cache_store.clear_gdvm_cache(now)?;
                    }
                    return Err(e.into());
                }
            };

            // Get current version and determine major version for upgrade compatibility.
            let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
            let major_version = current_version.major;

            // Check for updates within current major version.
            let version_req = format!("^{major_version}");
            if let Some(latest_stable_tag) =
                self.find_latest_stable_release(&releases, &version_req)?
            {
                let latest_version = Version::parse(latest_stable_tag.trim_start_matches('v'))?;
                if latest_version > current_version {
                    new_version = Some(latest_stable_tag);
                }
            }

            // Check for updates across all versions.
            if let Some(latest_major_tag) = self.find_latest_stable_release(&releases, "*")? {
                let latest_major_version =
                    Version::parse(latest_major_tag.trim_start_matches('v'))?;
                if latest_major_version > current_version {
                    // Only set new_major_version if it's different from new_version.
                    if new_version.as_ref() != Some(&latest_major_tag) {
                        new_major_version = Some(latest_major_tag);
                    }
                }
            }

            progress.finish_and_clear();

            // Display appropriate message based on available updates.
            if let (Some(minor_ver), Some(major_ver)) = (&new_version, &new_major_version) {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(
                    self.i18n,
                    "upgrade-available-both",
                    minor_version = minor_ver,
                    major_version = major_ver
                );
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            } else if let Some(new_ver) = &new_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(self.i18n, "upgrade-available", version = new_ver);
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            } else if let Some(major_ver) = &new_major_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(self.i18n, "upgrade-available-major", version = major_ver);
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            }

            self.cache_store.save_gdvm_cache(&GdvmCache {
                last_update_check: now,
                new_version,
                new_major_version,
            })?;
        } else if let Some(new_version) = &gdvm_cache.new_version {
            if let Ok(new_version) = Version::parse(new_version.trim_start_matches('v')) {
                let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
                if new_version > current_version {
                    eprint!("\x1b[1;32m"); // Bold and green
                    eprintln_i18n!(
                        self.i18n,
                        "upgrade-available",
                        version = new_version.to_string(),
                    );
                    eprint!("\x1b[0m"); // Reset
                    eprintln!();
                } else {
                    // Check cached versions.
                    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
                    let mut should_clear_cache = false;

                    // Parse cached versions.
                    let cached_minor = gdvm_cache
                        .new_version
                        .as_ref()
                        .and_then(|v| Version::parse(v.trim_start_matches('v')).ok());
                    let cached_major = gdvm_cache
                        .new_major_version
                        .as_ref()
                        .and_then(|v| Version::parse(v.trim_start_matches('v')).ok());

                    // Check if cached versions are still newer than current.
                    let valid_minor = cached_minor
                        .as_ref()
                        .map(|v| v > &current_version)
                        .unwrap_or(false);
                    let valid_major = cached_major
                        .as_ref()
                        .map(|v| v > &current_version)
                        .unwrap_or(false);

                    if valid_minor && valid_major && cached_minor != cached_major {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            self.i18n,
                            "upgrade-available-both",
                            minor_version = gdvm_cache.new_version.as_ref().unwrap(),
                            major_version = gdvm_cache.new_major_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else if valid_minor {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            self.i18n,
                            "upgrade-available",
                            version = gdvm_cache.new_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else if valid_major {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            self.i18n,
                            "upgrade-available-major",
                            version = gdvm_cache.new_major_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else {
                        should_clear_cache = true;
                    }

                    if should_clear_cache {
                        self.cache_store.clear_gdvm_cache(now)?;
                    }
                }
            } else {
                self.cache_store.clear_gdvm_cache(now)?;
            }
        }

        Ok(())
    }

    pub async fn upgrade(&self, allow_major: bool) -> Result<()> {
        println_i18n!(self.i18n, "upgrade-starting");
        println_i18n!(self.i18n, "upgrade-downloading-latest");

        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        let releases = self
            .get_github_json("https://api.github.com/repos/adalinesimonian/gdvm/releases")
            .await?;

        // Determine version requirement based on allow_major flag.
        let version_req = if allow_major {
            "*".to_string() // Allow any version.
        } else {
            format!("^{}", current_version.major) // Stay within current major version.
        };

        let latest_stable_tag = self
            .find_latest_stable_release(&releases, &version_req)?
            .ok_or_else(|| {
                if allow_major {
                    anyhow!("No stable releases found")
                } else {
                    anyhow!("No stable {}.x.x releases found", current_version.major)
                }
            })?;

        // Check if upgrade is necessary.
        let latest_version = Version::parse(latest_stable_tag.trim_start_matches('v'))?;
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        match latest_version.cmp(&current_version) {
            std::cmp::Ordering::Equal => {
                println_i18n!(
                    self.i18n,
                    "upgrade-not-needed",
                    version = latest_version.to_string()
                );
                return Ok(());
            }
            std::cmp::Ordering::Less => {
                println_i18n!(
                    self.i18n,
                    "upgrade-current-version-newer",
                    current = current_version.to_string(),
                    latest = latest_version.to_string()
                );
                return Ok(());
            }
            std::cmp::Ordering::Greater => {}
        }

        // Define install directory
        let install_dir = self.get_base_path().join("bin");
        std::fs::create_dir_all(&install_dir)
            .map_err(|_| anyhow!(t!(self.i18n, "upgrade-install-dir-failed")))?;

        // Detect architecture
        let arch = detect_host(self.i18n)?.gdvm_target_triple(self.i18n)?;

        // Set download URL based on architecture.
        let repo_url = "https://github.com/adalinesimonian/gdvm";
        let release_url = format!("{repo_url}/releases/download/{latest_stable_tag}");
        #[cfg(target_os = "windows")]
        let file = format!("gdvm-{arch}.exe");
        #[cfg(not(target_os = "windows"))]
        let file = format!("gdvm-{arch}");
        let bin_url = format!("{release_url}/{file}");
        let out_file = install_dir.join("gdvm.new");

        // Download the new binary.
        if let Err(err) = download_file(&bin_url, &out_file, self.i18n).await {
            eprintln_i18n!(self.i18n, "upgrade-download-failed");
            return Err(err);
        }

        // Find the specific release to get the digest.
        let mut found_digest = None;
        if let Some(releases_array) = releases.as_array() {
            for release in releases_array {
                if release.get("tag_name").and_then(|t| t.as_str()) == Some(&latest_stable_tag) {
                    if let Some(assets) = release.get("assets").and_then(|a| a.as_array()) {
                        found_digest = assets.iter().find_map(|asset| {
                            let name = asset.get("name").and_then(|n| n.as_str());
                            let digest = asset.get("digest").and_then(|d| d.as_str());

                            if name == Some(&file) {
                                digest
                                    .and_then(|d| d.strip_prefix("sha256:"))
                                    .map(|d| d.to_string())
                            } else {
                                None
                            }
                        });
                    }
                    break;
                }
            }
        }

        if let Some(digest) = found_digest {
            if let Err(e) = verify_sha(&out_file, &digest, self.i18n) {
                let _ = std::fs::remove_file(&out_file);
                return Err(e);
            }
        } else {
            eprintln_i18n!(self.i18n, "warning-sha-sums-missing");
        }

        #[cfg(target_family = "unix")]
        {
            // Make the new binary executable
            let mut perms = out_file.metadata()?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&out_file, perms)?;
        }

        // Rename current executable to .bak and replace it with the new file
        let current_exe = std::env::current_exe()?;
        let backup_exe = current_exe.with_extension("bak");

        std::fs::rename(&current_exe, &backup_exe)
            .map_err(|_| anyhow!(t!(self.i18n, "upgrade-rename-failed")))?;
        std::fs::rename(&out_file, &current_exe)
            .map_err(|_| anyhow!(t!(self.i18n, "upgrade-replace-failed")))?;

        // Update gdvm cache
        let last_update_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        self.cache_store.clear_gdvm_cache(last_update_check)?;

        migrations::run_migrations(self.paths.base(), self.i18n)?;

        println_i18n!(self.i18n, "upgrade-complete");

        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<'a> RunVersionSource for GodotManager<'a> {
    async fn get_pinned_version(&self) -> Option<VersionSelection> {
        GodotManager::get_pinned_version(self)
    }

    async fn get_default(&self) -> Result<Option<DeterminateSelection>> {
        GodotManager::get_default(self)
    }

    async fn determine_version<P: AsRef<Path> + Send + Sync>(
        &self,
        path: Option<P>,
    ) -> Option<(GodotVersion, Option<String>)> {
        GodotManager::determine_version(self, path)
    }

    async fn auto_install_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
    ) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone + Send + Sync,
    {
        GodotManager::auto_install_version(self, gv, variant, registry, include_pre).await
    }

    async fn ensure_installed_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone + Send + Sync,
    {
        let gv: GodotVersion = gv.clone().into();
        let matches = self
            .resolve_installed_version(&gv, variant, registry)
            .await?;

        match matches.len() {
            0 => Err(anyhow!(t!(self.i18n, "error-version-not-found"))),
            1 => Ok(matches[0].version.clone()),
            _ => {
                eprintln_i18n!(self.i18n, "error-multiple-versions-found");
                for v in &matches {
                    println!(
                        "- {}",
                        crate::version_utils::display_version(
                            &v.version.to_display_str(),
                            &v.variant,
                            v.registry.as_deref(),
                        )
                    );
                }
                Err(anyhow!(t!(self.i18n, "error-version-not-found")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|(n, u)| ((*n).to_string(), (*u).to_string()))
            .collect()
    }

    #[test]
    fn registry_override_conflict_flags_differing_url() {
        let machine = pairs(&[("acme", "https://acme.example/"), ("other", "https://o/")]);
        let project = pairs(&[("acme", "file:///evil")]);

        let conflicts = registry_override_conflicts(&machine, &project);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].name, "acme");
        assert_eq!(conflicts[0].machine_url, "https://acme.example/");
        assert_eq!(conflicts[0].project_url, "file:///evil");
    }

    #[test]
    fn registry_override_conflict_ignores_matching_url_and_new_aliases() {
        let machine = pairs(&[("acme", "https://acme.example/")]);
        let project = pairs(&[
            ("acme", "https://acme.example/"),
            ("fresh", "https://fresh/"),
        ]);

        let conflicts = registry_override_conflicts(&machine, &project);

        assert!(conflicts.is_empty());
    }

    fn cache_with_tags(tags: &[&str]) -> RegistryReleasesCache {
        RegistryReleasesCache {
            last_fetched: 0,
            releases: tags
                .iter()
                .map(|tag| ReleaseCache {
                    tag_name: (*tag).to_string(),
                    variants: None,
                    source: crate::registry::ReleaseRef::V2 {
                        path: format!("releases/{tag}.json"),
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn filter_cached_releases_sorts_by_version_desc() {
        let cache = cache_with_tags(&["4.1.1-rc1", "3.5-stable", "4.1.1-stable"]);

        let releases = filter_cached_releases(&cache, None);
        let tags: Vec<String> = releases.into_iter().map(|r| r.to_remote_str()).collect();

        assert_eq!(tags, vec!["4.1.1-stable", "4.1.1-rc1", "3.5-stable"]);
    }

    #[test]
    fn filter_cached_releases_applies_filter() {
        let cache = cache_with_tags(&["4.1.1-rc1", "3.5-stable", "4.1.1-stable"]);

        let filter = GodotVersion::from_match_str("4.1.1-stable").unwrap();
        let releases = filter_cached_releases(&cache, Some(&filter));
        let tags: Vec<String> = releases.into_iter().map(|r| r.to_remote_str()).collect();

        assert_eq!(tags, vec!["4.1.1-stable"]);
    }

    #[tokio::test]
    async fn test_find_latest_stable_release() {
        let i18n = I18n::new().unwrap();
        let manager = GodotManager::new(&i18n).await.unwrap();

        // Mock release data based on GitHub API response.
        let releases = json!([
            {
                "tag_name": "v1.0.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.9.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.1",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.7.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0-beta.1",
                "draft": false,
                "prerelease": true
            },
            {
                "tag_name": "v0.9.0-rc.1",
                "draft": false,
                "prerelease": false
            }
        ]);

        let result = manager.find_latest_stable_release(&releases, "^0").unwrap();

        assert_eq!(result, Some("v0.9.0".to_string()));

        let result = manager
            .find_latest_stable_release(&releases, "^0.8")
            .unwrap();

        assert_eq!(result, Some("v0.8.1".to_string()));

        let result = manager
            .find_latest_stable_release(&releases, "=0.8.0")
            .unwrap();

        assert_eq!(result, Some("v0.8.0".to_string()));
    }

    #[tokio::test]
    async fn test_find_latest_stable_release_no_matches() {
        let i18n = I18n::new().unwrap();
        let manager = GodotManager::new(&i18n).await.unwrap();

        // Mock releases with no stable 0.x.x versions.
        let releases = json!([
            {
                "tag_name": "v1.0.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v1.1.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0-beta.1",
                "draft": false,
                "prerelease": true
            }
        ]);

        let result = manager.find_latest_stable_release(&releases, "^0").unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_find_latest_stable_release_skips_drafts() {
        let i18n = I18n::new().unwrap();
        let manager = GodotManager::new(&i18n).await.unwrap();

        // Mock releases with drafts.
        let releases = json!([
            {
                "tag_name": "v0.9.0",
                "draft": true,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0",
                "draft": false,
                "prerelease": false
            }
        ]);

        let result = manager.find_latest_stable_release(&releases, "^0").unwrap();

        assert_eq!(result, Some("v0.8.0".to_string()));
    }

    #[tokio::test]
    async fn test_find_latest_stable_release_invalid_requirement() {
        let i18n = I18n::new().unwrap();
        let manager = GodotManager::new(&i18n).await.unwrap();

        let releases = json!([]);

        let result = manager.find_latest_stable_release(&releases, "invalid-version-req");

        assert!(result.is_err());
    }
}
