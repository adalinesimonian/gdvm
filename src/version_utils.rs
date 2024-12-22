use semver::Version;
use std::fmt;

/// Normalize a version string to include the patch version.
/// If the version is in the format "major.minor-identifier", it converts it to "major.minor.0-identifier".
pub fn normalize_version(s: &str) -> Option<Version> {
    // Remove '-csharp' suffix if present
    let s = if s.ends_with("-csharp") {
        &s[..s.len() - 8]
    } else {
        s
    };
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() == 2 {
        let version_part = parts[0];
        let prerelease = parts[1];
        match version_part.matches('.').count() {
            3 => {
                // Handle extra patch version by merging the extra segment into prerelease
                // Blame 2.0.4.1 for this mess (what were you all smoking that day, Godot devs?)
                let segments: Vec<&str> = version_part.split('.').collect();
                let version_with_prerelease = format!(
                    "{}.{}.{}-{}.{}",
                    segments[0], segments[1], segments[2], prerelease, segments[3]
                );
                Version::parse(&version_with_prerelease).ok()
            }
            2 => Version::parse(s).ok(),
            1 => {
                // Append ".0" to include the patch version
                let version_with_patch = format!("{}.0-{}", version_part, prerelease);
                Version::parse(&version_with_patch).ok()
            }
            _ => None, // Invalid version format
        }
    } else {
        // No pre-release identifier; attempt to parse directly
        Version::parse(s).ok()
    }
}

/// Get the priority of a pre-release identifier.
/// Lower numbers have higher priority (e.g., stable > rc > beta > dev).
pub fn get_pre_release_priority(pre_release: &str) -> i32 {
    if pre_release.is_empty() || pre_release.starts_with("stable") {
        0
    } else if pre_release.starts_with("rc") {
        1
    } else if pre_release.starts_with("beta") {
        2
    } else if pre_release.starts_with("dev") {
        3
    } else {
        4 // Unknown pre-release type (lowest priority)
    }
}

/// Sorts the releases in descending order based on version and pre-release priority.
pub fn sort_releases(releases: &mut Vec<String>) {
    releases.sort_by(|a, b| {
        let va = normalize_version(a).unwrap_or_else(|| Version::new(0, 0, 0));
        let vb = normalize_version(b).unwrap_or_else(|| Version::new(0, 0, 0));

        // Compare by major, minor, and patch versions first
        match va
            .major
            .cmp(&vb.major)
            .then(va.minor.cmp(&vb.minor))
            .then(va.patch.cmp(&vb.patch))
        {
            std::cmp::Ordering::Equal => {
                // If major, minor, and patch are equal, compare pre-release priorities
                let pa = get_pre_release_priority(va.pre.as_str());
                let pb = get_pre_release_priority(vb.pre.as_str());
                pa.cmp(&pb)
            }
            other => other.reverse(), // Reverse to have descending order
        }
    });
}

/// Gets a release tag for a given version and branch.
pub fn build_release_tag(version: &str, branch: &GodotBranch) -> String {
    match branch {
        GodotBranch::Stable => format!("{}-stable", version),
        GodotBranch::PreRelease(pr) => format!("{}-{}", version, pr),
    }
}

/// Given an installed version, replaces a `-csharp` suffix with ` (C#)`
pub fn friendly_installed_version(version: &str) -> String {
    if version.ends_with("-csharp") {
        format!("{} (C#)", &version[..version.len() - 7])
    } else {
        version.to_string()
    }
}

#[derive(Debug)]
pub struct GodotVersion {
    pub version: String,
    pub branch: GodotBranch,
    pub is_csharp: bool,
}

#[derive(Debug)]
pub enum GodotBranch {
    Stable,
    PreRelease(String),
}

impl fmt::Display for GodotVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_csharp {
            write!(f, "{} (C#)", build_release_tag(&self.version, &self.branch))
        } else {
            write!(f, "{}", build_release_tag(&self.version, &self.branch))
        }
    }
}
