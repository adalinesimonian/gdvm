use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::version_utils::{GodotVersion, GodotVersionDeterminate, GodotVersionDeterminateVecExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Metadata for a single Godot release stored in the registry cache.
pub struct ReleaseCache {
    pub id: u64,
    pub tag_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Cached list of Godot releases fetched from the remote registry.
pub struct RegistryReleasesCache {
    /// Unix timestamp in seconds
    pub last_fetched: u64,
    pub releases: Vec<ReleaseCache>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Metadata about GDVM itself.
pub struct GdvmCache {
    pub last_update_check: u64,
    pub new_version: Option<String>,
    pub new_major_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Aggregates all cached metadata into a single JSON file on disk.
pub struct FullCache {
    /// Cache for GDVM metadata
    pub gdvm: GdvmCache,
    /// Cache for Godot releases
    pub godot_registry: RegistryReleasesCache,
}

impl Default for FullCache {
    fn default() -> Self {
        Self {
            gdvm: GdvmCache {
                last_update_check: 0,
                new_version: None,
                new_major_version: None,
            },
            godot_registry: RegistryReleasesCache {
                last_fetched: 0,
                releases: vec![],
            },
        }
    }
}

/// Helper for loading and updating the JSON metadata cache on disk.
pub struct CacheStore {
    index_path: PathBuf,
}

impl CacheStore {
    pub fn new(index_path: PathBuf) -> Self {
        Self { index_path }
    }

    pub fn index_path(&self) -> &Path {
        &self.index_path
    }

    fn load_full_cache(&self) -> Result<FullCache> {
        if self.index_path.exists() {
            let data = fs::read_to_string(&self.index_path)?;
            match serde_json::from_str::<FullCache>(&data) {
                Ok(full) => Ok(full),
                Err(_) => {
                    let empty_full = FullCache::default();
                    self.save_full_cache(&empty_full)?;
                    Ok(empty_full)
                }
            }
        } else {
            Ok(FullCache::default())
        }
    }

    fn save_full_cache(&self, full: &FullCache) -> Result<()> {
        let data = serde_json::to_string(full)?;
        atomic_write(&self.index_path, data)
    }

    fn update_full_cache<F>(&self, update: F) -> Result<()>
    where
        F: FnOnce(&mut FullCache),
    {
        let mut full = self.load_full_cache()?;
        update(&mut full);
        self.save_full_cache(&full)
    }

    pub fn load_registry_cache(&self) -> Result<RegistryReleasesCache> {
        let full = self.load_full_cache()?;
        Ok(full.godot_registry)
    }

    pub fn save_registry_cache(&self, cache: &RegistryReleasesCache) -> Result<()> {
        self.update_full_cache(|full| {
            full.godot_registry = cache.clone();
        })
    }

    pub fn load_gdvm_cache(&self) -> Result<GdvmCache> {
        let full = self.load_full_cache()?;
        Ok(full.gdvm)
    }

    pub fn save_gdvm_cache(&self, cache: &GdvmCache) -> Result<()> {
        self.update_full_cache(|full| {
            full.gdvm = cache.clone();
        })
    }
}

/// Write data to a file atomically by writing to a temp file in the same directory
/// and renaming it into place. Ensures the parent directory exists before writing.
fn atomic_write(path: &Path, data: String) -> Result<()> {
    let parent = path.parent().ok_or_else(|| anyhow!("Invalid cache path"))?;

    fs::create_dir_all(parent)?;

    let tmp_path = path.with_extension("tmp");
    {
        let mut tmp = fs::File::create(&tmp_path)?;
        tmp.write_all(data.as_bytes())?;
        tmp.sync_all()?;
    }

    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(path)?;
            fs::rename(&tmp_path, path)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn filter_cached_releases(
    cache: &RegistryReleasesCache,
    filter: Option<&GodotVersion>,
) -> Vec<GodotVersionDeterminate> {
    let mut releases: Vec<GodotVersionDeterminate> = cache
        .releases
        .iter()
        .filter_map(|r| GodotVersion::from_remote_str(&r.tag_name, None).ok())
        .map(|gv| gv.to_determinate())
        .filter(|r| filter.is_none_or(|f| f.matches(r)))
        .collect();

    releases.sort_by_version();

    releases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_parent_and_overwrites() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("nested").join("cache.json");

        atomic_write(&path, "first".to_string())?;
        assert_eq!(fs::read_to_string(&path)?, "first");

        atomic_write(&path, "second".to_string())?;
        assert_eq!(fs::read_to_string(&path)?, "second");

        Ok(())
    }
}
