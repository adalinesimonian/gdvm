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
use crate::terr;

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
    Pattern(VersionQuery),
}

impl VersionSpec {
    /// Parse a specifier string using the grammar `[registry/][variant:]version_or_keyword`.
    pub fn parse(input: &str) -> Result<Self, anyhow::Error> {
        let (registry, remainder) = match input.find('/') {
            Some(pos) => {
                let reg = &input[..pos];
                if reg.is_empty() {
                    return Err(terr!("error-spec-empty-registry", input = input).into());
                }
                (Some(reg.to_string()), &input[pos + 1..])
            }
            None => (None, input),
        };

        let (variant, version_str) = match remainder.find(':') {
            Some(pos) => {
                let var = &remainder[..pos];
                if var.is_empty() {
                    return Err(terr!("error-spec-empty-variant", input = input).into());
                }
                (Some(var.to_string()), &remainder[pos + 1..])
            }
            None => (None, remainder),
        };

        if version_str.is_empty() {
            return Err(terr!("error-spec-empty-version", input = input).into());
        }

        let target = if is_keyword(version_str) {
            VersionTarget::Keyword(normalize_keyword(version_str))
        } else {
            VersionTarget::Pattern(VersionQuery::from_remote_str(version_str)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_version_spec() {
        assert!(validate_version_spec("stable").is_ok());
        assert!(validate_version_spec("latest").is_ok());
        assert!(validate_version_spec("csharp:4.4").is_ok());
        assert!(validate_version_spec("4.1.1-rc1").is_ok());
        assert!(validate_version_spec("not-a-version").is_err());
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
}
