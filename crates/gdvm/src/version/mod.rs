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

mod query;
mod resolved;
mod selection;
mod spec;
mod variant;

pub use query::VersionQuery;
pub use resolved::{ResolvedVersion, cmp_versions_newest_first, split_release_tag};
pub use selection::{
    QuerySelection, ResolvedSelection, display_version, install_dir_subpath, legacy_pinned_str,
    parse_pinned_str, pinned_str,
};
pub use spec::{VersionSpec, VersionTarget, validate_version_spec};
pub use variant::Variant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant() {
        assert!(Variant::from_option(None).is_default());
        assert!(Variant::from_option(Some("default")).is_default());
        assert!(!Variant::from_option(Some("csharp")).is_default());

        assert_eq!(Variant::from_option(None).as_str(), "default");
        assert_eq!(Variant::from_option(Some("default")).as_str(), "default");
        assert_eq!(Variant::from_option(Some("csharp")).as_str(), "csharp");
        assert_eq!(Variant::default().as_str(), "default");

        let spec = VersionSpec::parse("default:4.1.1-stable").unwrap();
        assert_eq!(spec.variant.as_deref(), Some("default"));

        let default = Variant::default();
        assert_eq!(
            install_dir_subpath("official-abc123", "4.1.1-stable", &default),
            "official-abc123/default/4.1.1-stable"
        );
        assert_eq!(
            pinned_str(None, "4.1.1-stable", &default),
            "default:4.1.1-stable"
        );

        let v411 = ResolvedVersion {
            major: 4,
            minor: 1,
            patch: 1,
            subpatch: 0,
            release_type: "stable".to_string(),
        };
        assert_eq!(display_version(&v411, &default, None), "4.1.1-stable");
        assert_eq!(
            display_version(&v411, &Variant::from_option(Some("csharp")), None),
            "csharp:4.1.1-stable"
        );
    }
}
