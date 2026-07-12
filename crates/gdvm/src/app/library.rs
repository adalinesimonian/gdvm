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

use crate::paths::GdvmPaths;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

use crate::t;
use crate::usage_tracker::UsageTracker;
use crate::version::Variant;
use crate::version::VersionQuery;

use crate::version::ResolvedVersion;

use super::*;

#[derive(Debug, Clone)]
pub struct InstalledVersion {
    pub version: ResolvedVersion,
    pub variant: Variant,
    pub registry: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Library<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) usage_tracker: &'a UsageTracker,
    pub(super) catalogs: Catalogs<'a>,
}

impl<'a> Library<'a> {
    fn catalogs(&self) -> Catalogs<'a> {
        self.catalogs
    }

    fn defaults(&self) -> Defaults<'a> {
        Defaults {
            paths: self.paths,
            usage_tracker: self.usage_tracker,
            catalogs: self.catalogs,
        }
    }

    /// A unique key for an install path.
    pub fn install_key(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<String> {
        Ok(crate::version::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        ))
    }

    /// The install store directory name for a registry.
    pub(super) fn install_store_key(&self, registry: Option<&str>) -> Result<String> {
        let base_url = self.catalogs().catalog(registry)?.registry_base_url();
        Ok(crate::registry::store_dir_name(&base_url))
    }

    /// Record that a link was created at `link_path` pointing into the install
    /// identified by `install_key`.
    pub fn record_link(&self, link_path: &Path, install_key: &str) -> Result<()> {
        self.usage_tracker.record_link(link_path, install_key)
    }

    /// Record that an install was run or referenced.
    pub(super) fn track_install_use(&self, install_key: &str) -> Result<()> {
        self.usage_tracker.record_install(install_key)
    }

    /// The label to display for a registry.
    pub(super) fn display_registry_for_url(
        &self,
        url: &str,
        display_name: Option<&str>,
    ) -> Option<String> {
        let normalized = crate::registry::normalize_url(url);
        if normalized == crate::registry::normalize_url(crate::registry::OFFICIAL_BASE_URL) {
            return None;
        }
        for name in self.catalogs.catalogs.names() {
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
            let mut v = [a.version.clone(), b.version.clone()];
            v.sort_by(|a, b| b.cmp(a));
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
                        VersionQuery::from_install_str(&leaf.file_name().to_string_lossy())
                {
                    out.push(InstalledVersion {
                        version: gv.to_resolved(),
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

            if let Ok(gv) = VersionQuery::from_install_str(&sub_name) {
                out.push(InstalledVersion {
                    version: gv.to_resolved(),
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
                            VersionQuery::from_install_str(&leaf.file_name().to_string_lossy())
                    {
                        out.push(InstalledVersion {
                            version: gv.to_resolved(),
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
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
    ) -> Result<()> {
        let install_name = crate::version::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        );

        let _lock = crate::locks::Lock::acquire(
            &self.paths.locks(),
            crate::locks::Resource::Install(&install_name),
        )?;

        self.remove_locked(gv, variant, registry, &install_name)
    }

    /// Inner removal function that does not lock. Only use when a lock was
    /// already acquired.
    pub(super) fn remove_locked(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
        install_name: &str,
    ) -> Result<()> {
        let path = self.paths.installs().join(install_name);

        if path.exists() {
            // If this version is the default, unset it
            if let Some(def) = self.defaults().get_default()?
                && def.version.to_remote_str() == gv.to_remote_str()
                && def.variant == *variant
                && crate::registry::normalize_registry(def.registry.as_deref())
                    == crate::registry::normalize_registry(registry)
            {
                self.defaults().unset_default()?;
            }
            fs::remove_dir_all(path)?;
            self.usage_tracker.forget_install(install_name)?;
            Ok(())
        } else {
            Err(anyhow!(t!("error-version-not-found")))
        }
    }

    /// Resolve the path to the Godot executable for the given version and console preference.
    pub fn get_executable_path(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
        console: bool,
    ) -> Result<std::path::PathBuf> {
        let install_name = crate::version::install_dir_subpath(
            &self.install_store_key(registry)?,
            &gv.to_remote_str(),
            variant,
        );
        let version_dir = self.paths.installs().join(&install_name);
        if !version_dir.exists() {
            return Err(anyhow!(t!(
                "error-version-not-found",
                version = &gv.to_display_str(),
            )));
        }

        let godot_executable = find_godot_executable(&version_dir, console)?;

        let godot_executable = godot_executable.ok_or_else(|| {
            anyhow!(t!(
                "godot-executable-not-found",
                version = &gv.to_display_str(),
            ))
        })?;

        self.track_install_use(&install_name)?;

        Ok(godot_executable)
    }

    pub(super) fn is_version_installed<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> Result<bool>
    where
        T: Into<VersionQuery> + Clone,
    {
        let gv: VersionQuery = gv.clone().into();

        let installed_versions = self.list_installed()?;
        let target_variant = Variant::from_option(variant);
        Ok(installed_versions.iter().any(|v| {
            v.variant == target_variant
                && crate::registry::normalize_registry(v.registry.as_deref())
                    == crate::registry::normalize_registry(registry)
                && gv.matches(&v.version)
        }))
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
        T: Into<VersionQuery> + Clone,
    {
        let gv: VersionQuery = gv.clone().into();
        let installed = self.list_installed()?;
        // Filter by registry and variant, then filter by version match.
        let target_variant = Variant::from_option(variant);
        let matches: Vec<_> = installed
            .into_iter()
            .filter(|v| {
                v.variant == target_variant
                    && crate::registry::normalize_registry(v.registry.as_deref())
                        == crate::registry::normalize_registry(registry)
                    && gv.matches(&v.version)
            })
            .collect();
        Ok(matches)
    }
}
