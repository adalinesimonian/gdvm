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

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use semver::Version;

use crate::host::detect_host;
use crate::metadata_cache::{CacheStore, GdvmCache};
use crate::paths::GdvmPaths;
use crate::{println_i18n, self_update, t, terr, ui};

#[derive(Clone, Copy)]
pub struct Updater<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) cache_store: &'a CacheStore,
}

impl<'a> Updater<'a> {
    /// How long to wait before running another update check.
    const CHECK_INTERVAL: Duration = Duration::from_secs(48 * 3600);

    /// Environment variable identifying the instance of gdvm that's running the
    /// update check in the background.
    pub const BACKGROUND_CHECK_ENV_VAR: &'static str = "GDVM_INTERNAL_UPDATE_CHECK";

    /// Print an upgrade notice from the cached result of the last update check.
    pub fn print_upgrade_notice(&self) -> Result<()> {
        let gdvm_cache = self.cache_store.load_gdvm_cache()?;
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        match upgrade_notice(&gdvm_cache, &current_version) {
            NoticeState::Notice(notice) => print_notice(&notice),
            NoticeState::Stale => {
                self.cache_store
                    .clear_gdvm_cache(gdvm_cache.last_update_check)?;
            }
            NoticeState::None => {}
        }

        Ok(())
    }

    /// Run an update check in the background if the last check was more than
    /// `CHECK_INTERVAL` ago.
    pub fn spawn_background_check_if_due(&self) -> Result<()> {
        let gdvm_cache = self.cache_store.load_gdvm_cache()?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| terr!("error-system-time"))? // Should never fail.
            .as_secs();
        let cache_age = crate::date_utils::age_secs(now, gdvm_cache.last_update_check);

        if cache_age <= Self::CHECK_INTERVAL.as_secs() {
            return Ok(());
        }

        self.cache_store.save_gdvm_cache(&GdvmCache {
            last_update_check: now,
            ..gdvm_cache
        })?;

        crate::process_utils::spawn_detached(
            std::process::Command::new(std::env::current_exe()?)
                .env(Self::BACKGROUND_CHECK_ENV_VAR, "1"),
        )?;

        Ok(())
    }

    /// Run an update check.
    pub async fn run_background_check(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| terr!("error-system-time"))? // Should never fail.
            .as_secs();

        let manifest =
            match self_update::fetch_manifest(&self_update::releases_url(), Duration::from_secs(3))
                .await
            {
                Ok(manifest) => manifest,
                Err(_) => {
                    self.cache_store.clear_gdvm_cache(now)?;
                    return Ok(());
                }
            };

        // Get current version and determine major version for upgrade compatibility.
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        let new_version =
            self_update::select_upgrade(&manifest.releases, &current_version, false, false)
                .map(|r| r.normalized_version().to_string());

        let new_major_version =
            self_update::select_upgrade(&manifest.releases, &current_version, true, false)
                .map(|r| r.normalized_version().to_string())
                .filter(|major| Some(major) != new_version.as_ref());

        self.cache_store.save_gdvm_cache(&GdvmCache {
            last_update_check: now,
            new_version,
            new_major_version,
        })?;

        Ok(())
    }

    pub async fn upgrade(&self, allow_major: bool, allow_pre: bool) -> Result<()> {
        let _lock =
            crate::locks::Lock::acquire(&self.paths.locks(), crate::locks::Resource::SelfUpgrade)?;

        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        let manifest = {
            let task = ui::progress::activity(t!("status-fetching"), t!("subject-update-manifest"));
            let manifest =
                self_update::fetch_manifest(&self_update::releases_url(), Duration::from_secs(10))
                    .await?;
            drop(task);
            manifest
        };

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

        let subject = t!("upgrade-target", version = target_version.to_string());
        ui::milestone(t!("status-upgrading"), &subject);

        let triple = detect_host()?.gdvm_target_triple()?;
        let binary = target.binary_for(triple).ok_or_else(|| {
            terr!(
                "upgrade-no-binary",
                version = target_version.to_string(),
                target = triple
            )
        })?;
        let bin_url = binary.urls.first().ok_or_else(|| {
            terr!(
                "upgrade-no-binary",
                version = target_version.to_string(),
                target = triple
            )
        })?;

        let expected_sha = binary
            .sha256
            .as_deref()
            .ok_or_else(|| terr!("upgrade-checksum-required"))?;

        // Define install directory
        let install_dir = self.paths.base().join("bin");
        std::fs::create_dir_all(&install_dir)
            .map_err(|e| terr!("upgrade-install-dir-failed").with_source(e))?;

        let new_exe = install_dir.join(".gdvm-upgrade-new");
        let partial = install_dir.join(".gdvm-upgrade-new.partial");
        let partial_meta = install_dir.join(".gdvm-upgrade-new.partial.meta");

        let downloaded = crate::download_utils::download_verified(
            bin_url,
            &new_exe,
            &partial,
            &partial_meta,
            crate::download_utils::ExpectedDigests {
                sha: expected_sha,
                size: binary.size,
            },
            &subject,
        )
        .await;

        let file = match downloaded {
            Ok(file) => file,
            Err(err) => return Err(err),
        };

        #[cfg(target_family = "unix")]
        {
            // Make the new binary executable
            file.set_permissions(std::fs::Permissions::from_mode(0o755))?;
        }
        drop(file);

        // Rename current executable to .bak and replace it with the new file
        let current_exe = std::env::current_exe()?;
        let backup_exe = current_exe.with_extension("bak");

        std::fs::rename(&current_exe, &backup_exe)
            .map_err(|e| terr!("upgrade-rename-failed").with_source(e))?;

        if let Err(err) = std::fs::rename(&new_exe, &current_exe) {
            let _ = std::fs::rename(&backup_exe, &current_exe);
            return Err(terr!("upgrade-replace-failed").with_source(err).into());
        }

        // Update gdvm cache
        let last_update_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| terr!("error-system-time"))?
            .as_secs();

        self.cache_store.clear_gdvm_cache(last_update_check)?;

        ui::milestone(t!("status-upgraded"), &subject);

        Ok(())
    }
}

/// An upgrade notice.
#[derive(Debug, PartialEq, Eq)]
enum UpgradeNotice {
    /// Both a compatible and a major upgrade are available.
    Both { minor: String, major: String },
    /// A compatible upgrade is available.
    Minor(String),
    /// Only a major upgrade is available.
    Major(String),
}

/// The state of whether an upgrade notice should be shown.
#[derive(Debug, PartialEq, Eq)]
enum NoticeState {
    /// A notice should be shown.
    Notice(UpgradeNotice),
    /// The cached upgrade notice is no longer relevant to the current version.
    Stale,
    /// Nothing is cached.
    None,
}

/// Determine whether an upgrade notice should be shown based on the cached
/// upgrade information and the current version.
fn upgrade_notice(cache: &GdvmCache, current_version: &Version) -> NoticeState {
    if cache.new_version.is_none() && cache.new_major_version.is_none() {
        return NoticeState::None;
    }

    let parse = |v: &Option<String>| {
        v.as_ref()
            .and_then(|v| Version::parse(v.trim_start_matches('v')).ok())
    };
    let cached_minor = parse(&cache.new_version);
    let cached_major = parse(&cache.new_major_version);

    let valid_minor = cached_minor
        .as_ref()
        .map(|v| v > current_version)
        .unwrap_or(false);
    let valid_major = cached_major
        .as_ref()
        .map(|v| v > current_version)
        .unwrap_or(false);

    if valid_minor && valid_major && cached_minor != cached_major {
        NoticeState::Notice(UpgradeNotice::Both {
            minor: cache.new_version.clone().unwrap(),
            major: cache.new_major_version.clone().unwrap(),
        })
    } else if valid_minor {
        NoticeState::Notice(UpgradeNotice::Minor(cache.new_version.clone().unwrap()))
    } else if valid_major {
        NoticeState::Notice(UpgradeNotice::Major(
            cache.new_major_version.clone().unwrap(),
        ))
    } else {
        NoticeState::Stale
    }
}

/// Print an upgrade notice.
fn print_notice(notice: &UpgradeNotice) {
    let message = match notice {
        UpgradeNotice::Both { minor, major } => t!(
            "upgrade-available-both",
            minor_version = minor.as_str(),
            major_version = major.as_str()
        ),
        UpgradeNotice::Minor(version) => t!("upgrade-available", version = version.as_str()),
        UpgradeNotice::Major(version) => t!("upgrade-available-major", version = version.as_str()),
    };
    ui::tip(message);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache(minor: Option<&str>, major: Option<&str>) -> GdvmCache {
        GdvmCache {
            last_update_check: 0,
            new_version: minor.map(str::to_string),
            new_major_version: major.map(str::to_string),
        }
    }

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn notice_for_newer_minor() {
        assert_eq!(
            upgrade_notice(&cache(Some("1.2.0"), None), &v("1.1.0")),
            NoticeState::Notice(UpgradeNotice::Minor("1.2.0".into()))
        );
    }

    #[test]
    fn notice_for_newer_major_only() {
        assert_eq!(
            upgrade_notice(&cache(None, Some("2.0.0")), &v("1.1.0")),
            NoticeState::Notice(UpgradeNotice::Major("2.0.0".into()))
        );
    }

    #[test]
    fn notice_for_both() {
        assert_eq!(
            upgrade_notice(&cache(Some("1.2.0"), Some("2.0.0")), &v("1.1.0")),
            NoticeState::Notice(UpgradeNotice::Both {
                minor: "1.2.0".into(),
                major: "2.0.0".into(),
            })
        );
    }

    #[test]
    fn stale_when_no_longer_newer() {
        assert_eq!(
            upgrade_notice(&cache(Some("1.2.0"), None), &v("1.2.0")),
            NoticeState::Stale
        );
    }

    #[test]
    fn stale_when_unparsable() {
        assert_eq!(
            upgrade_notice(&cache(Some("not-a-version"), None), &v("1.0.0")),
            NoticeState::Stale
        );
    }

    #[test]
    fn none_when_cache_empty() {
        assert_eq!(
            upgrade_notice(&cache(None, None), &v("1.0.0")),
            NoticeState::None
        );
    }
}
