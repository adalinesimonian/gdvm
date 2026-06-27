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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::date_utils::now_iso8601;

/// File name of the per-store sidecar.
pub const STORE_META_FILE: &str = ".gdvm-registry.toml";

/// Current schema version.
pub const STORE_META_SCHEMA_VERSION: u32 = 1;

const HEADER: &str = "# Managed by gdvm. Identifies the registry this install store belongs to.\n";

/// Parsed `.gdvm-registry.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStoreMeta {
    /// Schema version.
    pub schema: u32,
    /// Normalized registry URL.
    pub url: String,
    /// The most recent alias used to refer to this registry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_alias: Option<String>,
    /// The registry's display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Preserve keys written by newer gdvm versions.
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

impl RegistryStoreMeta {
    fn new(url: String) -> Self {
        Self {
            schema: STORE_META_SCHEMA_VERSION,
            url,
            last_alias: None,
            display_name: None,
            created_at: None,
            updated_at: None,
            extra: BTreeMap::new(),
        }
    }
}

fn meta_path(store_dir: &Path) -> std::path::PathBuf {
    store_dir.join(STORE_META_FILE)
}

/// Read the sidecar from a store directory, if present.
pub fn read(store_dir: &Path) -> Result<Option<RegistryStoreMeta>> {
    let path = meta_path(store_dir);
    if !path.is_file() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)?;
    Ok(Some(toml::from_str(&contents)?))
}

/// Write the sidecar into a store directory.
pub fn write(store_dir: &Path, meta: &RegistryStoreMeta) -> Result<()> {
    std::fs::create_dir_all(store_dir)?;
    let body = toml::to_string(meta)?;
    std::fs::write(meta_path(store_dir), format!("{HEADER}{body}"))?;
    Ok(())
}

/// Create or refresh the sidecar for a store.
pub fn upsert(
    store_dir: &Path,
    url: &str,
    last_alias: Option<&str>,
    display_name: Option<&str>,
) -> Result<()> {
    let now = now_iso8601();
    let mut meta = read(store_dir)?.unwrap_or_else(|| RegistryStoreMeta::new(url.to_string()));

    meta.schema = STORE_META_SCHEMA_VERSION;
    meta.url = url.to_string();
    if meta.created_at.is_none() {
        meta.created_at = Some(now.clone());
    }
    meta.updated_at = Some(now);
    if last_alias.is_some() {
        meta.last_alias = last_alias.map(str::to_string);
    }
    if display_name.is_some() {
        meta.display_name = display_name.map(str::to_string);
    }

    write(store_dir, &meta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn upsert_then_read_roundtrips_and_preserves_created_at() {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join("store");

        upsert(
            &store,
            "https://builds.example.com/godot",
            Some("mybuilds"),
            Some("Example Builds"),
        )
        .unwrap();
        let first = read(&store).unwrap().unwrap();
        assert_eq!(first.schema, STORE_META_SCHEMA_VERSION);
        assert_eq!(first.url, "https://builds.example.com/godot");
        assert_eq!(first.last_alias.as_deref(), Some("mybuilds"));
        assert_eq!(first.display_name.as_deref(), Some("Example Builds"));
        assert!(first.created_at.is_some());

        upsert(
            &store,
            "https://builds.example.com/godot",
            Some("other"),
            None,
        )
        .unwrap();
        let second = read(&store).unwrap().unwrap();
        assert_eq!(second.created_at, first.created_at);
        assert_eq!(second.last_alias.as_deref(), Some("other"));
        assert_eq!(second.display_name.as_deref(), Some("Example Builds"));
    }

    #[test]
    fn unknown_keys_survive_a_rewrite() {
        let tmp = TempDir::new().unwrap();
        let store = tmp.path().join("store");
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(
            meta_path(&store),
            "schema = 1\nurl = \"https://x.test/r\"\nfuture_key = \"keep me\"\n",
        )
        .unwrap();

        upsert(&store, "https://x.test/r", None, None).unwrap();

        let contents = std::fs::read_to_string(meta_path(&store)).unwrap();
        assert!(
            contents.contains("future_key"),
            "unknown key must be preserved, got: {contents}"
        );
    }

    #[test]
    fn read_missing_is_none() {
        let tmp = TempDir::new().unwrap();
        assert!(read(tmp.path()).unwrap().is_none());
    }
}
