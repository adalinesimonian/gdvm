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
