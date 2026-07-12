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

mod v2;

pub mod publish;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::host::{HostArch, HostOs, HostPlatform};
use crate::t;

/// Returns true if the registry refers to the official gdvm registry.
pub fn is_official_registry(registry: Option<&str>) -> bool {
    match registry {
        None => true,
        Some(r) => r == crate::registry::OFFICIAL_REGISTRY,
    }
}

/// Normalize a registry name to `None` for the official registry, or
/// `Some(name)` for a custom registry.
pub fn normalize_registry(registry: Option<&str>) -> Option<&str> {
    match registry {
        Some(r) if r != crate::registry::OFFICIAL_REGISTRY => Some(r),
        _ => None,
    }
}

/// Alias for the official registry. Cannot be overridden by a project pin or
/// machine config.
pub const OFFICIAL_REGISTRY: &str = "official";

/// Built-in base URL for the official registry.
pub const OFFICIAL_BASE_URL: &str = "https://registry.gdvm.io/v2";

/// Normalize a registry URL into a form suitable for identifying the registry.
pub fn normalize_url(url: &str) -> String {
    let trimmed = url.trim();

    let without_frag = trimmed.split(['?', '#']).next().unwrap_or(trimmed);

    let Some((scheme, rest)) = without_frag.split_once("://") else {
        return without_frag.trim_end_matches('/').to_string();
    };
    let scheme = scheme.to_ascii_lowercase();

    // File URLs get used basically as-is.
    if scheme == "file" {
        let path = rest.trim_end_matches('/');
        return format!("file://{path}");
    }

    // Split authority from the path for http(s).
    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, ""),
    };

    // Keep user info if present.
    let (userinfo, hostport) = match authority.rsplit_once('@') {
        Some((user, hp)) => (Some(user), hp),
        None => (None, authority),
    };

    let mut host = hostport.to_ascii_lowercase();
    if let Some((h, port)) = hostport.rsplit_once(':')
        && port.chars().all(|c| c.is_ascii_digit())
    {
        let default_port = match scheme.as_str() {
            "http" => Some("80"),
            "https" => Some("443"),
            _ => None,
        };
        host = if Some(port) == default_port {
            h.to_ascii_lowercase()
        } else {
            format!("{}:{port}", h.to_ascii_lowercase())
        };
    }

    let authority = match userinfo {
        Some(user) => format!("{user}@{host}"),
        None => host,
    };

    let path = path.trim_end_matches('/');
    format!("{scheme}://{authority}{path}")
}

/// Get a host token for a registry URL.
fn host_token(normalized_url: &str) -> String {
    let host = normalized_url
        .split_once("://")
        .map(|(scheme, rest)| {
            if scheme == "file" {
                "file"
            } else {
                let authority = rest.split('/').next().unwrap_or(rest);
                authority.rsplit_once('@').map_or(authority, |(_, hp)| hp)
            }
        })
        .unwrap_or("registry");

    let token: String = host
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();

    let token = token.trim_matches('-').to_string();
    if token.is_empty() {
        "registry".to_string()
    } else {
        token
    }
}

/// Get the directory name for a registry's install store based on its URL.
pub fn store_dir_name(url: &str) -> String {
    let normalized = normalize_url(url);
    let digest = Sha256::digest(normalized.as_bytes());
    let hash = crate::hash_utils::to_hex(&digest[..8]);
    format!("{}-{hash}", host_token(&normalized))
}

/// The install store directory name for the official registry.
pub fn official_store_dir_name() -> String {
    store_dir_name(OFFICIAL_BASE_URL)
}

/// Normalized, schema-independent download metadata for a single release.
#[derive(Debug, Clone)]
pub struct ReleaseMetadata {
    pub version: String,
    /// Map of variant names to binaries.
    pub variants: HashMap<String, HashMap<String, BinaryInfo>>,
}

/// Integrity and download information for a single platform binary.
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    pub sha512: String,
    /// Archive size in bytes.
    pub size: Option<u64>,
    /// Download URLs.
    pub urls: Vec<String>,
}

/// A normalized index entry describing one release available in a registry.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub version: String,
    /// Map of variant names to platform keys, e.g. `default` to `linux-x86_64`.
    pub variants: Option<HashMap<String, Vec<String>>>,
    /// How to address this release's download metadata file.
    pub source: ReleaseRef,
}

/// How to address a release's metadata file within a registry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ReleaseRef {
    /// In v2, addressed by an explicit relative path from the index.
    V2 { path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySelectionError {
    UnsupportedPlatform,
    UnsupportedArch,
    MissingUrl,
}

/// A registry base location.
#[derive(Debug, Clone)]
pub enum RegistryUrl {
    /// HTTP/HTTPS base URL with no trailing slash.
    Http(String),
    /// Local directory acting as the registry root.
    File(PathBuf),
}

impl RegistryUrl {
    /// Parse a registry base URL. Accepts `http://`, `https://`, and `file://`.
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(rest) = s.strip_prefix("file://") {
            Ok(RegistryUrl::File(PathBuf::from(rest)))
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Ok(RegistryUrl::Http(s.trim_end_matches('/').to_string()))
        } else {
            Err(anyhow!(t!(
                "error-registry-unsupported-url-scheme",
                url = s
            )))
        }
    }

    /// Return true if `url` already carries a scheme and must be used as-is.
    fn is_absolute(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://")
    }

    /// Resolve a possibly relative download URL against this base. Absolute
    /// URLs pass through.
    pub fn join(&self, rel: &str) -> String {
        if Self::is_absolute(rel) {
            return rel.to_string();
        }
        match self {
            RegistryUrl::Http(base) => format!("{base}/{}", rel.trim_start_matches('/')),
            RegistryUrl::File(dir) => {
                let base = dir.to_string_lossy();
                format!(
                    "file://{}/{}",
                    base.trim_end_matches('/'),
                    rel.trim_start_matches('/')
                )
            }
        }
    }

    /// A display string for the base URL.
    pub fn as_display(&self) -> String {
        match self {
            RegistryUrl::Http(base) => base.clone(),
            RegistryUrl::File(dir) => format!("file://{}", dir.to_string_lossy()),
        }
    }
}

/// The OS portion of a platform key for the host.
pub fn registry_os_key(host: HostPlatform) -> &'static str {
    match host.os {
        HostOs::Windows => "windows",
        HostOs::Macos => "macos",
        HostOs::Linux => "linux",
    }
}

/// The arch portion of a platform key for the host.
pub fn registry_arch_key(host: HostPlatform) -> &'static str {
    match host.arch {
        HostArch::X86_64 => "x86_64",
        HostArch::X86 => "x86",
        HostArch::Aarch64 => "arm64",
    }
}

/// Candidate platform keys for the host, sorted by preference, with the most
/// specific first.
pub fn platform_candidates(host: HostPlatform) -> Vec<String> {
    let os = registry_os_key(host);
    let arch = registry_arch_key(host);
    let exact = format!("{os}-{arch}");
    let universal = format!("{os}-universal");
    if matches!(host.os, HostOs::Macos) {
        vec![universal, exact]
    } else {
        vec![exact, universal]
    }
}

/// Select the binary entry for a given host platform and variant.
pub fn select_binary<'a>(
    meta: &'a ReleaseMetadata,
    host: HostPlatform,
    variant: &crate::version::Variant,
) -> Result<&'a BinaryInfo, BinarySelectionError> {
    let platform_map = meta
        .variants
        .get(variant.as_str())
        .ok_or(BinarySelectionError::UnsupportedPlatform)?;

    let binary = platform_candidates(host)
        .iter()
        .find_map(|key| platform_map.get(key))
        .ok_or(BinarySelectionError::UnsupportedArch)?;

    if binary.urls.is_empty() {
        return Err(BinarySelectionError::MissingUrl);
    }

    Ok(binary)
}

/// A handle to a single registry.
pub struct Registry {
    client: reqwest::Client,
    name: String,
    base_url: RegistryUrl,
}

impl Registry {
    /// Construct the built-in official registry.
    pub fn official() -> Result<Self> {
        Self::new(OFFICIAL_REGISTRY, OFFICIAL_BASE_URL)
    }

    /// Construct a registry with the given name and base URL.
    pub fn new(name: &str, base_url: &str) -> Result<Self> {
        Ok(Self {
            client: crate::download_utils::http_client()?,
            name: name.to_string(),
            base_url: RegistryUrl::parse(base_url)?,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// A display string for the registry's base URL.
    pub fn base_url_display(&self) -> String {
        self.base_url.as_display()
    }

    /// Key for this registry's cached metadata.
    pub fn cache_key(&self) -> String {
        normalize_url(&self.base_url.as_display())
    }

    /// Fetch text for a registry-relative path. `Ok(None)` means the file is
    /// missing. Any other failure is an error.
    async fn fetch_text(&self, rel: &str) -> Result<Option<String>> {
        match &self.base_url {
            RegistryUrl::Http(base) => {
                crate::download_utils::ensure_url_scheme_allowed(base)?;
                let url = format!("{base}/{}", rel.trim_start_matches('/'));
                let resp = crate::download_utils::get_retrying(&self.client, &url, None).await?;
                if resp.status() == reqwest::StatusCode::NOT_FOUND {
                    return Ok(None);
                }
                if !resp.status().is_success() {
                    return Err(anyhow!(t!(
                        "error-registry-fetch-failed",
                        url = url,
                        status = resp.status().to_string()
                    )));
                }
                Ok(Some(
                    crate::download_utils::response_text_limited(
                        resp,
                        crate::download_utils::MAX_METADATA_RESPONSE_SIZE,
                    )
                    .await?,
                ))
            }
            RegistryUrl::File(dir) => {
                let path = dir.join(rel);
                match std::fs::read_to_string(&path) {
                    Ok(contents) => Ok(Some(contents)),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
        }
    }

    /// Fetch and normalize the registry index.
    pub async fn fetch_index(&self) -> Result<Vec<IndexEntry>> {
        let manifest_text = self.fetch_text("registry.json").await?.ok_or_else(|| {
            anyhow!(t!(
                "error-registry-missing-manifest",
                name = self.name.as_str()
            ))
        })?;
        let manifest: v2::Manifest = serde_json::from_str(&manifest_text).map_err(|e| {
            anyhow!(t!(
                "error-registry-parse-manifest",
                name = self.name.as_str(),
                error = e.to_string()
            ))
        })?;
        if manifest.schema != 2 {
            return Err(anyhow!(
                "Registry '{}' declares unsupported schema version {}",
                self.name,
                manifest.schema
            ));
        }

        let index_text = self.fetch_text("index.json").await?.ok_or_else(|| {
            anyhow!(t!(
                "error-registry-missing-index",
                name = self.name.as_str()
            ))
        })?;
        let index: v2::Index = serde_json::from_str(&index_text).map_err(|e| {
            anyhow!(t!(
                "error-registry-parse-index",
                name = self.name.as_str(),
                error = e.to_string()
            ))
        })?;
        Ok(v2::normalize_index(index))
    }

    /// Fetch and normalize the download metadata for a single release.
    pub async fn fetch_release(&self, source: &ReleaseRef) -> Result<ReleaseMetadata> {
        match source {
            ReleaseRef::V2 { path } => {
                let text = self
                    .fetch_text(path)
                    .await?
                    .ok_or_else(|| anyhow!(t!("error-registry-fetch-release-failed")))?;
                let meta: v2::ReleaseMetadata = serde_json::from_str(&text)?;
                Ok(v2::normalize_release(meta, &self.base_url))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binary(url: &str) -> BinaryInfo {
        BinaryInfo {
            sha512: "abc".to_string(),
            size: None,
            urls: vec![url.to_string()],
        }
    }

    fn meta_with(variant: &str, platform: &str, url: &str) -> ReleaseMetadata {
        let mut platforms = HashMap::new();
        platforms.insert(platform.to_string(), binary(url));
        let mut variants = HashMap::new();
        variants.insert(variant.to_string(), platforms);
        ReleaseMetadata {
            version: "4.4-stable".to_string(),
            variants,
        }
    }

    #[test]
    fn registry_url_join_relative_and_absolute() {
        let http = RegistryUrl::parse("https://example.com/godot/").unwrap();
        assert_eq!(
            http.join("binaries/4.4/linux.zip"),
            "https://example.com/godot/binaries/4.4/linux.zip"
        );
        assert_eq!(
            http.join("https://cdn.example.com/a.zip"),
            "https://cdn.example.com/a.zip"
        );

        let file = RegistryUrl::parse("file:///tmp/reg").unwrap();
        assert_eq!(file.join("index.json"), "file:///tmp/reg/index.json");
    }

    #[test]
    fn select_binary_prefers_macos_universal() {
        let mut meta = meta_with("default", "macos-x86_64", "http://example.com/x86.zip");
        meta.variants.get_mut("default").unwrap().insert(
            "macos-universal".to_string(),
            binary("http://example.com/u.zip"),
        );

        let host = HostPlatform {
            os: HostOs::Macos,
            arch: HostArch::X86_64,
        };
        let selected = select_binary(&meta, host, &crate::version::Variant::default()).unwrap();
        assert_eq!(selected.urls[0], "http://example.com/u.zip");
    }

    #[test]
    fn select_binary_errors_when_variant_missing() {
        let meta = meta_with("default", "linux-x86_64", "http://example.com/godot.zip");
        let host = HostPlatform {
            os: HostOs::Linux,
            arch: HostArch::X86_64,
        };
        let err = select_binary(
            &meta,
            host,
            &crate::version::Variant::from_option(Some("csharp")),
        )
        .unwrap_err();
        assert_eq!(err, BinarySelectionError::UnsupportedPlatform);
    }

    #[test]
    fn select_binary_errors_when_arch_missing() {
        let meta = meta_with("default", "linux-x86_64", "http://example.com/godot.zip");
        let host = HostPlatform {
            os: HostOs::Windows,
            arch: HostArch::X86_64,
        };
        let err = select_binary(&meta, host, &crate::version::Variant::default()).unwrap_err();
        assert_eq!(err, BinarySelectionError::UnsupportedArch);
    }

    #[test]
    fn normalize_url_folds_scheme_host_port_and_trailing_slash() {
        assert_eq!(
            normalize_url("HTTPS://Builds.Example.COM:443/godot/"),
            "https://builds.example.com/godot"
        );
        assert_eq!(
            normalize_url("http://example.com:8080/Godot/Builds"),
            "http://example.com:8080/Godot/Builds"
        );
        assert_eq!(
            normalize_url("https://example.com/r?token=abc#frag"),
            "https://example.com/r"
        );
        assert_eq!(normalize_url("file:///srv/reg/"), "file:///srv/reg");
    }

    #[test]
    fn store_dir_name_is_stable_and_equivalent_for_equivalent_urls() {
        let a = store_dir_name("https://Builds.Example.com/godot/");
        let b = store_dir_name("https://builds.example.com:443/godot");
        assert_eq!(a, b, "equivalent URLs must map to the same store");
        assert!(a.starts_with("builds.example.com-"));

        assert_ne!(a, store_dir_name("https://other.example.com/godot"));

        let official = official_store_dir_name();
        assert_eq!(official, store_dir_name(OFFICIAL_BASE_URL));
        assert!(official.starts_with("registry.gdvm.io-"));
    }

    #[test]
    fn store_dir_name_handles_file_urls() {
        let key = store_dir_name("file:///home/user/my-reg");
        assert!(key.starts_with("file-"), "got: {key}");
    }
}
