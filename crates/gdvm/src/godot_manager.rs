use crate::artifact_cache::ArtifactCache;
use crate::config::Config;
use crate::host::{HostPlatform, detect_host};
use crate::metadata_cache::{CacheStore, GdvmCache};
#[cfg(test)]
use crate::metadata_cache::{RegistryReleasesCache, ReleaseCache, filter_cached_releases};
use crate::paths::GdvmPaths;
use crate::registry::{self, BinarySelectionError, Registry};
use crate::releases::ReleaseCatalog;
use crate::run_version_resolver::RunVersionSource;
use anyhow::{Result, anyhow};
#[cfg(target_family = "unix")]
use daemonize::Daemonize;
use i18n::I18n;
use indicatif::{ProgressBar, ProgressStyle};
use semver::{Version, VersionReq};
use sha2::{Digest, Sha256, Sha512};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use crate::download_utils::download_file;
use crate::migrations;
use crate::registry_version_resolver::RegistryVersionResolver;
use crate::version_utils::GodotVersion;
use crate::zip_utils;
use crate::{eprintln_i18n, println_i18n};
use crate::{i18n, project_version_detector, t, t_w};

use crate::version_utils::{GodotVersionDeterminate, GodotVersionDeterminateVecExt};

#[derive(Debug)]
pub enum InstallOutcome {
    Installed,
    AlreadyInstalled,
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
    /// Release catalog for fetching Godot versions
    release_catalog: ReleaseCatalog,
    /// Client for GitHub API requests
    client: reqwest::Client,
    /// Host platform
    host: HostPlatform,
    i18n: &'a I18n,
}

/// Verifies the SHA of a file against an expected hash.
fn verify_sha(file_path: &Path, expected: &str, i18n: &I18n) -> Result<()> {
    let sha_type = ShaType::from_hash_length(expected).ok_or_else(|| {
        anyhow!(t_w!(
            i18n,
            "error-invalid-sha-length",
            length = expected.len()
        ))
    })?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(t_w!(i18n, "verifying-checksum"));

    let mut f = fs::File::open(file_path)?;

    let local_hash = match sha_type {
        ShaType::Sha256 => {
            let mut hasher = Sha256::new();
            std::io::copy(&mut f, &mut hasher)?;
            format!("{:x}", hasher.finalize())
        }
        ShaType::Sha512 => {
            let mut hasher = Sha512::new();
            std::io::copy(&mut f, &mut hasher)?;
            format!("{:x}", hasher.finalize())
        }
    };

    if local_hash == expected.to_lowercase() {
        pb.finish_with_message(t_w!(i18n, "checksum-verified"));
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

impl<'a> GodotManager<'a> {
    /// Create a new GodotManager instance and set up the installation and cache paths
    pub async fn new(i18n: &'a I18n) -> Result<Self> {
        let paths = GdvmPaths::new(i18n)?;
        let artifact_cache = ArtifactCache::new(paths.cache_dir().to_path_buf());
        artifact_cache.ensure_dir()?;

        let client = GodotManager::get_github_client(i18n)?;
        let registry = Registry::new()?;
        let cache_store = CacheStore::new(paths.cache_index().to_path_buf());
        let release_catalog = ReleaseCatalog::new(registry, cache_store);
        let host = detect_host(i18n)?;

        let manager = GodotManager {
            paths,
            artifact_cache,
            release_catalog,
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

    /// Install a specified Godot version
    ///
    /// - `force`: If true, reinstall the version even if it's already installed.
    /// - `redownload`: If true, ignore cached zip files and download fresh ones.
    pub async fn install(
        &self,
        gv: &GodotVersionDeterminate,
        force: bool,
        redownload: bool,
    ) -> Result<InstallOutcome> {
        let install_str = gv.to_install_str();
        let version_path = self.paths.installs().join(install_str);

        if version_path.exists() {
            if force {
                self.remove(gv)?;
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

        let meta = self.release_catalog.metadata_for(gv, self.i18n).await?;
        let is_csharp = gv.is_csharp.unwrap_or(false);

        let binary =
            registry::select_binary(&meta, self.host, is_csharp).map_err(|err| match err {
                BinarySelectionError::UnsupportedPlatform => {
                    anyhow!(t_w!(self.i18n, "unsupported-platform"))
                }
                BinarySelectionError::UnsupportedArch => {
                    anyhow!(t_w!(self.i18n, "unsupported-architecture"))
                }
                BinarySelectionError::MissingUrl => {
                    anyhow!(t_w!(self.i18n, "error-file-not-found"))
                }
            })?;

        let download_url = binary.urls.first().unwrap();
        let archive_name = download_url.split('/').next_back().unwrap_or("godot.zip");
        let cache_zip_path = self.artifact_cache.cached_zip_path(archive_name);

        if !redownload && cache_zip_path.exists() {
            eprintln_i18n!(self.i18n, "using-cached-zip");
        } else {
            if redownload && cache_zip_path.exists() {
                eprintln_i18n!(self.i18n, "force-redownload", version = gv.to_display_str());
            }

            let tmp_file = self
                .paths
                .installs()
                .join(format!("{}.zip", gv.to_install_str()));

            // Download the archive
            download_file(download_url, &tmp_file, self.i18n).await?;
            verify_sha(&tmp_file, &binary.sha512, self.i18n)?;

            // Move the verified zip to cache_dir
            fs::rename(&tmp_file, &cache_zip_path)?;
            eprintln_i18n!(self.i18n, "cached-zip-stored");
        }

        fs::create_dir_all(&version_path)?;

        // Extract from cache_zip_path
        zip_utils::extract_zip(&cache_zip_path, &version_path, self.i18n)?;

        Ok(InstallOutcome::Installed)
    }

    /// List all installed Godot versions
    pub fn list_installed(&self) -> Result<Vec<GodotVersionDeterminate>> {
        let mut versions = vec![];
        for entry in fs::read_dir(self.paths.installs())? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(gv) = GodotVersion::from_install_str(&name) {
                    versions.push(gv.to_determinate())
                }
            }
        }
        versions.sort_by_version();
        Ok(versions)
    }

    /// Remove a specified Godot version
    pub fn remove(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        let path = self.paths.installs().join(gv.to_install_str());

        if path.exists() {
            // If this version is the default, unset it
            if let Some(def) = self.get_default()?
                && def.to_install_str() == path.file_name().unwrap().to_string_lossy()
            {
                self.unset_default()?;
            }
            fs::remove_dir_all(path)?;
            Ok(())
        } else {
            Err(anyhow!(t_w!(self.i18n, "error-version-not-found")))
        }
    }

    /// Run a specified Godot version
    pub fn run(
        &self,
        gv: &GodotVersionDeterminate,
        console: bool,
        godot_args: &[String],
    ) -> Result<()> {
        let path = self.get_executable_path(gv, console)?;

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
        console: bool,
    ) -> Result<std::path::PathBuf> {
        let version_dir = self.paths.installs().join(gv.to_install_str());
        if !version_dir.exists() {
            return Err(anyhow!(t!(
                self.i18n,
                "error-version-not-found",
                version = &gv.to_display_str(),
            )));
        }

        let godot_executable = find_godot_executable(&version_dir, console)?;

        godot_executable.ok_or_else(|| {
            anyhow!(t!(
                self.i18n,
                "godot-executable-not-found",
                version = &gv.to_display_str(),
            ))
        })
    }

    /// Fetch available releases from GitHub with caching
    pub async fn fetch_available_releases(
        &self,
        filter: &Option<GodotVersion>,
        use_cache_only: bool,
    ) -> Result<Vec<GodotVersionDeterminate>> {
        self.release_catalog
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
            return Err(GithubJsonError::Api(anyhow!(t_w!(
                self.i18n,
                "error-github-rate-limit"
            ))));
        }

        let error_message = json.get("message").and_then(|m| m.as_str());

        Err(GithubJsonError::Api(anyhow!(t_w!(
            self.i18n,
            "error-github-api",
            error = error_message,
        ))))
    }

    /// Clears the release cache by deleting the cache file and all cached zip files
    pub fn clear_cache(&self) -> Result<()> {
        let cache_index = self.release_catalog.cache_store().index_path();
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

    /// Refresh the gdvm release cache by re-downloading the registry index.
    pub async fn refresh_cache(&self) -> Result<()> {
        self.release_catalog.refresh_cache(self.i18n).await
    }

    /// Resolve the Godot version from a string, for an installed version
    /// Returns a list of possible versions. If the input is ambiguous, the list
    /// will have more than one element. Otherwise, it will have one element,
    /// unless of course the version is not found, in which case the list will
    /// be empty.
    /// Accepts full and partial versions.
    pub async fn resolve_installed_version<T>(&self, gv: &T) -> Result<Vec<GodotVersionDeterminate>>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let installed = self.list_installed()?;
        let resolver = RegistryVersionResolver::new(&self.release_catalog, self.i18n, self.host);
        Ok(resolver.resolve_installed(&gv, &installed).await)
    }

    /// Resolve the Godot version from a string, for an available version
    /// Returns a single version, whichever is the latest that matches the
    /// input.
    /// Accepts full and partial versions.
    pub async fn resolve_available_version<T>(
        &self,
        gv: &T,
        use_cache_only: bool,
    ) -> Result<Option<GodotVersionDeterminate>>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let resolver = RegistryVersionResolver::new(&self.release_catalog, self.i18n, self.host);
        resolver.resolve_available(&gv, use_cache_only).await
    }

    pub fn set_default(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        // Check if the version exists
        let version_str = gv.to_install_str();
        let version_path = self.paths.installs().join(&version_str);
        if !version_path.exists() {
            return Err(anyhow!(t_w!(self.i18n, "error-version-not-found")));
        }

        // Write version to .gdvm/default
        let default_path = self.paths.default_file();
        fs::write(&default_path, &version_str)?;

        // Create directory symlink .gdvm/bin/current_godot -> .gdvm/<version_str>/
        let symlink_dir = self.paths.current_godot_symlink();
        let target_dir = self.paths.installs().join(version_str);

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
                return Err(anyhow!(t_w!(self.i18n, "error-create-symlink-windows")));
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

    pub fn get_default(&self) -> Result<Option<GodotVersionDeterminate>> {
        let default_file = self.paths.default_file();
        if default_file.exists() {
            let contents = fs::read_to_string(&default_file)?;
            Ok(Some(
                GodotVersion::from_install_str(contents.trim())?.to_determinate(),
            ))
        } else {
            Ok(None)
        }
    }

    /// Recursively search upward for .gdvmrc, return the pinned version if found
    pub fn get_pinned_version(&self) -> Option<GodotVersion> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let candidate = current.join(".gdvmrc");
            if candidate.is_file()
                && let Ok(contents) = fs::read_to_string(&candidate)
            {
                return GodotVersion::from_install_str(contents.trim()).ok();
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Try to determine the version to use based on the current Godot project
    pub fn determine_version<P: AsRef<Path>>(&self, path: Option<P>) -> Option<GodotVersion> {
        let current_dir = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => std::env::current_dir().ok()?,
        };

        project_version_detector::detect_godot_version_in_path(self.i18n, &current_dir)
    }

    /// Pin a version to .gdvmrc in the current directory
    pub fn pin_version(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        let path = std::env::current_dir()?;
        let file = path.join(".gdvmrc");
        fs::write(&file, gv.to_pinned_str())?;
        Ok(())
    }

    pub async fn auto_install_version<T>(&self, gv: &T) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let resolver = RegistryVersionResolver::new(&self.release_catalog, self.i18n, self.host);

        let actual_version = resolver.resolve_for_auto_install(&gv).await?;

        // Check if version is installed, if not, install
        if !self.is_version_installed(&actual_version)? {
            eprintln_i18n!(
                self.i18n,
                "auto-installing-version",
                version = &actual_version.to_display_str(),
            );
            self.install(&actual_version, false, false).await?;
        }
        Ok(actual_version)
    }

    fn is_version_installed<T>(&self, gv: &T) -> Result<bool>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();

        let installed_versions = self.list_installed()?;
        Ok(installed_versions.iter().any(|v| gv.matches(v)))
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
        let gdvm_cache = self.release_catalog.cache_store().load_gdvm_cache()?;

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
            progress.set_message(t_w!(self.i18n, "checking-updates"));

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
                        self.release_catalog.cache_store().clear_gdvm_cache(now)?;
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

            self.release_catalog
                .cache_store()
                .save_gdvm_cache(&GdvmCache {
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
                        self.release_catalog.cache_store().clear_gdvm_cache(now)?;
                    }
                }
            } else {
                self.release_catalog.cache_store().clear_gdvm_cache(now)?;
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
            .map_err(|_| anyhow!(t_w!(self.i18n, "upgrade-install-dir-failed")))?;

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
            .map_err(|_| anyhow!(t_w!(self.i18n, "upgrade-rename-failed")))?;
        std::fs::rename(&out_file, &current_exe)
            .map_err(|_| anyhow!(t_w!(self.i18n, "upgrade-replace-failed")))?;

        // Update gdvm cache
        let last_update_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        self.release_catalog
            .cache_store()
            .clear_gdvm_cache(last_update_check)?;

        migrations::run_migrations(self.paths.base(), self.i18n)?;

        println_i18n!(self.i18n, "upgrade-complete");

        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<'a> RunVersionSource for GodotManager<'a> {
    async fn get_pinned_version(&self) -> Option<GodotVersion> {
        GodotManager::get_pinned_version(self)
    }

    async fn get_default(&self) -> Result<Option<GodotVersionDeterminate>> {
        GodotManager::get_default(self)
    }

    async fn determine_version<P: AsRef<Path> + Send + Sync>(
        &self,
        path: Option<P>,
    ) -> Option<GodotVersion> {
        GodotManager::determine_version(self, path)
    }

    async fn auto_install_version<T>(&self, gv: &T) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone + Send + Sync,
    {
        GodotManager::auto_install_version(self, gv).await
    }

    async fn ensure_installed_version<T>(&self, gv: &T) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone + Send + Sync,
    {
        let gv: GodotVersion = gv.clone().into();
        let matches = self.resolve_installed_version(&gv).await?;

        match matches.len() {
            0 => Err(anyhow!(t!(self.i18n, "error-version-not-found"))),
            1 => Ok(matches[0].clone()),
            _ => {
                eprintln_i18n!(self.i18n, "error-multiple-versions-found");
                for v in matches {
                    println!("- {}", v.to_display_str());
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

    fn cache_with_tags(tags: &[&str]) -> RegistryReleasesCache {
        RegistryReleasesCache {
            last_fetched: 0,
            releases: tags
                .iter()
                .enumerate()
                .map(|(idx, tag)| ReleaseCache {
                    id: idx as u64,
                    tag_name: (*tag).to_string(),
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
        let i18n = I18n::new(100).unwrap();
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
        let i18n = I18n::new(100).unwrap();
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
        let i18n = I18n::new(100).unwrap();
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
        let i18n = I18n::new(100).unwrap();
        let manager = GodotManager::new(&i18n).await.unwrap();

        let releases = json!([]);

        let result = manager.find_latest_stable_release(&releases, "invalid-version-req");

        assert!(result.is_err());
    }
}
