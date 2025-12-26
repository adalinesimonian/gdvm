use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::collections::HashMap;

use crate::host::{HostArch, HostOs, HostPlatform};

const BASE_URL: &str =
    "https://raw.githubusercontent.com/adalinesimonian/gdvm/refs/heads/registry/v1";

#[derive(Debug, Deserialize, Clone)]
pub struct IndexEntry {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinaryInfo {
    pub sha512: String,
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReleaseMetadata {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub binaries: HashMap<String, HashMap<String, BinaryInfo>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySelectionError {
    UnsupportedPlatform,
    UnsupportedArch,
    MissingUrl,
}

pub struct Registry {
    client: reqwest::blocking::Client,
}

pub fn registry_platform_key(host: HostPlatform, is_csharp: bool) -> &'static str {
    match host.os {
        HostOs::Windows => {
            if is_csharp {
                "windows-csharp"
            } else {
                "windows"
            }
        }
        HostOs::Macos => {
            if is_csharp {
                "macos-csharp"
            } else {
                "macos"
            }
        }
        HostOs::Linux => {
            if is_csharp {
                "linux-csharp"
            } else {
                "linux"
            }
        }
    }
}

pub fn registry_arch_key(host: HostPlatform) -> &'static str {
    match host.arch {
        HostArch::X86_64 => "x86_64",
        HostArch::X86 => "x86",
        HostArch::Aarch64 => "arm64",
    }
}

/// Select the binary entry for a given host platform.
pub fn select_binary(
    meta: &ReleaseMetadata,
    host: HostPlatform,
    is_csharp: bool,
) -> Result<&BinaryInfo, BinarySelectionError> {
    let platform_key = registry_platform_key(host, is_csharp);
    let platform_map = meta
        .binaries
        .get(platform_key)
        .ok_or(BinarySelectionError::UnsupportedPlatform)?;

    let arch_key = if matches!(host.os, HostOs::Macos) && platform_map.contains_key("universal") {
        "universal"
    } else {
        registry_arch_key(host)
    };

    let binary = platform_map
        .get(arch_key)
        .ok_or(BinarySelectionError::UnsupportedArch)?;

    if binary.urls.is_empty() {
        return Err(BinarySelectionError::MissingUrl);
    }

    Ok(binary)
}

impl Registry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::blocking::ClientBuilder::new()
                .user_agent("gdvm")
                .build()?,
        })
    }

    pub fn fetch_index(&self) -> Result<Vec<IndexEntry>> {
        let url = format!("{BASE_URL}/index.json");
        let resp = self.client.get(&url).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Err(anyhow!("Failed to fetch registry index"))
        }
    }

    pub fn fetch_release(&self, id: u64, name: &str) -> Result<ReleaseMetadata> {
        let url = format!("{BASE_URL}/releases/{id}_{name}.json");
        let resp = self.client.get(&url).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Err(anyhow!("Failed to fetch release metadata"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(platform_key: &str, arch_key: &str) -> ReleaseMetadata {
        let mut binaries = HashMap::new();
        let mut arch_map = HashMap::new();
        arch_map.insert(
            arch_key.to_string(),
            BinaryInfo {
                sha512: "abc".to_string(),
                urls: vec!["http://example.com/godot.zip".to_string()],
            },
        );
        binaries.insert(platform_key.to_string(), arch_map);

        ReleaseMetadata {
            id: 1,
            name: "4.0.stable".to_string(),
            url: "http://example.com".to_string(),
            binaries,
        }
    }

    #[test]
    fn selects_macos_universal_when_present() {
        let mut meta = make_meta("macos", "x86_64");
        meta.binaries.get_mut("macos").unwrap().insert(
            "universal".to_string(),
            BinaryInfo {
                sha512: "def".to_string(),
                urls: vec!["http://example.com/universal.zip".to_string()],
            },
        );

        let host = HostPlatform {
            os: HostOs::Macos,
            arch: HostArch::X86_64,
        };

        let selected = select_binary(&meta, host, false).unwrap();
        assert_eq!(selected.urls[0], "http://example.com/universal.zip");
    }

    #[test]
    fn errors_when_platform_missing() {
        let meta = make_meta("linux", "x86_64");
        let host = HostPlatform {
            os: HostOs::Windows,
            arch: HostArch::X86_64,
        };

        let err = select_binary(&meta, host, false).unwrap_err();
        assert_eq!(err, BinarySelectionError::UnsupportedPlatform);
    }
}
