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

use serde::{Deserialize, Serialize};
use std::fmt;

/// The gdvm pin file.
#[derive(Debug, Serialize, Deserialize)]
pub struct GdvmToml {
    pub godot: GdvmTomlGodot,
}

/// The `[godot]` section of `gdvm.toml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GdvmTomlGodot {
    pub version: String,
}

/// A fully parsed version specifier from the specifier format
/// `[registry/][variant:]version_or_keyword`.
///
/// Examples:
/// - `4.4` - official registry, no variant, version pattern
/// - `csharp:4.4` - official registry, csharp variant, version pattern
/// - `stable` - official registry, no variant, keyword
/// - `myregistry/csharp:stable` - custom registry, csharp variant, keyword
#[derive(Debug, Clone)]
pub struct VersionSpec {
    pub registry: Option<String>,
    pub variant: Option<String>,
    pub target: VersionTarget,
}

/// Either a client-side keyword or a version pattern.
#[derive(Debug, Clone)]
pub enum VersionTarget {
    /// A client-side keyword: `stable`, `latest`
    Keyword(String),
    /// A version pattern: `4.4`, `4.4.1-rc1`
    Pattern(GodotVersion),
}

impl VersionSpec {
    /// Parse a specifier string using the grammar `[registry/][variant:]version_or_keyword`.
    pub fn parse(input: &str) -> Result<Self, anyhow::Error> {
        let (registry, remainder) = match input.find('/') {
            Some(pos) => {
                let reg = &input[..pos];
                if reg.is_empty() {
                    return Err(anyhow::anyhow!("Empty registry name in '{input}'"));
                }
                (Some(reg.to_string()), &input[pos + 1..])
            }
            None => (None, input),
        };

        let (variant, version_str) = match remainder.find(':') {
            Some(pos) => {
                let var = &remainder[..pos];
                if var.is_empty() {
                    return Err(anyhow::anyhow!("Empty variant name in '{input}'"));
                }
                (Some(var.to_string()), &remainder[pos + 1..])
            }
            None => (None, remainder),
        };

        if version_str.is_empty() {
            return Err(anyhow::anyhow!("Empty version in '{input}'"));
        }

        let target = if is_keyword(version_str) {
            VersionTarget::Keyword(normalize_keyword(version_str))
        } else {
            VersionTarget::Pattern(GodotVersion::from_remote_str(version_str)?)
        };

        Ok(VersionSpec {
            registry,
            variant,
            target,
        })
    }
}

/// Returns true if the string is a known client-side keyword.
fn is_keyword(s: &str) -> bool {
    matches!(s, "stable" | "latest")
}

/// Normalize keyword to its canonical lowercase form.
fn normalize_keyword(s: &str) -> String {
    s.to_lowercase()
}

/// Validate a version specifier for use as a clap value parser.
pub fn validate_version_spec(s: &str) -> Result<String, String> {
    VersionSpec::parse(s)
        .map(|_| s.to_string())
        .map_err(|e| e.to_string())
}

/// The variant name to pull from the registry when the user doesn't explicitly
/// specify a variant.
pub const DEFAULT_VARIANT: &str = "default";

/// Returns true if `variant` is `None` or refers to the default variant.
pub fn is_default_variant(variant: Option<&str>) -> bool {
    matches!(variant, None | Some(DEFAULT_VARIANT))
}

/// Return the variant string for a given variant specifier, e.g. `None` becomes
/// `default`, `Some("csharp")` returns `csharp`.
pub fn normalize_variant(variant: Option<&str>) -> &str {
    match variant {
        Some(v) if v != DEFAULT_VARIANT => v,
        _ => DEFAULT_VARIANT,
    }
}

/// Get a user-friendly name to use when displaying the Godot version. If the
/// variant is default, omits the variant from the output.
pub fn display_with_variant(version_str: &str, variant: Option<&str>) -> String {
    if is_default_variant(variant) {
        version_str.to_string()
    } else {
        format!("{version_str} ({})", variant.unwrap())
    }
}

/// Get the install directory subpath for a given version and variant.
pub fn install_dir_subpath(version_str: &str, variant: Option<&str>) -> String {
    format!("{}/{version_str}", normalize_variant(variant))
}

/// Build a pinned version string.
pub fn pinned_str(version_str: &str, variant: Option<&str>) -> String {
    format!("{}:{version_str}", normalize_variant(variant))
}

/// Parse a pinned string back into (variant, version_string).
/// Supports both new format `csharp:4.4.0-stable` and legacy `4.4.0-stable-csharp`.
pub fn parse_pinned_str(s: &str) -> (Option<String>, String) {
    // New format: variant:version (variant doesn't start with digit)
    if let Some(pos) = s.find(':') {
        let candidate_variant = &s[..pos];
        if !candidate_variant.is_empty()
            && !candidate_variant
                .chars()
                .next()
                .unwrap_or('0')
                .is_ascii_digit()
        {
            return (
                Some(candidate_variant.to_string()),
                s[pos + 1..].to_string(),
            );
        }
    }

    // Legacy format: version-csharp
    if let Some(version_part) = s.strip_suffix("-csharp") {
        return (Some("csharp".to_string()), version_part.to_string());
    }

    (None, s.to_string())
}

/// Build a legacy `.gdvmrc` pin string in the old pre-refactor format.
/// Produces the format that old gdvm versions understand:
/// `4.3.0-stable-csharp` (zero-padded, `-csharp` suffix) or `4.3.0-stable`.
pub fn legacy_pinned_str(gv: &GodotVersionDeterminate, variant: Option<&str>) -> String {
    let mut base = gv.to_pinned_str();
    if variant == Some("csharp") {
        base.push_str("-csharp");
    }
    base
}

/// Serialize a `GdvmToml` to a TOML string.
pub fn serialize_gdvm_toml(version_specifier: &str) -> String {
    let gdvm_toml = GdvmToml {
        godot: GdvmTomlGodot {
            version: version_specifier.to_string(),
        },
    };
    toml::to_string(&gdvm_toml).expect("GdvmToml serialization should never fail")
}

/// Deserialize a `GdvmToml` from a TOML string.
pub fn deserialize_gdvm_toml(contents: &str) -> Result<GdvmToml, toml::de::Error> {
    toml::from_str(contents)
}

#[derive(Debug, Default)]
pub struct GodotVersion {
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub subpatch: Option<u32>,
    pub release_type: Option<String>,
}

impl From<GodotVersionDeterminate> for GodotVersion {
    fn from(gvd: GodotVersionDeterminate) -> Self {
        gvd.to_indeterminate()
    }
}

impl From<&GodotVersionDeterminate> for GodotVersion {
    fn from(gvd: &GodotVersionDeterminate) -> Self {
        gvd.to_indeterminate()
    }
}

impl Clone for GodotVersion {
    fn clone(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            subpatch: self.subpatch,
            release_type: self.release_type.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GodotVersionDeterminate {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub subpatch: u32,
    pub release_type: String,
}

impl From<GodotVersion> for GodotVersionDeterminate {
    fn from(gv: GodotVersion) -> Self {
        gv.to_determinate()
    }
}

impl From<&GodotVersion> for GodotVersionDeterminate {
    fn from(gv: &GodotVersion) -> Self {
        gv.to_determinate()
    }
}

impl Default for GodotVersionDeterminate {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            subpatch: 0,
            release_type: "stable".to_string(),
        }
    }
}

impl Clone for GodotVersionDeterminate {
    fn clone(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            subpatch: self.subpatch,
            release_type: self.release_type.clone(),
        }
    }
}

impl GodotVersion {
    /// Parse an installed version folder name .
    pub fn from_install_str(s: &str) -> Result<Self, anyhow::Error> {
        // Strip legacy -csharp suffix if present
        let clean = s.strip_suffix("-csharp").unwrap_or(s);
        Self::parse_version_and_pre_release(clean)
    }

    /// Parse a remote version tag, e.g. "4.1-stable", "3-rc1".
    pub fn from_remote_str(s: &str) -> Result<Self, anyhow::Error> {
        Self::parse_version_and_pre_release(s)
    }

    /// Converts the version to a string, omitting trailing zeros and ensuring component integrity.
    ///
    /// Returns:
    /// - `Some(String)` with the formatted version if all necessary components are present.
    /// - `None` if any intermediate component is missing, leading to an invalid version string.
    fn to_version_string(&self) -> Option<String> {
        // - If `subpatch` is Some, `patch` must be Some.
        // - If `patch` is Some, `minor` must be Some.
        // Prevents cases like "3.None.1" or "3.1.None.1".
        if self.subpatch.is_some() && self.patch.is_none() {
            return None;
        }
        if self.patch.is_some() && self.minor.is_none() {
            return None;
        }

        // Arrange components in order: major, minor, patch, subpatch.
        let components = [
            self.major,    // Major is Optional
            self.minor,    // Minor is Optional
            self.patch,    // Patch is Optional
            self.subpatch, // Subpatch is Optional (only present in 2.0.4.1)
        ];

        // Find the last significant component, the one that is Some and non-zero.
        let last_non_zero = components.iter().rposition(|&x| match x {
            Some(val) => val != 0,
            None => false,
        });

        // If all components are zero or only major is present
        let last_non_zero = last_non_zero.unwrap_or(0);

        // - If the last non-zero component is before minor but minor is present, include minor.
        // - This ensures "4.0.0.0" becomes "4.0" instead of "4".
        let truncate_to = if last_non_zero < 1 && self.minor.is_some() {
            1
        } else {
            last_non_zero
        };

        // Ensure all components up to `truncate_to` are present
        for component in components.iter().take(truncate_to + 1) {
            (*component)?;
        }

        // Collect and format the version string
        let version_components: Vec<String> = components[..=truncate_to]
            .iter()
            .map(|&x| x.unwrap().to_string())
            .collect();

        Some(version_components.join("."))
    }

    /// Convert to a remote tag, e.g. "4.1.1-rc1" or "2.0.4.1-stable"
    /// .0 patch versions are omitted, e.g. "4.1" instead of "4.1.0"
    /// Same with subpatch, e.g. "2.0.0.0" => "2.0.0" (however only 2.0.4.1 has subpatch anyway)
    pub fn to_remote_str(&self) -> Option<String> {
        let mut base = self.to_version_string()?;
        if let Some(ref release_type) = self.release_type {
            base.push('-');
            base.push_str(release_type);
        }
        Some(base)
    }

    /// Convert to a display-friendly string, e.g. "4.1.1-rc1".
    pub fn to_display_str(&self) -> Option<String> {
        self.to_remote_str()
    }

    fn parse_version_and_pre_release(raw: &str) -> Result<Self, anyhow::Error> {
        // Read the pre-release identifier, if present.
        let (version_part, pre_release) = match raw.rfind('-') {
            Some(index) => (&raw[..index], Some(raw[index + 1..].to_string())),
            None => (raw, None),
        };

        // Parse major/minor/patch from version_part, including special 2.0.4.1
        let pieces: Vec<&str> = version_part.split('.').collect();
        let (major, minor, patch, subpatch) = match pieces.len() {
            0 => (None, None, None, None),
            1 => (Some(pieces[0].parse()?), None, None, None),
            2 => (
                Some(pieces[0].parse()?),
                Some(pieces[1].parse()?),
                None,
                None,
            ),
            3 => (
                Some(pieces[0].parse()?),
                Some(pieces[1].parse()?),
                Some(pieces[2].parse()?),
                None,
            ),
            4 => (
                Some(pieces[0].parse()?),
                Some(pieces[1].parse()?),
                Some(pieces[2].parse()?),
                Some(pieces[3].parse()?),
            ),
            _ => return Err(anyhow::anyhow!("Unrecognized version format: {raw}")),
        };

        Ok(GodotVersion {
            major,
            minor,
            patch,
            subpatch,
            release_type: pre_release,
        })
    }

    /// Creates a GodotVersion from a match string, e.g. "stable", "4", "3.5"
    /// Match strings are just like remote tags, but can also be "stable".
    pub fn from_match_str(s: &str) -> Result<Self, anyhow::Error> {
        if s == "stable" {
            return Ok(GodotVersion {
                release_type: Some("stable".to_string()),
                ..Default::default()
            });
        }

        Self::from_remote_str(s)
    }

    /// Returns if the version is a stable release.
    pub fn is_stable(&self) -> bool {
        self.release_type.as_deref() == Some("stable")
    }

    /// Returns if the version is incomplete, i.e. any of the major, minor, or patch components are
    /// missing. (For 2.0.4, subpatch is considered missing, for all other versions it is ignored.
    /// This is because 2.0.4.1 is the only version with a subpatch.)
    pub fn is_incomplete(&self) -> bool {
        if self.major == Some(2) && self.minor == Some(0) && self.patch == Some(4) {
            self.subpatch.is_none() || self.release_type.is_none()
        } else {
            self.major.is_none()
                || self.minor.is_none()
                || self.patch.is_none()
                || self.release_type.is_none()
        }
    }

    /// Returns a new version with all missing components zeroed out.
    pub fn to_determinate(&self) -> GodotVersionDeterminate {
        GodotVersionDeterminate {
            major: self.major.unwrap_or(0),
            minor: self.minor.unwrap_or(0),
            patch: self.patch.unwrap_or(0),
            subpatch: self.subpatch.unwrap_or(0),
            release_type: self.release_type.as_deref().unwrap_or("").to_string(),
        }
    }

    /// Checks if the version matches another version. Version parts that are None are interpreted
    /// as wildcards.
    pub fn matches<T>(&self, other: &T) -> bool
    where
        T: Into<GodotVersion> + Clone,
    {
        let other: GodotVersion = other.clone().into();

        if let Some(major) = self.major
            && other.major.is_some_and(|x| x != major)
        {
            return false;
        }
        if let Some(minor) = self.minor
            && other.minor.is_some_and(|x| x != minor)
        {
            return false;
        }
        if let Some(patch) = self.patch
            && other.patch.is_some_and(|x| x != patch)
        {
            return false;
        }
        if let Some(subpatch) = self.subpatch
            && other.subpatch.is_some_and(|x| x != subpatch)
        {
            return false;
        }
        if let Some(release_type) = &self.release_type
            && other.release_type.is_some_and(|x| &x != release_type)
        {
            return false;
        }
        true
    }

    /// Returns true when all specified project constraints conflict with the requested version.
    /// Missing components in either version act as wildcards.
    pub fn conflicts_with(&self, requested: &GodotVersion) -> bool {
        fn differs(lhs: Option<u32>, rhs: Option<u32>) -> bool {
            lhs.is_some() && rhs.is_some() && lhs != rhs
        }

        // Check if they don't match (project versions at most specify major.minor or
        // major.minor.patch, and if .patch is not specified, it's assumed to allow any patch)
        differs(self.major, requested.major)
            || differs(self.minor, requested.minor)
            || differs(self.patch, requested.patch)
    }
}

impl GodotVersionDeterminate {
    /// Converts the version to a string.
    pub fn to_version_string(&self, fully_qualified: bool) -> String {
        let mut base = format!("{}.{}", self.major, self.minor);

        if self.patch != 0 || self.subpatch != 0 || fully_qualified {
            base.push_str(&format!(".{}", self.patch));

            if self.subpatch != 0 {
                base.push_str(&format!(".{}", self.subpatch));
            }
        }

        base
    }

    /// Convert to a remote tag, e.g. "4.1.1.1-stable"
    pub fn to_remote_str(&self) -> String {
        let mut base = self.to_version_string(false);
        base.push('-');
        base.push_str(&self.release_type);
        base
    }

    /// Convert to a string to be used for pinning versions, e.g. "4.1.0-stable".
    pub fn to_pinned_str(&self) -> String {
        let mut base = self.to_version_string(true);
        base.push('-');
        base.push_str(&self.release_type);
        base
    }

    /// Convert to a display-friendly string, e.g. "4.1.1.1-stable".
    pub fn to_display_str(&self) -> String {
        self.to_remote_str()
    }

    /// Returns if the version is a stable release.
    pub fn is_stable(&self) -> bool {
        self.release_type == "stable"
    }

    /// Returns an indeterminate version to be used in comparisons.
    pub fn to_indeterminate(&self) -> GodotVersion {
        GodotVersion {
            major: Some(self.major),
            minor: Some(self.minor),
            patch: Some(self.patch),
            subpatch: Some(self.subpatch),
            release_type: Some(self.release_type.clone()),
        }
    }
}

impl fmt::Display for GodotVersionDeterminate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_str())
    }
}

/// Get the priority of a pre-release identifier.
/// Higher numbers have higher priority (e.g., stable > rc > beta > dev).
pub fn get_pre_release_priority(pre_release: &str) -> i32 {
    if pre_release.is_empty() || pre_release.starts_with("stable") {
        4
    } else if pre_release.starts_with("rc") {
        3
    } else if pre_release.starts_with("beta") {
        2
    } else if pre_release.starts_with("dev") {
        1
    } else {
        0 // Unknown pre-release type (lowest priority)
    }
}

pub trait GodotVersionDeterminateVecExt {
    fn sort_by_version(&mut self);
}

impl GodotVersionDeterminateVecExt for Vec<GodotVersionDeterminate> {
    fn sort_by_version(&mut self) {
        self.sort_by(|a, b| {
            {
                a.major
                    .cmp(&b.major)
                    .then(a.minor.cmp(&b.minor))
                    .then(a.patch.cmp(&b.patch))
                    .then(a.subpatch.cmp(&b.subpatch))
                    .then(
                        get_pre_release_priority(a.release_type.as_str())
                            .cmp(&get_pre_release_priority(b.release_type.as_str())),
                    )
            }
            .reverse()
        });
    }
}

impl fmt::Display for GodotVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_str().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_install_str() {
        let gv = GodotVersion::from_install_str("4.1.1-rc1-csharp").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));

        let gv = GodotVersion::from_install_str("4.1.1-rc1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));
    }

    #[test]
    fn test_from_remote_str() {
        let gv = GodotVersion::from_install_str("3.5-stable").unwrap();
        assert_eq!(gv.major, Some(3));
        assert_eq!(gv.minor, Some(5));
        assert!(gv.is_stable());
    }

    #[test]
    fn test_from_match_str() {
        let gv = GodotVersion::from_match_str("4.1.1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));

        let gv = GodotVersion::from_match_str("stable").unwrap();
        assert_eq!(gv.release_type.as_deref(), Some("stable"));

        let gv = GodotVersion::from_match_str("4.1.1-rc1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));

        let gv = GodotVersion::from_match_str("4.1.1-csharp").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("csharp"));
    }

    #[test]
    fn test_to_remote_str() {
        let gv = GodotVersion::from_install_str("2.0.4.1-stable-csharp")
            .unwrap()
            .to_determinate();
        assert_eq!(gv.to_remote_str(), "2.0.4.1-stable");
    }

    #[test]
    fn test_sort_by_version() {
        let mut versions = vec![
            GodotVersion::from_install_str("3.0.0-stable")
                .unwrap()
                .to_determinate(),
            GodotVersion::from_install_str("4.1.1-rc2")
                .unwrap()
                .to_determinate(),
            GodotVersion::from_install_str("4.1.1-rc1")
                .unwrap()
                .to_determinate(),
        ];
        versions.sort_by_version();
        // We expect 4.1.1-rc2 > 4.1.1-rc1 > 3.0.0-stable
        assert_eq!(versions[0].release_type, "rc2");
        assert_eq!(versions[1].release_type, "rc1");
        assert_eq!(versions[2].major, 3);
    }

    #[test]
    fn test_validate_version_spec() {
        assert!(validate_version_spec("stable").is_ok());
        assert!(validate_version_spec("latest").is_ok());
        assert!(validate_version_spec("csharp:4.4").is_ok());
        assert!(validate_version_spec("4.1.1-rc1").is_ok());
        assert!(validate_version_spec("not-a-version").is_err());
    }

    #[test]
    fn test_get_pre_release_priority() {
        assert_eq!(get_pre_release_priority("stable"), 4);
        assert_eq!(get_pre_release_priority("rc1"), 3);
        assert_eq!(get_pre_release_priority("beta1"), 2);
        assert_eq!(get_pre_release_priority("dev1"), 1);
        assert_eq!(get_pre_release_priority("unknown"), 0);
    }

    #[test]
    fn test_to_version_string() {
        let gv = GodotVersion::from_install_str("2.0.4.1-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.4.1");

        let gv = GodotVersion::from_install_str("2.0.4-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.4");

        let gv = GodotVersion::from_install_str("2.0-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0");

        let gv = GodotVersion::from_install_str("2-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2");

        let gv = GodotVersion::from_install_str("2.0.0.0-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0");

        let gv = GodotVersion::from_install_str("2.0.1-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.1");

        let gv = GodotVersion::from_install_str("2.0.1-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.1");
    }

    macro_rules! test_matches_internal {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let version = GodotVersion::from_install_str($version).unwrap();
            let result = $pattern.matches(&version);
            assert_eq!(
                result, $expected,
                "Expected match result for {:?} and {:?} to be {}, got {}",
                $pattern, version, $expected, result
            );
        };
    }

    macro_rules! test_matches_case {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let pattern = GodotVersion::from_match_str($pattern).unwrap();
            test_matches_internal!(pattern, $version, $expected);
        };
    }

    macro_rules! test_matches_case_inst {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let pattern = GodotVersion::from_install_str($pattern).unwrap();
            test_matches_internal!(pattern, $version, $expected);
        };
    }

    macro_rules! test_matches_case_rem {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let pattern = GodotVersion::from_remote_str($pattern).unwrap();
            test_matches_internal!(pattern, $version, $expected);
        };
    }

    #[test]
    fn test_matches() {
        test_matches_case!("4", "4.1.1-rc1", true);
        test_matches_case!("4.1", "4.1.1-rc1", true);
        test_matches_case!("4.1.1", "4.1.1-rc1", true);
        test_matches_case_rem!("4.1.1-rc1", "4.1.1-rc1", true);
        test_matches_case_inst!("4.1.1-rc1", "4.1.1-rc1-csharp", true);
        test_matches_case_inst!("4.1.1-rc1-csharp", "4.1.1-rc1-csharp", true);
        test_matches_case_rem!("4.1.1-rc2", "4.1.1-rc1", false);
        test_matches_case!("stable", "4.1.1-stable", true);
    }

    #[test]
    fn test_conflicts_with_components() {
        let project = GodotVersion {
            major: Some(4),
            minor: Some(1),
            patch: None,
            subpatch: None,
            release_type: None,
        };

        let requested = GodotVersion::from_match_str("3.2").unwrap();
        assert!(project.conflicts_with(&requested));

        let compatible = GodotVersion::from_match_str("4.1.3").unwrap();
        assert!(!project.conflicts_with(&compatible));
    }

    #[test]
    fn test_version_spec_parse() {
        // Simple version.
        let spec = VersionSpec::parse("4.1.1").unwrap();
        assert!(spec.registry.is_none());
        assert!(spec.variant.is_none());
        assert!(matches!(spec.target, VersionTarget::Pattern(_)));

        // Variant prefix.
        let spec = VersionSpec::parse("csharp:4.1.1").unwrap();
        assert!(spec.registry.is_none());
        assert_eq!(spec.variant.as_deref(), Some("csharp"));

        // Registry and variant.
        let spec = VersionSpec::parse("my-reg/csharp:4.1.1").unwrap();
        assert_eq!(spec.registry.as_deref(), Some("my-reg"));
        assert_eq!(spec.variant.as_deref(), Some("csharp"));

        // Keyword.
        let spec = VersionSpec::parse("latest").unwrap();
        assert!(matches!(spec.target, VersionTarget::Keyword(ref k) if k == "latest"));

        let spec = VersionSpec::parse("stable").unwrap();
        assert!(matches!(spec.target, VersionTarget::Keyword(ref k) if k == "stable"));

        // Variant with keyword
        let spec = VersionSpec::parse("csharp:latest").unwrap();
        assert_eq!(spec.variant.as_deref(), Some("csharp"));
        assert!(matches!(spec.target, VersionTarget::Keyword(ref k) if k == "latest"));
    }

    #[test]
    fn test_install_dir_name() {
        let spec = VersionSpec::parse("4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        assert_eq!(
            install_dir_subpath(&ver_str, spec.variant.as_deref()),
            "default/4.1.1-stable"
        );

        let spec = VersionSpec::parse("csharp:4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        assert_eq!(
            install_dir_subpath(&ver_str, spec.variant.as_deref()),
            "csharp/4.1.1-stable"
        );
    }

    #[test]
    fn test_default_variant() {
        assert!(is_default_variant(None));
        assert!(is_default_variant(Some("default")));
        assert!(!is_default_variant(Some("csharp")));

        assert_eq!(normalize_variant(None), "default");
        assert_eq!(normalize_variant(Some("default")), "default");
        assert_eq!(normalize_variant(Some("csharp")), "csharp");

        let spec = VersionSpec::parse("default:4.1.1-stable").unwrap();
        assert_eq!(spec.variant.as_deref(), Some("default"));

        assert_eq!(
            install_dir_subpath("4.1.1-stable", None),
            "default/4.1.1-stable"
        );
        assert_eq!(
            install_dir_subpath("4.1.1-stable", Some("default")),
            "default/4.1.1-stable"
        );

        assert_eq!(pinned_str("4.1.1-stable", None), "default:4.1.1-stable");
        assert_eq!(
            pinned_str("4.1.1-stable", Some("default")),
            "default:4.1.1-stable"
        );
        assert_eq!(display_with_variant("4.1.1-stable", None), "4.1.1-stable");
        assert_eq!(
            display_with_variant("4.1.1-stable", Some("default")),
            "4.1.1-stable"
        );
        assert_eq!(
            display_with_variant("4.1.1-stable", Some("csharp")),
            "4.1.1-stable (csharp)"
        );
    }

    #[test]
    fn test_pinned_str_roundtrip() {
        let spec = VersionSpec::parse("csharp:4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        let pinned = pinned_str(&ver_str, spec.variant.as_deref());
        assert_eq!(pinned, "csharp:4.1.1-stable");

        let (parsed_variant, parsed_ver) = parse_pinned_str(&pinned);
        assert_eq!(parsed_variant.as_deref(), Some("csharp"));
        let gv = GodotVersion::from_install_str(&parsed_ver).unwrap();
        assert_eq!(gv.major, Some(4));

        let spec = VersionSpec::parse("4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        let pinned = pinned_str(&ver_str, None);
        assert_eq!(pinned, "default:4.1.1-stable");

        let (parsed_variant, parsed_ver) = parse_pinned_str(&pinned);
        assert_eq!(parsed_variant.as_deref(), Some("default"));
        let gv = GodotVersion::from_install_str(&parsed_ver).unwrap();
        assert_eq!(gv.major, Some(4));
    }

    #[test]
    fn test_legacy_pinned_str() {
        let gv = GodotVersion::from_install_str("4.3-stable")
            .unwrap()
            .to_determinate();
        assert_eq!(legacy_pinned_str(&gv, None), "4.3.0-stable");

        let gv = GodotVersion::from_install_str("4.3-stable")
            .unwrap()
            .to_determinate();
        assert_eq!(
            legacy_pinned_str(&gv, Some("csharp")),
            "4.3.0-stable-csharp"
        );

        let gv = GodotVersion::from_install_str("4.3.1-rc1")
            .unwrap()
            .to_determinate();
        assert_eq!(legacy_pinned_str(&gv, None), "4.3.1-rc1");
        assert_eq!(legacy_pinned_str(&gv, Some("csharp")), "4.3.1-rc1-csharp");

        let gv = GodotVersion::from_install_str("4.3-stable")
            .unwrap()
            .to_determinate();
        assert_eq!(legacy_pinned_str(&gv, Some("web")), "4.3.0-stable");
    }

    #[test]
    fn test_serialize_gdvm_toml() {
        let toml_str = serialize_gdvm_toml("csharp:4.3-stable");
        assert!(toml_str.contains("version = \"csharp:4.3-stable\""));
        assert!(toml_str.contains("[godot]"));
    }

    #[test]
    fn test_deserialize_gdvm_toml() {
        let input = "[godot]\nversion = \"csharp:4.3-stable\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert_eq!(parsed.godot.version, "csharp:4.3-stable");

        let input = "[godot]\nversion = \"4.3-stable\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert_eq!(parsed.godot.version, "4.3-stable");
    }

    #[test]
    fn test_gdvm_toml_roundtrip() {
        let specifier = "csharp:4.3-stable";
        let toml_str = serialize_gdvm_toml(specifier);
        let parsed = deserialize_gdvm_toml(&toml_str).unwrap();
        assert_eq!(parsed.godot.version, specifier);
    }

    #[test]
    fn test_gdvm_toml_ignores_unknown_keys() {
        let input = "[godot]\nversion = \"4.3-stable\"\n\n[registries.mybuilds]\nurl = \"https://example.com\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert_eq!(parsed.godot.version, "4.3-stable");
    }

    #[test]
    fn test_deserialize_gdvm_toml_invalid() {
        let input = "version = \"4.3-stable\"\n";
        assert!(deserialize_gdvm_toml(input).is_err());

        let input = "not valid toml {{{}";
        assert!(deserialize_gdvm_toml(input).is_err());
    }
}
