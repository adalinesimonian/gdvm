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

use anyhow::Result;

use crate::artifact_cache::ArtifactCache;
use crate::config::Config;
use crate::host::{HostPlatform, detect_host};
use crate::metadata_cache::CacheStore;
#[cfg(test)]
use crate::metadata_cache::{RegistryReleasesCache, ReleaseCache, filter_cached_releases};
use crate::paths::GdvmPaths;
use crate::releases::CatalogSet;
use crate::run_version_resolver::RunVersionSource;
use crate::usage_tracker::UsageTracker;
use crate::version::{QuerySelection, ResolvedSelection, ResolvedVersion, VersionQuery};
use crate::{eprintln_i18n, post_upgrade, t, terr};

mod catalog;
mod defaults;
mod installer;
mod launcher;
mod library;
mod pruner;
mod updater;

pub use catalog::Catalogs;
pub use defaults::Defaults;
pub use installer::{InstallOutcome, Installer};
pub use launcher::{Launcher, find_godot_executable};
pub use library::{InstalledVersion, Library};
pub use pruner::{PruneOptions, PruneReport, PrunedItem, Pruner};
pub use updater::Updater;

/// App dependency container.
pub struct Gdvm {
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
    /// Host platform
    host: HostPlatform,
    /// Env vars from `.env` for passing to Godot.
    dotenv_vars: Vec<(String, String)>,
}

/// Get env vars from `.env` to pass to Godot.
fn dotenv_vars() -> Vec<(String, String)> {
    let Ok(iter) = dotenvy::dotenv_iter() else {
        return Vec::new();
    };
    iter.filter_map(|item| item.ok())
        .filter(|(key, _)| std::env::var_os(key).is_none())
        .collect()
}

/// Get registry information from the nearest gdvm.toml in the current directory or its parents.
fn project_registry_pairs() -> Vec<(String, String)> {
    let Ok(mut current) = std::env::current_dir() else {
        return Vec::new();
    };

    loop {
        let candidate = current.join("gdvm.toml");
        if candidate.is_file() {
            let parsed = fs::read_to_string(&candidate)
                .map_err(|e| e.to_string())
                .and_then(|contents| {
                    crate::gdvm_toml::deserialize_gdvm_toml(&contents).map_err(|e| e.to_string())
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
                    crate::ui::warn(t!(
                        "gdvm-toml-malformed",
                        path = path.as_str(),
                        error = error.as_str()
                    ));
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

impl Gdvm {
    /// Create a new Gdvm instance and set up the installation and cache paths.
    pub async fn new() -> Result<Self> {
        let paths = GdvmPaths::new()?;
        let artifact_cache = ArtifactCache::new(paths.cache_dir().to_path_buf());
        artifact_cache.ensure_dir()?;

        let config = Config::load().unwrap_or_default();
        let mut registries = config.registry_pairs();
        let project = project_registry_pairs();
        for conflict in registry_override_conflicts(&registries, &project) {
            crate::ui::warn(t!(
                "registry-project-override-conflict",
                registry = conflict.name.as_str(),
                machine_url = conflict.machine_url.as_str(),
                project_url = conflict.project_url.as_str(),
            ));
        }
        registries.extend(project);
        let catalogs = CatalogSet::new(paths.cache_index(), &registries)?;
        let cache_store = CacheStore::new(paths.cache_index().to_path_buf());
        let usage_tracker = UsageTracker::new(paths.usage_index().to_path_buf(), paths.locks());
        let host = detect_host()?;

        let gdvm = Gdvm {
            paths,
            artifact_cache,
            cache_store,
            usage_tracker,
            catalogs,
            host,
            dotenv_vars: dotenv_vars(),
        };

        post_upgrade::run(gdvm.paths.base())?;

        // Report any available upgrade from the last update check.
        if std::env::var_os(Updater::BACKGROUND_CHECK_ENV_VAR).is_none() {
            gdvm.updater().print_upgrade_notice().ok();
            gdvm.updater().spawn_background_check_if_due().ok();
        }

        Ok(gdvm)
    }

    /// Gets the path to gdvm's base directory
    /// (e.g. `~/.gdvm` on Unix-like systems)
    pub fn get_base_path(&self) -> &Path {
        self.paths.base()
    }

    /// Clears the release cache by deleting the cache file and all cached zip files
    pub fn clear_cache(&self) -> Result<()> {
        let cache_index = self.cache_store.index_path();
        if cache_index.exists() {
            fs::remove_file(cache_index)?;
            crate::ui::step(t!("status-removed"), t!("subject-cache-metadata"));
        } else {
            crate::ui::step(t!("status-skipped"), t!("subject-cache-metadata"));
            crate::ui::note(t!("no-cache-metadata-found"));
        }

        if self.artifact_cache.exists() {
            self.artifact_cache.clear_files()?;
            crate::ui::step(t!("status-removed"), t!("subject-cache-files"));
        } else {
            crate::ui::step(t!("status-skipped"), t!("subject-cache-files"));
            crate::ui::note(t!("no-cache-files-found"));
        }
        Ok(())
    }

    /// Release-catalog and registry queries.
    pub fn catalogs(&self) -> Catalogs<'_> {
        Catalogs {
            catalogs: &self.catalogs,
            host: &self.host,
        }
    }

    /// Installed-version inventory and store paths.
    pub fn library(&self) -> Library<'_> {
        Library {
            paths: &self.paths,
            usage_tracker: &self.usage_tracker,
            catalogs: self.catalogs(),
        }
    }

    /// Default- and pinned-version management.
    pub fn defaults(&self) -> Defaults<'_> {
        Defaults {
            paths: &self.paths,
            usage_tracker: &self.usage_tracker,
            catalogs: self.catalogs(),
        }
    }

    /// Godot installation and archive caching.
    pub fn installer(&self) -> Installer<'_> {
        Installer {
            paths: &self.paths,
            artifact_cache: &self.artifact_cache,
            usage_tracker: &self.usage_tracker,
            catalogs: self.catalogs(),
        }
    }

    /// Launching installed Godot versions.
    pub fn launcher(&self) -> Launcher<'_> {
        Launcher {
            paths: &self.paths,
            usage_tracker: &self.usage_tracker,
            catalogs: self.catalogs(),
            dotenv_vars: &self.dotenv_vars,
        }
    }

    /// Removal of unused installs and cached archives.
    pub fn pruner(&self) -> Pruner<'_> {
        Pruner {
            paths: &self.paths,
            artifact_cache: &self.artifact_cache,
            usage_tracker: &self.usage_tracker,
            catalogs: self.catalogs(),
        }
    }

    /// gdvm self-update checks and upgrades.
    pub fn updater(&self) -> Updater<'_> {
        Updater {
            paths: &self.paths,
            cache_store: &self.cache_store,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl RunVersionSource for Gdvm {
    async fn get_pinned_version(&self) -> Option<QuerySelection> {
        self.defaults().get_pinned_version()
    }

    async fn get_default(&self) -> Result<Option<ResolvedSelection>> {
        self.defaults().get_default()
    }

    async fn determine_version<P: AsRef<Path> + Send + Sync>(
        &self,
        path: Option<P>,
    ) -> Option<(VersionQuery, Option<String>)> {
        self.defaults().determine_version(path)
    }

    async fn auto_install_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
    ) -> Result<ResolvedVersion>
    where
        T: Into<VersionQuery> + Clone + Send + Sync,
    {
        self.installer()
            .auto_install_version(gv, variant, registry, include_pre)
            .await
    }

    async fn ensure_installed_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> Result<ResolvedVersion>
    where
        T: Into<VersionQuery> + Clone + Send + Sync,
    {
        let gv: VersionQuery = gv.clone().into();
        let matches = self
            .library()
            .resolve_installed_version(&gv, variant, registry)
            .await?;

        match matches.len() {
            0 => Err(terr!("error-version-not-found")),
            1 => Ok(matches[0].version.clone()),
            _ => {
                eprintln_i18n!("error-multiple-versions-found");
                for v in &matches {
                    println!(
                        "- {}",
                        crate::version::display_version(
                            &v.version,
                            &v.variant,
                            v.registry.as_deref(),
                        )
                    );
                }
                Err(terr!("error-version-not-found"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let filter = VersionQuery::from_match_str("4.1.1-stable").unwrap();
        let releases = filter_cached_releases(&cache, Some(&filter));
        let tags: Vec<String> = releases.into_iter().map(|r| r.to_remote_str()).collect();

        assert_eq!(tags, vec!["4.1.1-stable"]);
    }
}
