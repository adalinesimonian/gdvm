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
use super::resolved::ResolvedVersion;
use super::variant::Variant;
use crate::registry::normalize_registry;

/// A resolved selection of a determinate version.
#[derive(Debug, Clone)]
pub struct ResolvedSelection {
    pub version: ResolvedVersion,
    pub variant: Variant,
    pub registry: Option<String>,
}

/// An unresolved selection of a version, which may be indeterminate.
#[derive(Debug, Clone)]
pub struct QuerySelection {
    pub version: VersionQuery,
    pub variant: Option<String>,
    pub registry: Option<String>,
}

/// Display a version.
pub fn display_version(version_str: &str, variant: &Variant, registry: Option<&str>) -> String {
    let base = variant.decorate(version_str);
    match normalize_registry(registry) {
        Some(r) => format!("{r}/{base}"),
        None => base,
    }
}

/// Get the install directory subpath for a given version and variant.
pub fn install_dir_subpath(store_key: &str, version_str: &str, variant: &Variant) -> String {
    format!("{store_key}/{}/{version_str}", variant.as_str())
}

/// Build a pinned version string.
pub fn pinned_str(registry: Option<&str>, version_str: &str, variant: &Variant) -> String {
    let base = format!("{}:{version_str}", variant.as_str());
    match normalize_registry(registry) {
        Some(r) => format!("{r}/{base}"),
        None => base,
    }
}

/// Parse a pinned string back into `(registry, variant, version_string)`.
/// Supports `[registry/][variant:]version` and legacy `4.4.0-stable-csharp`.
pub fn parse_pinned_str(s: &str) -> (Option<String>, Option<String>, String) {
    // Optional leading registry. A registry segment never contains ':' (which would
    // indicate a `variant:version` pair that happened to be preceded by a slash).
    let (registry, rest) = match s.split_once('/') {
        Some((reg, rest)) if !reg.is_empty() && !reg.contains(':') => (Some(reg.to_string()), rest),
        _ => (None, s),
    };

    // New format: variant:version (variant doesn't start with digit).
    if let Some(pos) = rest.find(':') {
        let candidate_variant = &rest[..pos];
        if !candidate_variant.is_empty()
            && !candidate_variant
                .chars()
                .next()
                .unwrap_or('0')
                .is_ascii_digit()
        {
            return (
                registry,
                Some(candidate_variant.to_string()),
                rest[pos + 1..].to_string(),
            );
        }
    }

    // Legacy format: version-csharp.
    if let Some(version_part) = rest.strip_suffix("-csharp") {
        return (
            registry,
            Some("csharp".to_string()),
            version_part.to_string(),
        );
    }

    (registry, None, rest.to_string())
}

/// Build a legacy `.gdvmrc` pin string in the old pre-refactor format.
/// Produces the format that old gdvm versions understand:
/// `4.3.0-stable-csharp` (zero-padded, `-csharp` suffix) or `4.3.0-stable`.
pub fn legacy_pinned_str(gv: &ResolvedVersion, variant: &Variant) -> String {
    let mut base = gv.to_pinned_str();
    if variant.as_str() == "csharp" {
        base.push_str("-csharp");
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{VersionSpec, VersionTarget};

    #[test]
    fn test_install_dir_name() {
        let spec = VersionSpec::parse("4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        let variant = Variant::from_option(spec.variant.as_deref());
        assert_eq!(
            install_dir_subpath("official-abc123", &ver_str, &variant),
            "official-abc123/default/4.1.1-stable"
        );

        let spec = VersionSpec::parse("csharp:4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        let variant = Variant::from_option(spec.variant.as_deref());
        assert_eq!(
            install_dir_subpath("official-abc123", &ver_str, &variant),
            "official-abc123/csharp/4.1.1-stable"
        );

        assert_eq!(
            install_dir_subpath("mybuilds-deadbe", &ver_str, &variant),
            "mybuilds-deadbe/csharp/4.1.1-stable"
        );
    }

    #[test]
    fn test_pinned_str_roundtrip() {
        let spec = VersionSpec::parse("csharp:4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        let pinned = pinned_str(
            None,
            &ver_str,
            &Variant::from_option(spec.variant.as_deref()),
        );
        assert_eq!(pinned, "csharp:4.1.1-stable");

        let (parsed_registry, parsed_variant, parsed_ver) = parse_pinned_str(&pinned);
        assert!(parsed_registry.is_none());
        assert_eq!(parsed_variant.as_deref(), Some("csharp"));
        let gv = VersionQuery::from_install_str(&parsed_ver).unwrap();
        assert_eq!(gv.major, Some(4));

        let spec = VersionSpec::parse("4.1.1-stable").unwrap();
        let ver_str = match &spec.target {
            VersionTarget::Pattern(gv) => gv.to_remote_str().unwrap(),
            _ => panic!("expected pattern"),
        };
        let pinned = pinned_str(None, &ver_str, &Variant::default());
        assert_eq!(pinned, "default:4.1.1-stable");

        let (parsed_registry, parsed_variant, parsed_ver) = parse_pinned_str(&pinned);
        assert!(parsed_registry.is_none());
        assert_eq!(parsed_variant.as_deref(), Some("default"));
        let gv = VersionQuery::from_install_str(&parsed_ver).unwrap();
        assert_eq!(gv.major, Some(4));

        let pinned = pinned_str(
            Some("mybuilds"),
            "4.1.1-stable",
            &Variant::from_option(Some("csharp")),
        );
        assert_eq!(pinned, "mybuilds/csharp:4.1.1-stable");
        let (parsed_registry, parsed_variant, parsed_ver) = parse_pinned_str(&pinned);
        assert_eq!(parsed_registry.as_deref(), Some("mybuilds"));
        assert_eq!(parsed_variant.as_deref(), Some("csharp"));
        assert_eq!(parsed_ver, "4.1.1-stable");

        let (parsed_registry, parsed_variant, parsed_ver) = parse_pinned_str("4.3.0-stable-csharp");
        assert!(parsed_registry.is_none());
        assert_eq!(parsed_variant.as_deref(), Some("csharp"));
        assert_eq!(parsed_ver, "4.3.0-stable");
    }

    #[test]
    fn test_legacy_pinned_str() {
        let gv = VersionQuery::from_install_str("4.3-stable")
            .unwrap()
            .to_resolved();
        assert_eq!(legacy_pinned_str(&gv, &Variant::default()), "4.3.0-stable");

        let gv = VersionQuery::from_install_str("4.3-stable")
            .unwrap()
            .to_resolved();
        assert_eq!(
            legacy_pinned_str(&gv, &Variant::from_option(Some("csharp"))),
            "4.3.0-stable-csharp"
        );

        let gv = VersionQuery::from_install_str("4.3.1-rc1")
            .unwrap()
            .to_resolved();
        assert_eq!(legacy_pinned_str(&gv, &Variant::default()), "4.3.1-rc1");
        assert_eq!(
            legacy_pinned_str(&gv, &Variant::from_option(Some("csharp"))),
            "4.3.1-rc1-csharp"
        );

        let gv = VersionQuery::from_install_str("4.3-stable")
            .unwrap()
            .to_resolved();
        assert_eq!(
            legacy_pinned_str(&gv, &Variant::from_option(Some("web"))),
            "4.3.0-stable"
        );
    }
}
