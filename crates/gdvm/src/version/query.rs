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

use std::fmt;

use super::resolved::ResolvedVersion;
use crate::t;

#[derive(Debug, Default)]
pub struct VersionQuery {
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub subpatch: Option<u32>,
    pub release_type: Option<String>,
}

impl From<ResolvedVersion> for VersionQuery {
    fn from(gvd: ResolvedVersion) -> Self {
        gvd.to_query()
    }
}

impl From<&ResolvedVersion> for VersionQuery {
    fn from(gvd: &ResolvedVersion) -> Self {
        gvd.to_query()
    }
}

impl Clone for VersionQuery {
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

/// Check if a release tag pattern matches a release tag.
pub fn release_pattern_matches(pattern: &str, tag: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => tag.starts_with(prefix),
        None => tag == pattern,
    }
}

impl VersionQuery {
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
    pub(super) fn to_version_string(&self) -> Option<String> {
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
        let (version_part, pre_release) = match raw.find('-') {
            Some(index) => (&raw[..index], Some(raw[index + 1..].to_string())),
            None => (raw, None),
        };

        if let Some(pre) = &pre_release {
            if !pre
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+' | '*'))
            {
                return Err(anyhow::anyhow!(t!(
                    "error-unrecognized-version-format",
                    input = raw
                )));
            }

            let wildcards = pre.matches('*').count();
            if wildcards > 1 || (wildcards == 1 && !pre.ends_with('*')) {
                return Err(anyhow::anyhow!(t!("error-wildcard-position", input = raw)));
            }
        }

        // Parse major/minor/patch from version_part, including special 2.0.4.1
        let pieces: Vec<&str> = version_part.split('.').collect();
        let component = |i: usize| -> Result<u32, anyhow::Error> {
            pieces[i]
                .parse()
                .map_err(|_| anyhow::anyhow!(t!("error-unrecognized-version-format", input = raw)))
        };
        let (major, minor, patch, subpatch) = match pieces.len() {
            0 => (None, None, None, None),
            1 => (Some(component(0)?), None, None, None),
            2 => (Some(component(0)?), Some(component(1)?), None, None),
            3 => (
                Some(component(0)?),
                Some(component(1)?),
                Some(component(2)?),
                None,
            ),
            4 => (
                Some(component(0)?),
                Some(component(1)?),
                Some(component(2)?),
                Some(component(3)?),
            ),
            _ => {
                return Err(anyhow::anyhow!(t!(
                    "error-unrecognized-version-format",
                    input = raw
                )));
            }
        };

        Ok(VersionQuery {
            major,
            minor,
            patch,
            subpatch,
            release_type: pre_release,
        })
    }

    /// Creates a VersionQuery from a match string, e.g. "stable", "4", "3.5"
    /// Match strings are just like remote tags, but can also be "stable".
    pub fn from_match_str(s: &str) -> Result<Self, anyhow::Error> {
        if s == "stable" {
            return Ok(VersionQuery {
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
    pub fn to_resolved(&self) -> ResolvedVersion {
        ResolvedVersion {
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
        T: Into<VersionQuery> + Clone,
    {
        let other: VersionQuery = other.clone().into();

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
            && other
                .release_type
                .is_some_and(|x| !release_pattern_matches(release_type, &x))
        {
            return false;
        }
        true
    }

    /// Get whether the release tag has a wildcard.
    pub fn has_release_wildcard(&self) -> bool {
        self.release_type
            .as_deref()
            .is_some_and(|rt| rt.ends_with('*'))
    }

    /// Get whether the release requested is explicitly requesting a pre-release.
    pub fn is_explicit_prerelease(&self) -> bool {
        self.release_type
            .as_deref()
            .is_some_and(|rt| !rt.is_empty() && rt != "stable" && rt != "stable*")
    }

    /// Returns true when all specified project constraints conflict with the requested version.
    /// Missing components in either version act as wildcards.
    pub fn conflicts_with(&self, requested: &VersionQuery) -> bool {
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

impl fmt::Display for VersionQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_str().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_install_str() {
        let gv = VersionQuery::from_install_str("4.1.1-rc1-csharp").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));

        let gv = VersionQuery::from_install_str("4.1.1-rc1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));
    }

    #[test]
    fn test_from_remote_str() {
        let gv = VersionQuery::from_install_str("3.5-stable").unwrap();
        assert_eq!(gv.major, Some(3));
        assert_eq!(gv.minor, Some(5));
        assert!(gv.is_stable());
    }

    #[test]
    fn test_rejects_path_unsafe_release_types() {
        for tag in [
            "4.4-x/../../evil",
            "4.4-x\\evil",
            "4.4-rc1/extra",
            "4.4-rc1\0",
        ] {
            assert!(
                VersionQuery::from_remote_str(tag).is_err(),
                "tag {tag:?} must be rejected"
            );
        }

        for tag in [
            "4.4-stable",
            "4.4-rc1",
            "3.2-alpha0-unofficial",
            "4.4-pre.1",
        ] {
            assert!(
                VersionQuery::from_remote_str(tag).is_ok(),
                "tag {tag:?} must parse"
            );
        }
    }

    #[test]
    fn test_multi_part_release_type() {
        let gv = VersionQuery::from_remote_str("3.2-alpha0-unofficial").unwrap();
        assert_eq!(gv.major, Some(3));
        assert_eq!(gv.minor, Some(2));
        assert_eq!(gv.patch, None);
        assert_eq!(gv.release_type.as_deref(), Some("alpha0-unofficial"));
        assert!(!gv.is_stable());

        let determinate = gv.to_resolved();
        assert_eq!(determinate.to_remote_str(), "3.2-alpha0-unofficial");

        let from_install = VersionQuery::from_install_str("3.2-alpha0-unofficial").unwrap();
        assert_eq!(
            from_install.release_type.as_deref(),
            Some("alpha0-unofficial")
        );
    }

    #[test]
    fn test_from_match_str() {
        let gv = VersionQuery::from_match_str("4.1.1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));

        let gv = VersionQuery::from_match_str("stable").unwrap();
        assert_eq!(gv.release_type.as_deref(), Some("stable"));

        let gv = VersionQuery::from_match_str("4.1.1-rc1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));

        let gv = VersionQuery::from_match_str("4.1.1-csharp").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("csharp"));
    }

    #[test]
    fn test_to_remote_str() {
        let gv = VersionQuery::from_install_str("2.0.4.1-stable-csharp")
            .unwrap()
            .to_resolved();
        assert_eq!(gv.to_remote_str(), "2.0.4.1-stable");
    }

    #[test]
    fn test_to_version_string() {
        let gv = VersionQuery::from_install_str("2.0.4.1-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.4.1");

        let gv = VersionQuery::from_install_str("2.0.4-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.4");

        let gv = VersionQuery::from_install_str("2.0-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0");

        let gv = VersionQuery::from_install_str("2-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2");

        let gv = VersionQuery::from_install_str("2.0.0.0-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0");

        let gv = VersionQuery::from_install_str("2.0.1-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.1");

        let gv = VersionQuery::from_install_str("2.0.1-stable-csharp").unwrap();
        assert_eq!(gv.to_version_string().unwrap(), "2.0.1");
    }

    macro_rules! test_matches_internal {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let version = VersionQuery::from_install_str($version).unwrap();
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
            let pattern = VersionQuery::from_match_str($pattern).unwrap();
            test_matches_internal!(pattern, $version, $expected);
        };
    }

    macro_rules! test_matches_case_inst {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let pattern = VersionQuery::from_install_str($pattern).unwrap();
            test_matches_internal!(pattern, $version, $expected);
        };
    }

    macro_rules! test_matches_case_rem {
        ($pattern:expr, $version:expr, $expected:expr) => {
            let pattern = VersionQuery::from_remote_str($pattern).unwrap();
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
    fn wildcard_matches_release_type() {
        test_matches_case_rem!("4.7-dev*", "4.7-dev", true);
        test_matches_case_rem!("4.7-dev*", "4.7-dev1", true);
        test_matches_case_rem!("4.7-dev*", "4.7-dev12", true);
        test_matches_case_rem!("4.7-rc*", "4.7-rc2", true);
        test_matches_case_rem!("4.7-dev*", "4.7-beta1", false);
        test_matches_case_rem!("4.7-dev*", "4.8-dev1", false);
        test_matches_case_rem!("4.7-*", "4.7-stable", true);
        test_matches_case_rem!("4.7-*", "4.7-dev3", true);
    }

    #[test]
    fn exact_tags_never_prefix_match() {
        test_matches_case_rem!("4.7-dev1", "4.7-dev12", false);
        test_matches_case_rem!("4.7-dev", "4.7-dev1", false);
        test_matches_case_rem!("4.7-dev", "4.7-dev", true);
        test_matches_case_rem!("4.7-abc123", "4.7-abc123", true);
        test_matches_case_rem!("4.7-abc12", "4.7-abc123", false);
    }

    #[test]
    fn wildcard_only_parses_at_the_end() {
        assert!(VersionQuery::from_remote_str("4.7-dev*").is_ok());
        assert!(VersionQuery::from_remote_str("4.7-*").is_ok());
        assert!(VersionQuery::from_remote_str("4.7-d*v").is_err());
        assert!(VersionQuery::from_remote_str("4.7-dev**").is_err());
        assert!(VersionQuery::from_remote_str("4.7-*dev").is_err());
    }

    #[test]
    fn wildcard_and_prerelease_helpers() {
        let q = |s: &str| VersionQuery::from_remote_str(s).unwrap();
        assert!(q("4.7-dev*").has_release_wildcard());
        assert!(!q("4.7-dev").has_release_wildcard());
        assert!(q("4.7-dev*").is_explicit_prerelease());
        assert!(q("4.7-rc1").is_explicit_prerelease());
        assert!(!q("4.7-stable").is_explicit_prerelease());
        assert!(!q("4.7-stable*").is_explicit_prerelease());
        assert!(!q("4.7").is_explicit_prerelease());
    }

    #[test]
    fn test_conflicts_with_components() {
        let project = VersionQuery {
            major: Some(4),
            minor: Some(1),
            patch: None,
            subpatch: None,
            release_type: None,
        };

        let requested = VersionQuery::from_match_str("3.2").unwrap();
        assert!(project.conflicts_with(&requested));

        let compatible = VersionQuery::from_match_str("4.1.3").unwrap();
        assert!(!project.conflicts_with(&compatible));
    }
}
