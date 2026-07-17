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
use std::time::Duration;

use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::terr;

/// URL to gdvm's own release manifest.
pub const GDVM_RELEASES_URL: &str = "https://registry.gdvm.io/gdvm/v1/releases.json";

/// Environment variable that overrides `GDVM_RELEASES_URL`.
pub const GDVM_RELEASES_URL_ENV_VAR: &str = "GDVM_RELEASES_URL";

/// The current schema version.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// The resolved manifest URL, honoring the override environment variable.
pub fn releases_url() -> String {
    std::env::var(GDVM_RELEASES_URL_ENV_VAR).unwrap_or_else(|_| GDVM_RELEASES_URL.to_string())
}

/// Parsed `releases.json` manifest for gdvm releases.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleasesManifest {
    pub schema: u32,
    /// ISO 8601 timestamp of the last change to the manifest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// All published releases.
    #[serde(default)]
    pub releases: Vec<GdvmRelease>,
}

/// A single published gdvm release.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GdvmRelease {
    /// Semantic version, e.g. `0.13.0` or `0.13.0-pre.1`.
    pub version: String,
    /// Git tag for the release.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Whether this release is a pre-release.
    #[serde(default)]
    pub prerelease: bool,
    /// Per-target binaries, keyed by Rust target.
    #[serde(default)]
    pub binaries: BTreeMap<String, GdvmBinary>,
}

/// Download and integrity information for a binary.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GdvmBinary {
    /// File name of the binary asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Archive size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Lower-case hex SHA 256 sum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// One or more download URLs. The first reachable URL is used.
    #[serde(default)]
    pub urls: Vec<String>,
    /// Provenance metadata for the binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

/// Provenance metadata for a binary.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Provenance {
    /// URL to the binary's attestation bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation_url: Option<String>,
}

impl GdvmRelease {
    /// The release version with any leading `v` stripped.
    pub fn normalized_version(&self) -> &str {
        self.version.trim_start_matches('v')
    }

    /// Parse the release's semantic version.
    pub fn parse_version(&self) -> Option<Version> {
        Version::parse(self.normalized_version()).ok()
    }

    /// The binary entry for a given target, if present.
    pub fn binary_for(&self, target: &str) -> Option<&GdvmBinary> {
        self.binaries.get(target)
    }
}

/// True when a release should be treated as stable.
fn is_stable(version: &Version, release: &GdvmRelease) -> bool {
    version.pre.is_empty() && !release.prerelease
}

/// Get releases and their versions.
fn parsed_releases(releases: &[GdvmRelease]) -> Vec<(Version, &GdvmRelease)> {
    releases
        .iter()
        .filter_map(|r| r.parse_version().map(|v| (v, r)))
        .collect()
}

/// Select the release that `gdvm upgrade` should upgrade to, or `None` when
/// there is nothing newer to upgrade to.
pub fn select_upgrade<'a>(
    releases: &'a [GdvmRelease],
    current: &Version,
    allow_major: bool,
    allow_pre: bool,
) -> Option<&'a GdvmRelease> {
    let parsed = parsed_releases(releases);
    let within_major = |v: &Version| allow_major || v.major == current.major;

    if allow_pre {
        return parsed
            .iter()
            .filter(|(v, _)| within_major(v) && v > current)
            .max_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, r)| *r);
    }

    let stable = parsed
        .iter()
        .filter(|(v, r)| is_stable(v, r) && within_major(v) && v > current)
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, r)| *r);

    if stable.is_some() {
        return stable;
    }

    // No newer stable release.
    if current.pre.is_empty() {
        return None;
    }

    parsed
        .iter()
        .filter(|(v, _)| {
            !v.pre.is_empty()
                && v.major == current.major
                && v.minor == current.minor
                && v.patch == current.patch
                && v > current
        })
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, r)| *r)
}

/// The highest stable version available within the major constraint.
pub fn highest_stable(
    releases: &[GdvmRelease],
    current: &Version,
    allow_major: bool,
) -> Option<Version> {
    parsed_releases(releases)
        .into_iter()
        .filter(|(v, r)| is_stable(v, r) && (allow_major || v.major == current.major))
        .map(|(v, _)| v)
        .max()
}

/// True when a newer pre-release is available.
pub fn newer_prerelease_available(
    releases: &[GdvmRelease],
    current: &Version,
    allow_major: bool,
) -> bool {
    // Only hint for users on a pre-release.
    if current.pre.is_empty() {
        return false;
    }

    let Some(with_pre) = select_upgrade(releases, current, allow_major, true) else {
        return false;
    };

    let Some(pre_version) = with_pre.parse_version() else {
        return false;
    };

    if pre_version.pre.is_empty() {
        return false;
    }

    match select_upgrade(releases, current, allow_major, false) {
        Some(stable) => stable
            .parse_version()
            .map(|sv| pre_version > sv)
            .unwrap_or(true),
        None => true,
    }
}

/// Fetch and parse the gdvm release manifest from `url`.
pub async fn fetch_manifest(url: &str, timeout: Duration) -> Result<ReleasesManifest> {
    let text = if let Some(path) = url.strip_prefix("file://") {
        std::fs::read_to_string(path)
            .map_err(|e| terr!("error-fetching-gdvm-releases").with_source(e))?
    } else {
        crate::download_utils::ensure_url_scheme_allowed(url)?;
        let client = crate::download_utils::http_client()?;
        let resp = crate::download_utils::get_retrying(&client, url, Some(timeout))
            .await
            .map_err(|e| terr!("error-fetching-gdvm-releases").with_string_source(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(terr!("error-fetching-gdvm-releases")
                .with_string_source(resp.status().to_string())
                .into());
        }
        crate::download_utils::response_text_limited(
            resp,
            crate::download_utils::MAX_METADATA_RESPONSE_SIZE,
        )
        .await
        .map_err(|e| terr!("error-fetching-gdvm-releases").with_string_source(e.to_string()))?
    };

    parse_manifest(&text)
}

/// Parse and validate a manifest from its JSON.
pub fn parse_manifest(text: &str) -> Result<ReleasesManifest> {
    let manifest: ReleasesManifest = serde_json::from_str(text)
        .map_err(|e| terr!("error-parsing-gdvm-releases").with_source(e))?;

    if manifest.schema != SUPPORTED_SCHEMA_VERSION {
        return Err(terr!("error-unsupported-gdvm-schema", schema = manifest.schema).into());
    }

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, prerelease: bool) -> GdvmRelease {
        let mut binaries = BTreeMap::new();
        binaries.insert(
            "x86_64-unknown-linux-gnu".to_string(),
            GdvmBinary {
                filename: Some(format!("gdvm-x86_64-unknown-linux-gnu-{version}")),
                size: Some(1024),
                sha256: Some("a".repeat(64)),
                urls: vec![format!(
                    "https://example.com/{version}/gdvm-x86_64-unknown-linux-gnu"
                )],
                provenance: None,
            },
        );
        GdvmRelease {
            version: version.to_string(),
            tag: Some(format!("v{version}")),
            prerelease,
            binaries,
        }
    }

    fn version(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    fn selected(target: Option<&GdvmRelease>) -> Option<String> {
        target.map(|r| r.version.clone())
    }

    #[test]
    fn stable_upgrade_stays_within_major_by_default() {
        let releases = vec![
            release("0.12.1", false),
            release("0.13.0", false),
            release("1.0.0", false),
        ];
        let current = version("0.12.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn stable_upgrade_crosses_major_when_allowed() {
        let releases = vec![
            release("0.12.1", false),
            release("0.13.0", false),
            release("1.0.0", false),
        ];
        let current = version("0.12.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, true, false)),
            Some("1.0.0".to_string())
        );
    }

    #[test]
    fn stable_upgrade_ignores_prereleases() {
        let releases = vec![
            release("0.12.1", false),
            release("0.13.0-pre.1", true),
            release("0.13.0-pre.2", true),
        ];
        let current = version("0.12.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            None
        );
    }

    #[test]
    fn pre_flag_picks_latest_prerelease() {
        let releases = vec![
            release("0.12.1", false),
            release("0.13.0-pre.1", true),
            release("0.13.0-pre.2", true),
        ];
        let current = version("0.12.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, true)),
            Some("0.13.0-pre.2".to_string())
        );
    }

    #[test]
    fn pre_flag_prefers_stable_of_same_version() {
        let releases = vec![release("0.13.0-pre.2", true), release("0.13.0", false)];
        let current = version("0.12.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, true)),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn prerelease_user_rolls_forward_to_newer_prerelease_of_same_version() {
        let releases = vec![
            release("0.13.0-pre.1", true),
            release("0.13.0-pre.2", true),
            release("0.13.0-pre.3", true),
        ];
        let current = version("0.13.0-pre.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            Some("0.13.0-pre.3".to_string())
        );
    }

    #[test]
    fn prerelease_user_does_not_jump_to_other_prerelease_versions() {
        let releases = vec![release("0.13.0-pre.1", true), release("0.14.0-pre.1", true)];
        let current = version("0.13.0-pre.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            None
        );
        assert_eq!(
            selected(select_upgrade(&releases, &current, false, true)),
            Some("0.14.0-pre.1".to_string())
        );
    }

    #[test]
    fn prerelease_user_prefers_newer_stable_over_prerelease_fallback() {
        let releases = vec![
            release("0.13.0-pre.1", true),
            release("0.13.0-pre.2", true),
            release("0.13.0", false),
        ];
        let current = version("0.13.0-pre.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn no_upgrade_when_current_is_newest() {
        let releases = vec![release("0.12.1", false), release("0.13.0", false)];
        let current = version("0.13.0");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            None
        );
        assert_eq!(
            selected(select_upgrade(&releases, &current, true, true)),
            None
        );
    }

    #[test]
    fn prerelease_flag_guards_clean_semver_from_being_stable() {
        let releases = vec![release("0.12.1", false), release("0.13.0", true)];
        let current = version("0.12.1");

        assert_eq!(
            selected(select_upgrade(&releases, &current, false, false)),
            None
        );
        assert_eq!(
            selected(select_upgrade(&releases, &current, false, true)),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn newer_prerelease_available_hint() {
        let current = version("0.12.1");

        let releases = vec![release("0.12.1", false), release("0.13.0-pre.1", true)];
        assert!(!newer_prerelease_available(&releases, &current, false));

        let releases = vec![release("0.12.1", false), release("0.13.0", false)];
        assert!(!newer_prerelease_available(&releases, &current, false));

        let releases = vec![release("0.12.1", false)];
        assert!(!newer_prerelease_available(&releases, &current, false));

        let releases = vec![release("0.12.1", false), release("1.0.0-pre.1", true)];
        assert!(!newer_prerelease_available(&releases, &current, false));
        assert!(!newer_prerelease_available(&releases, &current, true));
    }

    #[test]
    fn newer_prerelease_available_hint_prerelease_user() {
        let current = version("0.13.0-pre.1");
        let releases = vec![release("0.13.0-pre.1", true), release("0.14.0-pre.1", true)];
        assert!(newer_prerelease_available(&releases, &current, false));

        let releases = vec![release("0.13.0-pre.1", true), release("1.0.0-pre.1", true)];
        assert!(!newer_prerelease_available(&releases, &current, false));
        assert!(newer_prerelease_available(&releases, &current, true));

        let releases = vec![
            release("0.13.0-pre.1", true),
            release("0.14.0-pre.1", true),
            release("0.14.0", false),
        ];
        assert!(!newer_prerelease_available(&releases, &current, false));

        let releases = vec![release("0.13.0-pre.1", true)];
        assert!(!newer_prerelease_available(&releases, &current, false));
    }

    #[test]
    fn highest_stable_reports_newest_release() {
        let releases = vec![
            release("0.12.1", false),
            release("0.13.0", false),
            release("0.14.0-pre.1", true),
            release("1.0.0", false),
        ];
        let current = version("0.13.0");

        assert_eq!(
            highest_stable(&releases, &current, false),
            Some(version("0.13.0"))
        );
        assert_eq!(
            highest_stable(&releases, &current, true),
            Some(version("1.0.0"))
        );
    }

    #[test]
    fn parse_manifest_accepts_known_schema_and_ignores_unknown_fields() {
        let json = r#"{
            "schema": 1,
            "updated_at": "2026-06-30T00:00:00Z",
            "future_top_level_field": 42,
            "releases": [
                {
                    "version": "0.13.0",
                    "tag": "v0.13.0",
                    "prerelease": false,
                    "future_release_field": "ignored",
                    "binaries": {
                        "x86_64-unknown-linux-gnu": {
                            "filename": "gdvm-x86_64-unknown-linux-gnu",
                            "size": 1024,
                            "sha256": "deadbeef",
                            "urls": ["https://example.com/gdvm"],
                            "provenance": {
                                "attestation_url": "https://example.com/att.jsonl",
                                "future_provenance_field": true
                            }
                        }
                    }
                }
            ]
        }"#;

        let manifest = parse_manifest(json).expect("parse");
        assert_eq!(manifest.schema, 1);
        assert_eq!(manifest.releases.len(), 1);
        let release = &manifest.releases[0];
        assert_eq!(release.version, "0.13.0");
        let binary = release.binary_for("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(binary.sha256.as_deref(), Some("deadbeef"));
        assert_eq!(
            binary
                .provenance
                .as_ref()
                .and_then(|p| p.attestation_url.as_deref()),
            Some("https://example.com/att.jsonl")
        );
    }

    #[test]
    fn parse_manifest_rejects_unsupported_schema() {
        let json = r#"{ "schema": 99, "releases": [] }"#;
        assert!(parse_manifest(json).is_err());
    }
}
