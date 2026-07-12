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

use anyhow::{Result, anyhow, bail};

use super::*;
use crate::artifact_cache::ArtifactCache;
use crate::download_utils::download_to_file;
use crate::hash_utils::{self, ShaType};
use crate::paths::GdvmPaths;
use crate::registry_version_resolver::RegistryVersionResolver;
use crate::usage_tracker::UsageTracker;
use crate::version::{ResolvedVersion, Variant, VersionQuery};
use crate::{eprintln_i18n, progress_utils, t, zip_utils};

#[derive(Debug)]
pub enum InstallOutcome {
    Installed,
    AlreadyInstalled,
}

/// Build the checksum mismatch error for the given file.
fn checksum_mismatch_error(display_path: &Path) -> anyhow::Error {
    anyhow!(t!(
        "error-checksum-mismatch",
        file = display_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    ))
}

/// Verifies the SHA of a file against an expected hash.
fn verify_sha_file(file: &mut fs::File, expected: &str, display_path: &Path) -> Result<()> {
    let sha_type = ShaType::from_expected(expected)?;

    let pb = progress_utils::spinner(t!("verifying-checksum"))?;

    file.rewind()?;

    let local_hash = hash_utils::hash_reader(sha_type, file)?;

    if local_hash == expected.to_lowercase() {
        pb.finish_with_message(t!("checksum-verified"));
        Ok(())
    } else {
        pb.finish_and_clear();
        Err(checksum_mismatch_error(display_path))
    }
}

/// Checks streamed download digests against an expected hash, choosing SHA-256
/// or SHA-512 based on the hash length.
pub(super) fn verify_download_digests(
    digests: &crate::download_utils::DownloadDigests,
    expected: &str,
    display_path: &Path,
) -> Result<()> {
    let sha_type = ShaType::from_expected(expected)?;

    let actual = match sha_type {
        ShaType::Sha256 => &digests.sha256,
        ShaType::Sha512 => &digests.sha512,
    };

    if actual.eq_ignore_ascii_case(expected) {
        eprintln_i18n!("checksum-verified");
        Ok(())
    } else {
        Err(checksum_mismatch_error(display_path))
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

        if version_path.exists() {
            if force {
                self.library()
                    .remove_locked(gv, variant, registry, &install_str)?;
                eprintln_i18n!("force-reinstalling-version", version = gv.to_display_str(),);
            } else {
                return Ok(InstallOutcome::AlreadyInstalled);
            }
        }

        if !gv.is_stable() {
            eprintln_i18n!("warning-prerelease", branch = &gv.release_type);
        }

        self.artifact_cache.ensure_dir()?;

        let meta = self.catalogs().catalog(registry)?.metadata_for(gv).await?;

        let binary = self.catalogs().select_platform_binary(&meta, variant)?;

        let download_url = binary.urls.first().unwrap();
        let cache_zip_path = self.artifact_cache.cached_zip_path(&binary.sha512);

        let mut cached_zip: Option<fs::File> = None;

        if !redownload
            && let Ok(mut file) = fs::File::open(&cache_zip_path)
            && verify_sha_file(&mut file, &binary.sha512, &cache_zip_path).is_ok()
        {
            eprintln_i18n!("using-cached-zip");
            cached_zip = Some(file);
        }

        let mut zip_file = match cached_zip {
            Some(file) => file,
            None => {
                if redownload && cache_zip_path.exists() {
                    eprintln_i18n!("force-redownload", version = gv.to_display_str());
                }

                let tmp_file = tempfile::Builder::new()
                    .prefix(".partial-")
                    .suffix(".zip")
                    .tempfile_in(self.artifact_cache.dir())?;

                let mut async_file = tokio::fs::File::from_std(tmp_file.as_file().try_clone()?);
                let digests = download_to_file(download_url, &mut async_file).await?;
                drop(async_file);

                verify_download_digests(&digests, &binary.sha512, &cache_zip_path)?;

                if let Some(expected_size) = binary.size
                    && digests.size != expected_size
                {
                    return Err(anyhow!(t!(
                        "error-size-mismatch",
                        file = cache_zip_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        expected = expected_size,
                        actual = digests.size
                    )));
                }

                let file = tmp_file.persist(&cache_zip_path)?;

                eprintln_i18n!("cached-zip-stored");

                file
            }
        };

        fs::create_dir_all(&version_path)?;

        self.track_archive_use(&cache_zip_path)?;

        // Extract from cache_zip_path
        zip_utils::extract_zip_from_file(&mut zip_file, &cache_zip_path, &version_path)?;

        let base_url = self.catalogs().catalog(registry)?.registry_base_url();
        let store_dir = self.paths.installs().join(&store_key);
        crate::registry_store::upsert(&store_dir, &base_url, registry, None)?;

        self.library().track_install_use(&install_str)?;

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
            bail!(t!(
                "error-archive-not-cached",
                version = gv.to_display_str()
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
            eprintln_i18n!(
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

    /// Record that a cached archive was used.
    fn track_archive_use(&self, archive_path: &Path) -> Result<()> {
        if let Some(name) = archive_path.file_name().and_then(|n| n.to_str()) {
            self.usage_tracker.record_archive(name)?;
        }
        Ok(())
    }
}
