use regex::Regex;
use std::fmt;

#[derive(Debug, Default)]
pub struct GodotVersion {
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub subpatch: Option<u32>,
    pub release_type: Option<String>,
    pub is_csharp: Option<bool>,
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
            is_csharp: self.is_csharp,
        }
    }
}

#[derive(Debug)]
pub struct GodotVersionDeterminate {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub subpatch: u32,
    pub release_type: String,
    pub is_csharp: Option<bool>,
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
            is_csharp: None,
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
            is_csharp: self.is_csharp,
        }
    }
}

impl GodotVersion {
    /// Parse an installed version folder name (e.g. "4.1.1-rc1-csharp", "2.0.4.1-stable", "3-csharp").
    pub fn from_install_str(s: &str) -> Result<Self, anyhow::Error> {
        Self::parse_with_csharp_and_pre_release(s, false)
    }

    /// Parse a remote version tag (e.g. "4.1.stable", "3.rc1", "2.0.4.1.stable").
    pub fn from_remote_str(s: &str, is_csharp: Option<bool>) -> Result<Self, anyhow::Error> {
        let mut gv = Self::parse_with_csharp_and_pre_release(s, true)?;
        gv.is_csharp = is_csharp;
        Ok(gv)
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

    // /// Convert to an install folder name, e.g. "4.1.1-rc1-csharp"
    // pub fn to_install_str(&self) -> Option<String> {
    //     let mut base = self.to_remote_str()?;
    //     if self.is_csharp == Some(true) {
    //         base.push_str("-csharp");
    //     }
    //     Some(base)
    // }

    /// Convert to a display-friendly string, e.g. "4.1.1-rc1 (C#)"
    pub fn to_display_str(&self) -> Option<String> {
        let mut base = self.to_remote_str()?;
        if self.is_csharp == Some(true) {
            base.push_str(" (C#)");
        }
        Some(base)
    }

    fn parse_with_csharp_and_pre_release(raw: &str, remote: bool) -> Result<Self, anyhow::Error> {
        // Check for csharp suffix or parse is_csharp from call
        let (without_csharp, is_csharp) = if raw.ends_with("-csharp") && !remote {
            (&raw[..raw.len() - 7], Some(true))
        } else {
            (raw, if remote { None } else { Some(false) })
        };

        // Read the pre-release identifier, if present (e.g. "rc1" from "4.1.1-rc1")
        let (version_part, pre_release) = match without_csharp.rfind('-') {
            Some(index) => (
                &without_csharp[..index],
                Some(without_csharp[index + 1..].to_string()),
            ),
            None => (without_csharp, None),
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
            _ => return Err(anyhow::anyhow!("Unrecognized version format: {}", raw)),
        };

        Ok(GodotVersion {
            major,
            minor,
            patch,
            subpatch,
            release_type: pre_release,
            is_csharp,
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

        Self::from_remote_str(s, None)
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
            is_csharp: self.is_csharp,
        }
    }

    /// Checks if the version matches another version. Version parts that are None are interpreted
    /// as wildcards.
    pub fn matches<T>(&self, other: &T) -> bool
    where
        T: Into<GodotVersion> + Clone,
    {
        let other: GodotVersion = other.clone().into();

        if let Some(major) = self.major {
            if other.major.is_some_and(|x| x != major) {
                return false;
            }
        }
        if let Some(minor) = self.minor {
            if other.minor.is_some_and(|x| x != minor) {
                return false;
            }
        }
        if let Some(patch) = self.patch {
            if other.patch.is_some_and(|x| x != patch) {
                return false;
            }
        }
        if let Some(subpatch) = self.subpatch {
            if other.subpatch.is_some_and(|x| x != subpatch) {
                return false;
            }
        }
        if let Some(release_type) = &self.release_type {
            if other.release_type.is_some_and(|x| &x != release_type) {
                return false;
            }
        }
        if let Some(is_csharp) = self.is_csharp {
            if other.is_csharp.is_some_and(|x| x != is_csharp) {
                return false;
            }
        }
        true
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

    /// Convert to an install folder name, e.g. "4.1.1.1-csharp"
    pub fn to_install_str(&self) -> String {
        let mut base = self.to_remote_str();
        if self.is_csharp.unwrap_or(false) {
            base.push_str("-csharp");
        }
        base
    }

    /// Convert to a string to be used for pinning versions, e.g. "4.1.0-stable-csharp"
    pub fn to_pinned_str(&self) -> String {
        let mut base = self.to_version_string(true);
        base.push('-');
        base.push_str(&self.release_type);
        if self.is_csharp.unwrap_or(false) {
            base.push_str("-csharp");
        }
        base
    }

    /// Convert to a display-friendly string, e.g. "4.1.1.1 (C#)"
    pub fn to_display_str(&self) -> String {
        let mut base = self.to_remote_str();
        if self.is_csharp.unwrap_or(false) {
            base.push_str(" (C#)");
        }
        base
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
            is_csharp: self.is_csharp,
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

// pub trait GodotVersionVecExt {
//     fn sort_by_version(&mut self);
// }

// impl GodotVersionVecExt for Vec<GodotVersion> {
//     fn sort_by_version(&mut self) {
//         self.sort_by(|a, b| {
//             {
//                 a.major
//                     .unwrap_or(0)
//                     .cmp(&b.major.unwrap_or(0))
//                     .then(a.minor.unwrap_or(0).cmp(&b.minor.unwrap_or(0)))
//                     .then(a.patch.unwrap_or(0).cmp(&b.patch.unwrap_or(0)))
//                     .then(a.subpatch.unwrap_or(0).cmp(&b.subpatch.unwrap_or(0)))
//                     .then(
//                         get_pre_release_priority(a.release_type.as_deref().unwrap_or("")).cmp(
//                             &get_pre_release_priority(b.release_type.as_deref().unwrap_or("")),
//                         ),
//                     )
//             }
//             .reverse()
//         });
//     }
// }

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

pub fn validate_godot_version(s: &str) -> Result<String, String> {
    let re = Regex::new(r"^\d+(\.\d+){0,3}(-[A-Za-z0-9]+)?$").unwrap();
    if re.is_match(s) {
        Ok(s.to_string())
    } else {
        Err(String::from("error-invalid-godot-version"))
    }
}

pub fn validate_remote_version(s: &str) -> Result<String, String> {
    if s == "stable" {
        return Ok(s.to_string());
    }
    validate_godot_version(s).map_err(|_| String::from("error-invalid-remote-version"))
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
        assert_eq!(gv.is_csharp, Some(true));

        let gv = GodotVersion::from_install_str("4.1.1-rc1").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(1));
        assert_eq!(gv.patch, Some(1));
        assert_eq!(gv.release_type.as_deref(), Some("rc1"));
        assert_eq!(gv.is_csharp, Some(false));
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
    fn test_to_install_str() {
        let gv = GodotVersion::from_install_str("2.0.4.1-stable-csharp")
            .unwrap()
            .to_determinate();
        assert_eq!(gv.to_install_str(), "2.0.4.1-stable-csharp");
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
    fn test_validate_godot_version() {
        assert!(validate_godot_version("4.1.1-rc1").is_ok());
        assert!(validate_godot_version("not-a-version").is_err());
    }

    #[test]
    fn test_validate_remote_version() {
        assert!(validate_remote_version("stable").is_ok());
        assert!(validate_remote_version("4.1.1-rc1").is_ok());
        assert!(validate_remote_version("not-a-version").is_err());
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
            let pattern = GodotVersion::from_remote_str($pattern, None).unwrap();
            test_matches_internal!(pattern, $version, $expected);
        };
    }

    #[test]
    fn test_matches() {
        test_matches_case!("4", "4.1.1-rc1", true);
        test_matches_case!("4.1", "4.1.1-rc1", true);
        test_matches_case!("4.1.1", "4.1.1-rc1", true);
        test_matches_case_rem!("4.1.1-rc1", "4.1.1-rc1", true);
        test_matches_case_inst!("4.1.1-rc1", "4.1.1-rc1-csharp", false);
        test_matches_case_inst!("4.1.1-rc1-csharp", "4.1.1-rc1-csharp", true);
        test_matches_case_rem!("4.1.1-rc2", "4.1.1-rc1", false);
        test_matches_case!("stable", "4.1.1-stable", true);
    }
}
