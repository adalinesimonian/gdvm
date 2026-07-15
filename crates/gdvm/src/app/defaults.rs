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
use std::path::Path;

use anyhow::{Result, anyhow};

use super::*;
use crate::paths::GdvmPaths;
use crate::usage_tracker::UsageTracker;
use crate::version::{QuerySelection, ResolvedSelection, ResolvedVersion, Variant, VersionQuery};
use crate::{project_version_detector, t};

#[derive(Clone, Copy)]
pub struct Defaults<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) usage_tracker: &'a UsageTracker,
    pub(super) catalogs: Catalogs<'a>,
}

impl<'a> Defaults<'a> {
    fn library(&self) -> Library<'a> {
        Library {
            paths: self.paths,
            usage_tracker: self.usage_tracker,
            catalogs: self.catalogs,
        }
    }

    pub fn set_default(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<()> {
        let _lock =
            crate::locks::Lock::acquire(&self.paths.locks(), crate::locks::Resource::Defaults)?;
        // Check if the version exists
        let install_name = crate::version::install_dir_subpath(
            &self.library().install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        );
        let version_path = self.paths.installs().join(&install_name);
        if !version_path.exists() {
            return Err(anyhow!(t!("error-version-not-found")));
        }

        self.library().track_install_use(&install_name)?;

        // Write pinned-format string to .gdvm/default
        let default_path = self.paths.default_file();
        let default_str = crate::version::pinned_str(registry, &gv.to_pinned_str(), variant);
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
                return Err(anyhow!(t!("error-create-symlink-windows")));
            }
            return Err(anyhow!(e));
        }

        Ok(())
    }

    pub fn unset_default(&self) -> Result<()> {
        let _lock =
            crate::locks::Lock::acquire(&self.paths.locks(), crate::locks::Resource::Defaults)?;
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

    pub fn get_default(&self) -> Result<Option<ResolvedSelection>> {
        let default_file = self.paths.default_file();
        if default_file.exists() {
            let contents = fs::read_to_string(&default_file)?;
            let (registry, variant, version_str) =
                crate::version::parse_pinned_str(contents.trim());
            let version = VersionQuery::from_install_str(&version_str)?.to_resolved();
            Ok(Some(ResolvedSelection {
                version,
                variant: Variant::from_option(variant.as_deref()),
                registry,
            }))
        } else {
            Ok(None)
        }
    }

    /// Recursively search upward for gdvm.toml, return the pinned version if found.
    pub fn get_pinned_version(&self) -> Option<QuerySelection> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let toml_candidate = current.join("gdvm.toml");
            if toml_candidate.is_file()
                && let Ok(contents) = fs::read_to_string(&toml_candidate)
                && let Ok(gdvm_toml) = crate::gdvm_toml::deserialize_gdvm_toml(&contents)
                && let Some(godot) = gdvm_toml.godot
            {
                let (registry, variant, version_str) =
                    crate::version::parse_pinned_str(godot.version.trim());
                if let Ok(version) = VersionQuery::from_install_str(&version_str) {
                    return Some(QuerySelection {
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
                    crate::version::parse_pinned_str(contents.trim());
                if let Ok(version) = VersionQuery::from_install_str(&version_str) {
                    return Some(QuerySelection {
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
    ) -> Option<(VersionQuery, Option<String>)> {
        let current_dir = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => std::env::current_dir().ok()?,
        };

        project_version_detector::detect_godot_version_in_path(&current_dir)
    }

    /// Pin a version to gdvm.toml in the current directory.
    pub fn pin_version(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
        gdvm_toml_only: bool,
    ) -> Result<()> {
        let path = std::env::current_dir()?;

        let specifier = crate::version::pinned_str(registry, &gv.to_pinned_str(), variant);
        let toml_content = crate::gdvm_toml::serialize_gdvm_toml(&specifier);
        crate::fs_utils::atomic_write(&path.join("gdvm.toml"), &toml_content)?;

        // Write deprecated .gdvmrc for backward compatibility with older versions of gdvm.
        // The legacy format predates registries, so we skip writing it for builds from custom
        // registries.
        if !gdvm_toml_only && crate::registry::is_official_registry(registry) {
            let legacy = crate::version::legacy_pinned_str(gv, variant);
            fs::write(path.join(".gdvmrc"), legacy)?;
        }

        Ok(())
    }
}
