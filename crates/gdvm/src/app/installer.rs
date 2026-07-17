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

use std::fs;
use std::io::Seek;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::*;
use crate::artifact_cache::ArtifactCache;
use crate::hash_utils::{self, ShaType};
use crate::paths::GdvmPaths;
use crate::registry_version_resolver::RegistryVersionResolver;
use crate::usage_tracker::UsageTracker;
use crate::version::{ResolvedVersion, Variant, VersionQuery};
use crate::{t, terr, ui, zip_utils};

#[derive(Debug)]
pub enum InstallOutcome {
    Installed,
    AlreadyInstalled,
}

/// Get the path to where to extract versions before moving them into place.
fn staging_path_for(version_path: &Path) -> Result<PathBuf> {
    let name = version_path
        .file_name()
        .ok_or_else(|| terr!("error-invalid-path"))?
        .to_string_lossy();
    Ok(version_path.with_file_name(format!(".staging-{name}")))
}

/// Directory where an extracted Godot was staged, along with whether or not it
/// was committed, i.e. written into its final destination.
struct StagingDir {
    path: PathBuf,
    committed: bool,
}

impl StagingDir {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            committed: false,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// Move the fully extracted directory into place.
    fn commit(mut self, version_path: &Path) -> Result<()> {
        fs::rename(&self.path, version_path)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for StagingDir {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// Verifies the SHA of a file against an expected hash.
fn verify_sha_file(file: &mut fs::File, expected: &str, display_path: &Path) -> Result<()> {
    let sha_type = ShaType::from_expected(expected)?;

    let _task = ui::progress::activity(t!("status-verifying"), t!("subject-cached-archive"));

    file.rewind()?;

    let local_hash = hash_utils::hash_reader(sha_type, file)?;

    if local_hash == expected.to_lowercase() {
        Ok(())
    } else {
        Err(crate::hash_utils::checksum_mismatch_error(display_path))
    }
}

#[derive(Clone, Copy)]
pub struct Installer<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) artifact_cache: &'a ArtifactCache,
    pub(super) usage_tracker: &'a UsageTracker,
    pub(super) catalogs: Catalogs<'a>,
}

impl<'a> Installer<'a> {
    fn catalogs(&self) -> Catalogs<'a> {
        self.catalogs
    }

    fn library(&self) -> Library<'a> {
        Library {
            paths: self.paths,
            usage_tracker: self.usage_tracker,
            catalogs: self.catalogs,
        }
    }

    /// Install a specified Godot version
    ///
    /// - `variant`: Optional variant, e.g. `Some("csharp")`.
    /// - `registry`: Optional registry name, `None` uses gdvm's official registry.
    /// - `force`: If true, reinstall the version even if it's already installed.
    /// - `redownload`: If true, ignore cached zip files and download fresh ones.
    pub async fn install(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
        force: bool,
        redownload: bool,
    ) -> Result<InstallOutcome> {
        let store_key = self.library().install_store_key(registry)?;
        let install_str =
            crate::version::install_dir_subpath(&store_key, &gv.to_remote_str(), variant);
        let version_path = self.paths.installs().join(&install_str);

        let _lock = crate::locks::Lock::acquire(
            &self.paths.locks(),
            crate::locks::Resource::Install(&install_str),
        )?;

        let display = crate::version::display_version(gv, variant, registry);

        if version_path.exists() {
            let healthy = crate::app::find_godot_executable(&version_path, false)?.is_some();

            if force || !healthy {
                if !healthy && !force {
                    ui::warn(t!(
                        "warning-broken-install-reinstalling",
                        version = &display
                    ));
                }
                self.library()
                    .remove_locked(gv, variant, registry, &install_str)?;
            } else {
                return Ok(InstallOutcome::AlreadyInstalled);
            }
        }

        ui::milestone(t!("status-installing"), &display);

        if !gv.is_stable() {
            ui::warn(t!("warning-prerelease", branch = &gv.release_type));
        }

        self.artifact_cache.ensure_dir()?;

        let meta = self.catalogs().catalog(registry)?.metadata_for(gv).await?;

        let binary = self.catalogs().select_platform_binary(&meta, variant)?;

        let download_url = binary.urls.first().unwrap();
        let cache_zip_path = self.artifact_cache.cached_zip_path(&binary.sha512);
        let partial_path = self.artifact_cache.partial_zip_path(&binary.sha512);
        let partial_meta_path = self.artifact_cache.partial_meta_path(&binary.sha512);

        // Get rid of any partial downloads older than 24 hours, except for the
        // partial files that are currently being used for this download.
        self.artifact_cache.sweep_stale_partials(
            std::time::Duration::from_secs(24 * 60 * 60),
            &[&partial_path, &partial_meta_path],
        );

        let mut cached_zip: Option<fs::File> = None;

        if !redownload
            && let Ok(mut file) = fs::File::open(&cache_zip_path)
            && verify_sha_file(&mut file, &binary.sha512, &cache_zip_path).is_ok()
        {
            cached_zip = Some(file);
        }

        let mut zip_file = match cached_zip {
            Some(file) => file,
            None => {
                crate::download_utils::download_verified(
                    download_url,
                    &cache_zip_path,
                    &partial_path,
                    &partial_meta_path,
                    crate::download_utils::ExpectedDigests {
                        sha: &binary.sha512,
                        size: binary.size,
                    },
                    &display,
                )
                .await?
            }
        };

        self.track_archive_use(&cache_zip_path)?;

        let staging_path = staging_path_for(&version_path)?;
        if staging_path.exists() {
            fs::remove_dir_all(&staging_path)?;
        }
        fs::create_dir_all(&staging_path)?;
        let staging = StagingDir::new(staging_path);

        // Extract from cache_zip_path
        zip_utils::extract_zip_from_file(&mut zip_file, &cache_zip_path, staging.path(), &display)?;

        staging.commit(&version_path)?;

        let base_url = self.catalogs().catalog(registry)?.registry_base_url();
        let store_dir = self.paths.installs().join(&store_key);
        crate::registry_store::upsert(&store_dir, &base_url, registry, None)?;

        self.library().track_install_use(&install_str)?;

        ui::milestone(t!("status-installed"), &display);

        Ok(InstallOutcome::Installed)
    }

    /// Resolve the path to the cached download archive for a release.
    pub async fn cached_archive_path(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<PathBuf> {
        let meta = self.catalogs().catalog(registry)?.metadata_for(gv).await?;
        let binary = self.catalogs().select_platform_binary(&meta, variant)?;
        let path = self.artifact_cache.cached_zip_path(&binary.sha512);
        if !path.exists() {
            return Err(terr!(
                "error-archive-not-cached",
                version = crate::version::display_version(gv, variant, registry)
            ));
        }
        self.track_archive_use(&path)?;
        Ok(path)
    }

    pub async fn auto_install_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
    ) -> Result<ResolvedVersion>
    where
        T: Into<VersionQuery> + Clone,
    {
        let gv: VersionQuery = gv.clone().into();
        let resolver =
            RegistryVersionResolver::new(self.catalogs().catalog(registry)?, *self.catalogs.host);

        let actual_version = resolver
            .resolve_for_auto_install(&gv, variant, include_pre)
            .await?;

        // Check if version is installed, if not, install
        if !self
            .library()
            .is_version_installed(&actual_version, variant, registry)?
        {
            ui::note(t!(
                "auto-installing-version",
                version = &crate::version::display_version(
                    &actual_version,
                    &Variant::from_option(variant),
                    registry,
                ),
            ));
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

    /// Record that a cached archive was used.
    fn track_archive_use(&self, archive_path: &Path) -> Result<()> {
        if let Some(name) = archive_path.file_name().and_then(|n| n.to_str()) {
            self.usage_tracker.record_archive(name)?;
        }
        Ok(())
    }
}
