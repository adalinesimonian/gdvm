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

use crate::host::detect_host;
use crate::metadata_cache::{CacheStore, GdvmCache};
use crate::paths::GdvmPaths;
use crate::self_update;
use anyhow::{Result, anyhow};
use semver::Version;
#[cfg(target_family = "unix")]
use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::download_utils::download_to_file;
use crate::migrations;
use crate::progress_utils;
use crate::t;
use crate::{eprintln_i18n, println_i18n};

#[derive(Clone, Copy)]
pub struct Updater<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) cache_store: &'a CacheStore,
}

impl<'a> Updater<'a> {
    pub async fn check_for_upgrades(&self) -> Result<()> {
        // Load or initialize gdvm cache
        let gdvm_cache = self.cache_store.load_gdvm_cache()?;

        // Check for updates
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))? // Should never fail.
            .as_secs();
        let cache_duration = Duration::from_secs(48 * 3600); // 48 hours
        let cache_age = crate::date_utils::age_secs(now, gdvm_cache.last_update_check);

        if cache_age > cache_duration.as_secs() {
            let progress = progress_utils::spinner(t!("checking-updates"))?;

            let manifest = match self_update::fetch_manifest(
                &self_update::releases_url(),
                Duration::from_secs(3),
            )
            .await
            {
                Ok(manifest) => manifest,
                Err(_) => {
                    progress.finish_and_clear();
                    self.cache_store.clear_gdvm_cache(now)?;
                    return Ok(());
                }
            };

            progress.finish_and_clear();

            // Get current version and determine major version for upgrade compatibility.
            let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

            let new_version =
                self_update::select_upgrade(&manifest.releases, &current_version, false, false)
                    .map(|r| r.normalized_version().to_string());

            let new_major_version =
                self_update::select_upgrade(&manifest.releases, &current_version, true, false)
                    .map(|r| r.normalized_version().to_string())
                    .filter(|major| Some(major) != new_version.as_ref());

            // Display appropriate message based on available updates.
            if let (Some(minor_ver), Some(major_ver)) = (&new_version, &new_major_version) {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(
                    "upgrade-available-both",
                    minor_version = minor_ver,
                    major_version = major_ver
                );
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            } else if let Some(new_ver) = &new_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!("upgrade-available", version = new_ver);
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            } else if let Some(major_ver) = &new_major_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!("upgrade-available-major", version = major_ver);
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
                    eprintln_i18n!("upgrade-available", version = new_version.to_string(),);
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
                            "upgrade-available-both",
                            minor_version = gdvm_cache.new_version.as_ref().unwrap(),
                            major_version = gdvm_cache.new_major_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else if valid_minor {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            "upgrade-available",
                            version = gdvm_cache.new_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else if valid_major {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
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

    pub async fn upgrade(&self, allow_major: bool, allow_pre: bool) -> Result<()> {
        println_i18n!("upgrade-starting");
        println_i18n!("upgrade-downloading-latest");

        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        let manifest =
            self_update::fetch_manifest(&self_update::releases_url(), Duration::from_secs(10))
                .await?;

        let target = match self_update::select_upgrade(
            &manifest.releases,
            &current_version,
            allow_major,
            allow_pre,
        ) {
            Some(target) => target,
            None => {
                if !allow_pre
                    && self_update::newer_prerelease_available(
                        &manifest.releases,
                        &current_version,
                        allow_major,
                    )
                {
                    println_i18n!("upgrade-prerelease-available");
                }

                match self_update::highest_stable(&manifest.releases, &current_version, allow_major)
                {
                    Some(latest) if latest < current_version => {
                        println_i18n!(
                            "upgrade-current-version-newer",
                            current = current_version.to_string(),
                            latest = latest.to_string()
                        );
                    }
                    _ => {
                        println_i18n!("upgrade-not-needed", version = current_version.to_string());
                    }
                }
                return Ok(());
            }
        };

        let target_version = Version::parse(target.normalized_version())?;

        if target_version <= current_version {
            println_i18n!("upgrade-not-needed", version = current_version.to_string());
            return Ok(());
        }

        let triple = detect_host()?.gdvm_target_triple()?;
        let binary = target.binary_for(triple).ok_or_else(|| {
            anyhow!(t!(
                "upgrade-no-binary",
                version = target_version.to_string(),
                target = triple
            ))
        })?;
        let bin_url = binary.urls.first().ok_or_else(|| {
            anyhow!(t!(
                "upgrade-no-binary",
                version = target_version.to_string(),
                target = triple
            ))
        })?;

        let expected_sha = binary
            .sha256
            .as_deref()
            .ok_or_else(|| anyhow!(t!("upgrade-checksum-required")))?;

        // Define install directory
        let install_dir = self.paths.base().join("bin");
        std::fs::create_dir_all(&install_dir)
            .map_err(|e| anyhow!(t!("upgrade-install-dir-failed", error = e.to_string())))?;

        let tmp_file = tempfile::Builder::new()
            .prefix(".gdvm-upgrade-")
            .tempfile_in(&install_dir)
            .map_err(|e| anyhow!(t!("upgrade-file-create-failed", error = e.to_string())))?;

        let mut async_file = tokio::fs::File::from_std(tmp_file.as_file().try_clone()?);
        let digests = match download_to_file(bin_url, &mut async_file).await {
            Ok(digests) => digests,
            Err(err) => {
                eprintln_i18n!("upgrade-download-failed", error = err.to_string());
                return Err(err);
            }
        };
        drop(async_file);

        super::installer::verify_download_digests(&digests, expected_sha, Path::new("gdvm"))?;

        if let Some(expected_size) = binary.size
            && digests.size != expected_size
        {
            return Err(anyhow!(t!(
                "error-size-mismatch",
                file = "gdvm",
                expected = expected_size,
                actual = digests.size
            )));
        }

        #[cfg(target_family = "unix")]
        {
            // Make the new binary executable
            tmp_file
                .as_file()
                .set_permissions(fs::Permissions::from_mode(0o755))?;
        }

        // Rename current executable to .bak and replace it with the new file
        let current_exe = std::env::current_exe()?;
        let backup_exe = current_exe.with_extension("bak");

        std::fs::rename(&current_exe, &backup_exe)
            .map_err(|e| anyhow!(t!("upgrade-rename-failed", error = e.to_string())))?;

        if let Err(err) = tmp_file.persist(&current_exe) {
            let _ = std::fs::rename(&backup_exe, &current_exe);
            return Err(anyhow!(t!(
                "upgrade-replace-failed",
                error = err.to_string()
            )));
        }

        // Update gdvm cache
        let last_update_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        self.cache_store.clear_gdvm_cache(last_update_check)?;

        migrations::run_migrations(self.paths.base())?;

        println_i18n!("upgrade-complete");

        Ok(())
    }
}
