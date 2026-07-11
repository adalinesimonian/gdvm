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

use super::query::VersionQuery;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ResolvedVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub subpatch: u32,
    pub release_type: String,
}

impl From<VersionQuery> for ResolvedVersion {
    fn from(gv: VersionQuery) -> Self {
        gv.to_resolved()
    }
}

impl From<&VersionQuery> for ResolvedVersion {
    fn from(gv: &VersionQuery) -> Self {
        gv.to_resolved()
    }
}

impl Default for ResolvedVersion {
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

impl Clone for ResolvedVersion {
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

impl ResolvedVersion {
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
    pub fn to_query(&self) -> VersionQuery {
        VersionQuery {
            major: Some(self.major),
            minor: Some(self.minor),
            patch: Some(self.patch),
            subpatch: Some(self.subpatch),
            release_type: Some(self.release_type.clone()),
        }
    }
}

impl fmt::Display for ResolvedVersion {
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

impl Eq for ResolvedVersion {}

impl PartialOrd for ResolvedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders versions ascending: by numeric components, then by pre-release
/// priority, so that `sort` yields oldest first and reversing yields newest
/// first.
impl Ord for ResolvedVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then(self.subpatch.cmp(&other.subpatch))
            .then(
                get_pre_release_priority(self.release_type.as_str())
                    .cmp(&get_pre_release_priority(other.release_type.as_str())),
            )
    }
}

/// Compare two remote version tags so that the newest sorts first.
pub fn cmp_versions_newest_first(a: &str, b: &str) -> Ordering {
    let parsed = |s: &str| {
        VersionQuery::from_remote_str(s)
            .ok()
            .map(|gv| gv.to_resolved())
    };
    match (parsed(a), parsed(b)) {
        (Some(a), Some(b)) => a.cmp(&b).reverse(),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => b.cmp(a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pre_release_priority() {
        assert_eq!(get_pre_release_priority("stable"), 4);
        assert_eq!(get_pre_release_priority("rc1"), 3);
        assert_eq!(get_pre_release_priority("beta1"), 2);
        assert_eq!(get_pre_release_priority("dev1"), 1);
        assert_eq!(get_pre_release_priority("unknown"), 0);
    }

    #[test]
    fn test_sort_by_version() {
        let mut versions = [
            VersionQuery::from_install_str("3.0.0-stable")
                .unwrap()
                .to_resolved(),
            VersionQuery::from_install_str("4.1.1-rc2")
                .unwrap()
                .to_resolved(),
            VersionQuery::from_install_str("4.1.1-rc1")
                .unwrap()
                .to_resolved(),
        ];
        versions.sort_by(|a, b| b.cmp(a));
        // We expect 4.1.1-rc2 > 4.1.1-rc1 > 3.0.0-stable
        assert_eq!(versions[0].release_type, "rc2");
        assert_eq!(versions[1].release_type, "rc1");
        assert_eq!(versions[2].major, 3);
    }
}
