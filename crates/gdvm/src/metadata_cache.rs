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

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::registry::ReleaseRef;
use crate::version_utils::{GodotVersion, GodotVersionDeterminate, GodotVersionDeterminateVecExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Metadata for a single Godot release stored in the registry cache.
pub struct ReleaseCache {
    pub tag_name: String,
    /// Map of variant names to platform keys, e.g. `linux-x86_64`.
    #[serde(default)]
    pub variants: Option<HashMap<String, Vec<String>>>,
    /// How to address this release's download metadata file.
    pub source: ReleaseRef,
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
    #[serde(default)]
    pub registries: HashMap<String, RegistryReleasesCache>,
}

impl Default for FullCache {
    fn default() -> Self {
        Self {
            gdvm: GdvmCache {
                last_update_check: 0,
                new_version: None,
                new_major_version: None,
            },
            registries: HashMap::new(),
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

    pub fn load_registry_cache(&self, registry: &str) -> Result<RegistryReleasesCache> {
        let full = self.load_full_cache()?;
        Ok(full
            .registries
            .get(registry)
            .cloned()
            .unwrap_or_else(|| RegistryReleasesCache {
                last_fetched: 0,
                releases: vec![],
            }))
    }

    pub fn save_registry_cache(&self, registry: &str, cache: &RegistryReleasesCache) -> Result<()> {
        self.update_full_cache(|full| {
            full.registries.insert(registry.to_string(), cache.clone());
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

    pub fn clear_gdvm_cache(&self, last_update_check: u64) -> Result<()> {
        self.save_gdvm_cache(&GdvmCache {
            last_update_check,
            new_version: None,
            new_major_version: None,
        })
    }
}

/// Write data to a file atomically by writing to a temp file in the same directory
/// and renaming it into place. Ensures the parent directory exists before writing.
fn atomic_write(path: &Path, data: String) -> Result<()> {
    let parent = path.parent().ok_or_else(|| anyhow!("Invalid cache path"))?;

    fs::create_dir_all(parent)?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(data.as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;

    Ok(())
}

pub fn filter_cached_releases(
    cache: &RegistryReleasesCache,
    filter: Option<&GodotVersion>,
) -> Vec<GodotVersionDeterminate> {
    let mut releases: Vec<GodotVersionDeterminate> = cache
        .releases
        .iter()
        .filter_map(|r| GodotVersion::from_remote_str(&r.tag_name).ok())
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
