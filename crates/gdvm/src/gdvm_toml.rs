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

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// The gdvm pin file.
#[derive(Debug, Serialize, Deserialize)]
pub struct GdvmToml {
    /// The pinned Godot version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub godot: Option<GdvmTomlGodot>,
    /// Registries defined in the pin file, keyed by alias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registries: Option<HashMap<String, GdvmTomlRegistry>>,
    /// Preserve keys written by newer gdvm versions.
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

/// The `[godot]` section of `gdvm.toml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GdvmTomlGodot {
    pub version: String,
}

/// A `[registries.<alias>]` entry in `gdvm.toml`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GdvmTomlRegistry {
    pub url: String,
}

/// Serialize a `GdvmToml` to a TOML string.
pub fn serialize_gdvm_toml(version_specifier: &str) -> String {
    let gdvm_toml = GdvmToml {
        godot: Some(GdvmTomlGodot {
            version: version_specifier.to_string(),
        }),
        registries: None,
        extra: BTreeMap::new(),
    };
    toml::to_string(&gdvm_toml).expect("GdvmToml serialization should never fail")
}

/// Deserialize a `GdvmToml` from a TOML string.
pub fn deserialize_gdvm_toml(contents: &str) -> Result<GdvmToml, toml::de::Error> {
    toml::from_str(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(parsed.godot.unwrap().version, "csharp:4.3-stable");

        let input = "[godot]\nversion = \"4.3-stable\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert_eq!(parsed.godot.unwrap().version, "4.3-stable");
    }

    #[test]
    fn test_gdvm_toml_registries_only_without_pin() {
        let input = "[registries.mybuilds]\nurl = \"https://example.com/godot\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert!(parsed.godot.is_none());
        let registries = parsed.registries.expect("registries present");
        assert_eq!(
            registries.get("mybuilds").map(|r| r.url.as_str()),
            Some("https://example.com/godot")
        );
    }

    #[test]
    fn test_gdvm_toml_roundtrip() {
        let specifier = "csharp:4.3-stable";
        let toml_str = serialize_gdvm_toml(specifier);
        let parsed = deserialize_gdvm_toml(&toml_str).unwrap();
        assert_eq!(parsed.godot.unwrap().version, specifier);
    }

    #[test]
    fn test_gdvm_toml_ignores_unknown_keys() {
        let input = "[godot]\nversion = \"4.3-stable\"\n\n[registries.mybuilds]\nurl = \"https://example.com\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert_eq!(parsed.godot.unwrap().version, "4.3-stable");
    }

    #[test]
    fn test_gdvm_toml_parses_project_registries() {
        let input = "[godot]\nversion = \"mybuilds/4.3-stable\"\n\n[registries.mybuilds]\nurl = \"https://example.com/godot\"\n";
        let parsed = deserialize_gdvm_toml(input).unwrap();
        assert_eq!(
            parsed.godot.as_ref().map(|g| g.version.as_str()),
            Some("mybuilds/4.3-stable")
        );
        let registries = parsed.registries.expect("registries present");
        assert_eq!(
            registries.get("mybuilds").map(|r| r.url.as_str()),
            Some("https://example.com/godot")
        );
    }

    #[test]
    fn test_gdvm_toml_no_registries_section_omitted_on_serialize() {
        let toml_str = serialize_gdvm_toml("4.3-stable");
        assert!(!toml_str.contains("registries"));
    }

    #[test]
    fn test_deserialize_gdvm_toml_invalid() {
        let input = "not valid toml {{{}";
        assert!(deserialize_gdvm_toml(input).is_err());

        let input = "[godot]\nversion = 4\n";
        assert!(deserialize_gdvm_toml(input).is_err());
    }
}
