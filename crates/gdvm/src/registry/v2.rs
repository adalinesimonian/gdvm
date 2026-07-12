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

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{RegistryUrl, ReleaseRef};

/// Parsed `registry.json`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub schema: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Parsed `index.json`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Index {
    pub schema: u32,
    pub releases: Vec<IndexRelease>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexRelease {
    pub version: String,
    /// Variant name -> platform keys. Sorted for deterministic, diff-friendly output.
    pub variants: BTreeMap<String, Vec<String>>,
    pub path: String,
}

/// Parsed `releases/<version>.json`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaseMetadata {
    pub schema: u32,
    /// ISO 8601 timestamp of the last change to this release file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub version: String,
    /// Variant -> platform key -> binary. Sorted for deterministic output.
    pub variants: BTreeMap<String, BTreeMap<String, BinaryInfo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BinaryInfo {
    pub sha512: String,
    /// Archive size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    pub urls: Vec<String>,
}

/// Normalize a v2 index into index entries.
pub fn normalize_index(index: Index) -> Vec<super::IndexEntry> {
    index
        .releases
        .into_iter()
        .map(|r| super::IndexEntry {
            version: r.version,
            variants: Some(r.variants.into_iter().collect()),
            source: ReleaseRef::V2 { path: r.path },
        })
        .collect()
}

/// Normalize a v2 release into release metadata.
pub fn normalize_release(meta: ReleaseMetadata, base: &RegistryUrl) -> super::ReleaseMetadata {
    let variants = meta
        .variants
        .into_iter()
        .map(|(variant, platforms)| {
            let platforms = platforms
                .into_iter()
                .map(|(platform, bin)| {
                    (
                        platform,
                        super::BinaryInfo {
                            sha512: bin.sha512,
                            size: bin.size,
                            urls: bin.urls.into_iter().map(|u| base.join(&u)).collect(),
                        },
                    )
                })
                .collect();
            (variant, platforms)
        })
        .collect();

    super::ReleaseMetadata {
        version: meta.version,
        variants,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX_JSON: &str = r#"{
        "schema": 2,
        "releases": [
            {
                "version": "4.4-stable",
                "variants": {
                    "default": ["linux-x86_64", "macos-universal"],
                    "csharp": ["linux-x86_64"]
                },
                "path": "releases/4.4-stable.json",
                "unknown_future_key": true
            }
        ]
    }"#;

    const RELEASE_JSON: &str = r#"{
        "schema": 2,
        "version": "4.4-stable",
        "variants": {
            "default": {
                "linux-x86_64": {
                    "sha512": "aa",
                    "size": 100,
                    "urls": ["binaries/4.4-stable/linux-x86_64.zip"]
                },
                "macos-universal": {
                    "sha512": "bb",
                    "size": 200,
                    "urls": ["https://cdn.example.com/macos.zip"]
                }
            }
        }
    }"#;

    #[test]
    fn parses_and_normalizes_index_ignoring_unknown_keys() {
        let index: Index = serde_json::from_str(INDEX_JSON).unwrap();
        let entries = normalize_index(index);
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.version, "4.4-stable");
        let variants = entry.variants.as_ref().unwrap();
        assert!(variants.contains_key("default"));
        assert!(variants.contains_key("csharp"));
        assert_eq!(
            entry.source,
            ReleaseRef::V2 {
                path: "releases/4.4-stable.json".to_string()
            }
        );
    }

    #[test]
    fn parses_and_normalizes_release_resolving_urls() {
        let meta: ReleaseMetadata = serde_json::from_str(RELEASE_JSON).unwrap();
        let base = RegistryUrl::parse("https://example.com/reg").unwrap();
        let normalized = normalize_release(meta, &base);

        let default = normalized.variants.get("default").unwrap();
        let linux = default.get("linux-x86_64").unwrap();
        assert_eq!(linux.size, Some(100));
        assert_eq!(
            linux.urls[0],
            "https://example.com/reg/binaries/4.4-stable/linux-x86_64.zip"
        );

        let macos = default.get("macos-universal").unwrap();
        assert_eq!(macos.urls[0], "https://cdn.example.com/macos.zip");
    }
}
