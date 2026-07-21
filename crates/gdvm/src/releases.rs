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

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::metadata_cache::{
    CacheStore, RegistryReleasesCache, ReleaseCache, filter_cached_releases,
};
use crate::registry::{OFFICIAL_REGISTRY, Registry, ReleaseMetadata};
use crate::version::{ResolvedVersion, VersionQuery};
use crate::{t, terr};

const CACHE_TTL: Duration = Duration::from_secs(48 * 3600); // 48 hours

/// Provides a single API for release discovery and metadata lookup.
pub struct ReleaseCatalog {
    registry: Registry,
    cache_store: CacheStore,
}

impl ReleaseCatalog {
    pub fn new(registry: Registry, cache_store: CacheStore) -> Self {
        Self {
            registry,
            cache_store,
        }
    }

    /// The name of the registry this catalog is scoped to.
    pub fn registry_name(&self) -> &str {
        self.registry.name()
    }

    /// A display string for the registry's base URL.
    pub fn registry_base_url(&self) -> String {
        self.registry.base_url_display()
    }

    /// List available releases, optionally filtering with a partial VersionQuery. Respects cache-
    /// only mode and refreshes the registry index when stale.
    pub async fn list_releases(
        &self,
        filter: Option<&VersionQuery>,
        use_cache_only: bool,
    ) -> Result<Vec<ResolvedVersion>> {
        let registry = self.registry.cache_key();
        let mut cache = self.cache_store.load_registry_cache(&registry)?;
        let now = now_seconds()?;
        let cache_age = crate::date_utils::age_secs(now, cache.last_fetched);
        let should_refresh = cache_age > CACHE_TTL.as_secs();

        if should_refresh
            && !use_cache_only
            && let Err(error) = self.update_cache(&mut cache).await
        {
            if cache.releases.is_empty() {
                return Err(error);
            }

            // Defer to cached data if available.
            crate::ui::warn(crate::t!(
                "warning-fetching-releases-using-cache",
                error = error.to_string()
            ));
        }

        Ok(filter_cached_releases(&cache, filter))
    }

    /// Get the platforms available for a release's variants.
    pub async fn platforms_by_variant(&self, tag: &str) -> Result<HashMap<String, Vec<String>>> {
        let key = self.registry.cache_key();
        let mut cache = self.cache_store.load_registry_cache(&key)?;

        let release = cache
            .releases
            .iter()
            .find(|r| r.tag_name == tag)
            .cloned()
            .ok_or_else(|| terr!("error-version-not-found"))?;

        if let Some(variants) = release.variants {
            return Ok(variants);
        }

        let metadata = self.registry.fetch_release(&release.source).await?;
        let variants = derive_variants(&metadata);

        if let Some(entry) = cache.releases.iter_mut().find(|r| r.tag_name == tag) {
            entry.variants = Some(variants.clone());
            self.cache_store.save_registry_cache(&key, &cache)?;
        }

        Ok(variants)
    }

    /// Retrieve release metadata for an exact version, refreshing the cache if needed.
    pub async fn metadata_for(&self, gv: &ResolvedVersion) -> Result<ReleaseMetadata> {
        let tag = gv.to_remote_str();
        let mut cache = self
            .cache_store
            .load_registry_cache(&self.registry.cache_key())?;

        if let Some(entry) = cache.releases.iter().find(|r| r.tag_name == tag) {
            return self.registry.fetch_release(&entry.source).await;
        }

        self.update_cache(&mut cache).await?;
        if let Some(entry) = cache.releases.iter().find(|r| r.tag_name == tag) {
            return self.registry.fetch_release(&entry.source).await;
        }

        Err(terr!("error-version-not-found").into())
    }

    pub async fn refresh_cache(&self) -> Result<()> {
        let mut cache = self
            .cache_store
            .load_registry_cache(&self.registry.cache_key())?;
        self.update_cache(&mut cache).await
    }

    pub fn cache_store(&self) -> &CacheStore {
        &self.cache_store
    }

    async fn update_cache(&self, cache: &mut RegistryReleasesCache) -> Result<()> {
        let task = crate::ui::progress::activity(t!("status-fetching"), t!("subject-releases"));

        let index = self.registry.fetch_index().await?;

        drop(task);

        cache.releases = index
            .into_iter()
            .map(|r| ReleaseCache {
                tag_name: r.version,
                variants: r.variants,
                source: r.source,
            })
            .collect();
        cache.last_fetched = now_seconds()?;

        self.cache_store
            .save_registry_cache(&self.registry.cache_key(), cache)?;

        Ok(())
    }
}

/// Holds a `ReleaseCatalog` per registry.
pub struct CatalogSet {
    catalogs: HashMap<String, ReleaseCatalog>,
}

/// Summary information about a configured registry.
#[derive(Debug, Clone)]
pub struct RegistryInfo {
    pub name: String,
    pub url: String,
    pub is_official: bool,
}

impl CatalogSet {
    /// Build a catalog set from configured registries.
    pub fn new(cache_index: &Path, registries: &[(String, String)]) -> Result<Self> {
        let mut catalogs = HashMap::new();

        catalogs.insert(
            OFFICIAL_REGISTRY.to_string(),
            ReleaseCatalog::new(
                Registry::official()?,
                CacheStore::new(cache_index.to_path_buf()),
            ),
        );

        for (name, url) in registries {
            if name == OFFICIAL_REGISTRY {
                // The official registry cannot be redefined.
                continue;
            }

            let registry = Registry::new(name, url)?;

            catalogs.insert(
                name.clone(),
                ReleaseCatalog::new(registry, CacheStore::new(cache_index.to_path_buf())),
            );
        }

        Ok(Self { catalogs })
    }

    /// True when registry is the official registry.
    pub fn is_official(&self, registry: Option<&str>) -> bool {
        registry.unwrap_or(OFFICIAL_REGISTRY) == OFFICIAL_REGISTRY
    }

    /// Select a catalog by registry name. Falls back to the official registry.
    pub fn catalog(&self, registry: Option<&str>) -> Result<&ReleaseCatalog> {
        let name = registry.unwrap_or(OFFICIAL_REGISTRY);
        self.catalogs
            .get(name)
            .ok_or_else(|| terr!("error-registry-unknown", name = name).into())
    }

    /// The official registry's release catalog.
    pub fn official(&self) -> &ReleaseCatalog {
        self.catalogs
            .get(OFFICIAL_REGISTRY)
            .expect("official registry catalog is always present")
    }

    /// All configured registry names. The official one comes first.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .catalogs
            .keys()
            .map(|s| s.as_str())
            .filter(|n| *n != OFFICIAL_REGISTRY)
            .collect();
        names.sort_unstable();
        names.insert(0, OFFICIAL_REGISTRY);
        names
    }

    /// Summaries of all configured registries. The official one comes first.
    pub fn list(&self) -> Vec<RegistryInfo> {
        self.names()
            .into_iter()
            .map(|name| RegistryInfo {
                name: name.to_string(),
                url: self
                    .catalogs
                    .get(name)
                    .map(|c| c.registry_base_url())
                    .unwrap_or_default(),
                is_official: name == OFFICIAL_REGISTRY,
            })
            .collect()
    }
}

/// Get the variant to platform map from a release's metadata.
fn derive_variants(metadata: &ReleaseMetadata) -> HashMap<String, Vec<String>> {
    let mut variants: HashMap<String, Vec<String>> = HashMap::new();

    for (variant, platforms) in &metadata.variants {
        let mut keys: Vec<String> = platforms.keys().cloned().collect();
        keys.sort();
        variants.insert(variant.clone(), keys);
    }

    variants
}

fn now_seconds() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| terr!("error-system-time"))?
        .as_secs())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn make_catalog_with_cache(tags: &[&str], last_fetched: u64) -> (ReleaseCatalog, TempDir) {
        let tmp = TempDir::new().expect("tempdir");
        let cache_store = CacheStore::new(tmp.path().join("cache.json"));

        let releases: Vec<ReleaseCache> = tags
            .iter()
            .map(|tag| ReleaseCache {
                tag_name: (*tag).to_string(),
                variants: None,
                source: crate::registry::ReleaseRef::V2 {
                    path: format!("releases/{tag}.json"),
                },
            })
            .collect();

        let cache = RegistryReleasesCache {
            last_fetched,
            releases,
        };
        let registry = Registry::official().expect("registry client");
        // Persist cache so ReleaseCatalog reads it.
        cache_store
            .save_registry_cache(&registry.cache_key(), &cache)
            .expect("write cache");

        (ReleaseCatalog::new(registry, cache_store), tmp)
    }

    #[tokio::test]
    async fn uses_cached_releases_when_fresh() {
        let now = now_seconds().unwrap();
        let (catalog, _tmp) = make_catalog_with_cache(&["4.3-stable", "4.2-rc1"], now);

        let releases = catalog
            .list_releases(None, false)
            .await
            .expect("list releases");

        assert_eq!(releases.len(), 2);
        // Sorted newest-first. RC precedes lower version stable, stable of higher version wins.
        assert_eq!(releases[0].to_remote_str(), "4.3-stable");
        assert_eq!(releases[1].to_remote_str(), "4.2-rc1");
    }

    #[tokio::test]
    async fn filters_cached_releases_with_query() {
        let now = now_seconds().unwrap();
        let (catalog, _tmp) = make_catalog_with_cache(&["4.3-stable", "4.2-rc1"], now);
        let filter = VersionQuery {
            major: Some(4),
            minor: Some(2),
            patch: None,
            subpatch: None,
            release_type: None,
        };

        let releases = catalog
            .list_releases(Some(&filter), true)
            .await
            .expect("filtered releases");

        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].to_remote_str(), "4.2-rc1");
    }

    fn cache_path() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("cache.json");
        (tmp, path)
    }

    #[test]
    fn catalog_set_defaults_to_official() {
        let (_tmp, cache) = cache_path();
        let set = CatalogSet::new(&cache, &[]).expect("catalog set");

        assert!(set.is_official(None));
        assert_eq!(set.catalog(None).unwrap().registry_name(), "official");
        assert_eq!(set.names(), vec!["official"]);
    }

    #[test]
    fn catalog_set_routes_by_name() {
        let (_tmp, cache) = cache_path();
        let registries = vec![
            (
                "mybuilds".to_string(),
                "https://example.com/godot".to_string(),
            ),
            ("local".to_string(), "file:///tmp/reg".to_string()),
        ];
        let set = CatalogSet::new(&cache, &registries).expect("catalog set");

        assert!(set.is_official(None));
        assert!(set.is_official(Some("official")));
        assert!(!set.is_official(Some("mybuilds")));
        assert_eq!(set.catalog(None).unwrap().registry_name(), "official");
        assert_eq!(
            set.catalog(Some("mybuilds")).unwrap().registry_name(),
            "mybuilds"
        );
        assert_eq!(set.catalog(Some("local")).unwrap().registry_name(), "local");
        assert!(set.catalog(Some("missing")).is_err());
        assert_eq!(set.names()[0], "official");
    }

    #[test]
    fn catalog_set_cannot_redefine_official() {
        let (_tmp, cache) = cache_path();
        let registries = vec![(
            "official".to_string(),
            "https://evil.example.com".to_string(),
        )];
        let set = CatalogSet::new(&cache, &registries).expect("catalog set");

        assert_eq!(set.names(), vec!["official"]);
        assert_ne!(
            set.catalog(None).unwrap().registry_base_url(),
            "https://evil.example.com"
        );
    }

    #[test]
    fn catalog_set_later_registry_wins_for_duplicate_alias() {
        let (_tmp, cache) = cache_path();
        let registries = vec![
            ("a".to_string(), "https://machine.example.com".to_string()),
            ("a".to_string(), "https://project.example.com".to_string()),
        ];
        let set = CatalogSet::new(&cache, &registries).expect("catalog set");

        assert_eq!(
            set.catalog(Some("a")).unwrap().registry_base_url(),
            "https://project.example.com"
        );
    }
}
