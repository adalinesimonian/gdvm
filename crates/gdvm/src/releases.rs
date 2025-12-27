use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::i18n::I18n;
use crate::metadata_cache::{
    CacheStore, RegistryReleasesCache, ReleaseCache, ReleaseCapabilitiesEntry,
    filter_cached_releases,
};
use crate::registry::{Registry, ReleaseMetadata};
use crate::t_w;
use crate::version_utils::{GodotVersion, GodotVersionDeterminate};

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

    /// List available releases, optionally filtering with a partial GodotVersion. Respects cache-
    /// only mode and refreshes the registry index when stale.
    pub fn list_releases(
        &self,
        filter: Option<&GodotVersion>,
        use_cache_only: bool,
        i18n: &I18n,
    ) -> Result<Vec<GodotVersionDeterminate>> {
        let mut cache = self.cache_store.load_registry_cache()?;
        let now = now_seconds()?;
        let cache_age = now - cache.last_fetched;
        let should_refresh = cache_age > CACHE_TTL.as_secs();

        // Drop per-release capability cache when the index is stale so it can be rebuilt on demand.
        if should_refresh {
            self.cache_store
                .clear_capabilities_cache(cache.last_fetched)?;
        }

        if should_refresh && !use_cache_only {
            if let Err(error) = self.update_cache(&mut cache, i18n) {
                if cache.releases.is_empty() {
                    return Err(error);
                } else {
                    // Defer to cached data if available, mirroring previous behavior.
                    crate::eprintln_i18n!(
                        i18n,
                        "warning-fetching-releases-using-cache",
                        error = error.to_string()
                    );
                }
            }
        }

        Ok(filter_cached_releases(&cache, filter))
    }

    /// Fetch capabilities for a given tag, caching results.
    pub fn capabilities_for(&self, tag: &str, i18n: &I18n) -> Result<ReleaseCapabilitiesEntry> {
        let mut caps_cache = self.cache_store.load_capabilities_cache()?;
        if let Some(entry) = caps_cache.entries.iter().find(|c| c.tag_name == tag) {
            return Ok(entry.clone());
        }

        let registry_cache = self.cache_store.load_registry_cache()?;
        let release = registry_cache
            .releases
            .iter()
            .find(|r| r.tag_name == tag)
            .cloned()
            .ok_or_else(|| anyhow!(t_w!(i18n, "error-version-not-found")))?;

        // Fetch metadata once, then derive capabilities.
        let metadata = self.registry.fetch_release(release.id, &release.tag_name)?;

        let capabilities = derive_capabilities(tag, &metadata);

        // Store in cache, aligning its timestamp to the registry cache so TTL matches the index.
        caps_cache.entries.retain(|c| c.tag_name != tag);
        caps_cache.entries.push(capabilities.clone());
        caps_cache.last_fetched = registry_cache.last_fetched;
        self.cache_store.save_capabilities_cache(&caps_cache)?;

        Ok(capabilities)
    }

    /// Retrieve release metadata for an exact version, refreshing the cache if needed.
    pub fn metadata_for(
        &self,
        gv: &GodotVersionDeterminate,
        i18n: &I18n,
    ) -> Result<ReleaseMetadata> {
        let tag = gv.to_remote_str();
        let mut cache = self.cache_store.load_registry_cache()?;

        if let Some(entry) = cache.releases.iter().find(|r| r.tag_name == tag) {
            return self.registry.fetch_release(entry.id, &entry.tag_name);
        }

        self.update_cache(&mut cache, i18n)?;
        if let Some(entry) = cache.releases.iter().find(|r| r.tag_name == tag) {
            return self.registry.fetch_release(entry.id, &entry.tag_name);
        }

        Err(anyhow!(t_w!(i18n, "error-version-not-found")))
    }

    pub fn refresh_cache(&self, i18n: &I18n) -> Result<()> {
        let mut cache = self.cache_store.load_registry_cache()?;
        self.update_cache(&mut cache, i18n)
    }

    pub fn cache_store(&self) -> &CacheStore {
        &self.cache_store
    }

    fn update_cache(&self, cache: &mut RegistryReleasesCache, i18n: &I18n) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message(t_w!(i18n, "fetching-releases"));

        let index = self.registry.fetch_index()?;

        pb.finish_with_message(t_w!(i18n, "releases-fetched"));

        cache.releases = index
            .into_iter()
            .map(|r| ReleaseCache {
                id: r.id,
                tag_name: r.name,
            })
            .collect();
        cache.last_fetched = now_seconds()?;

        self.cache_store.save_registry_cache(cache)?;
        // Index has been refreshed, align capability cache TTL and clear stale entries.
        self.cache_store
            .clear_capabilities_cache(cache.last_fetched)?;

        Ok(())
    }
}

fn derive_capabilities(tag: &str, metadata: &ReleaseMetadata) -> ReleaseCapabilitiesEntry {
    let mut platforms = Vec::new();
    let mut has_csharp = false;

    for (platform_key, arches) in &metadata.binaries {
        if platform_key.contains("csharp") {
            has_csharp = true;
        }
        for arch_key in arches.keys() {
            platforms.push(format!("{platform_key}-{arch_key}"));
        }
    }

    platforms.sort();
    platforms.dedup();

    ReleaseCapabilitiesEntry {
        tag_name: tag.to_string(),
        has_csharp,
        platforms,
    }
}

fn now_seconds() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
        .as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_cache::FullCache;
    use tempfile::TempDir;

    fn i18n() -> I18n {
        I18n::new(80).expect("i18n init")
    }

    fn make_catalog_with_cache(tags: &[&str], last_fetched: u64) -> (ReleaseCatalog, TempDir) {
        let tmp = TempDir::new().expect("tempdir");
        let cache_store = CacheStore::new(tmp.path().join("cache.json"));

        let releases: Vec<ReleaseCache> = tags
            .iter()
            .enumerate()
            .map(|(id, tag)| ReleaseCache {
                id: id as u64,
                tag_name: (*tag).to_string(),
            })
            .collect();

        let full = FullCache {
            gdvm: crate::metadata_cache::GdvmCache {
                last_update_check: 0,
                new_version: None,
                new_major_version: None,
            },
            godot_registry: RegistryReleasesCache {
                last_fetched,
                releases,
            },
            release_capabilities: Default::default(),
        };
        // Persist cache so ReleaseCatalog reads it.
        cache_store
            .save_registry_cache(&full.godot_registry)
            .expect("write cache");

        let registry = Registry::new().expect("registry client");
        (ReleaseCatalog::new(registry, cache_store), tmp)
    }

    #[test]
    fn uses_cached_releases_when_fresh() {
        let now = now_seconds().unwrap();
        let (catalog, _tmp) = make_catalog_with_cache(&["4.3-stable", "4.2-rc1"], now);
        let intl = i18n();

        let releases = catalog
            .list_releases(None, false, &intl)
            .expect("list releases");

        assert_eq!(releases.len(), 2);
        // Sorted newest-first. RC precedes lower version stable, stable of higher version wins.
        assert_eq!(releases[0].to_remote_str(), "4.3-stable");
        assert_eq!(releases[1].to_remote_str(), "4.2-rc1");
    }

    #[test]
    fn filters_cached_releases_with_query() {
        let now = now_seconds().unwrap();
        let (catalog, _tmp) = make_catalog_with_cache(&["4.3-stable", "4.2-rc1"], now);
        let intl = i18n();
        let filter = GodotVersion {
            major: Some(4),
            minor: Some(2),
            patch: None,
            subpatch: None,
            release_type: None,
            is_csharp: None,
        };

        let releases = catalog
            .list_releases(Some(&filter), true, &intl)
            .expect("filtered releases");

        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].to_remote_str(), "4.2-rc1");
    }
}
