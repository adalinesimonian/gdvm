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

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Config;
use crate::terr;

/// A machine-level registry config.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RegistryConfig {
    pub url: String,
}

/// Validate a registry alias.
pub fn validate_registry_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == crate::registry::OFFICIAL_REGISTRY
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains(':')
    {
        return Err(terr!("error-registry-invalid-name", name = name).into());
    }
    Ok(())
}

impl Config {
    /// Get registries as `(name, url)` pairs.
    pub fn registry_pairs(&self) -> Vec<(String, String)> {
        self.registries
            .iter()
            .map(|(name, cfg)| (name.clone(), cfg.url.clone()))
            .collect()
    }

    /// The URL configured for a registry alias, if present.
    pub fn registry_url(&self, name: &str) -> Option<&str> {
        self.registries.get(name).map(|cfg| cfg.url.as_str())
    }

    /// Store a registry in config.
    pub fn add_registry(&mut self, name: &str, url: &str) -> Result<()> {
        validate_registry_name(name)?;
        crate::registry::RegistryUrl::parse(url)?;
        self.registries.insert(
            name.to_string(),
            RegistryConfig {
                url: url.to_string(),
            },
        );
        Ok(())
    }

    /// Remove a registry from config.
    pub fn remove_registry(&mut self, name: &str) -> Result<()> {
        if self.registries.remove(name).is_none() {
            return Err(terr!("error-registry-not-configured", name = name).into());
        }
        Ok(())
    }

    /// True when the registry at `url` has been trusted by the user.
    pub fn is_registry_trusted(&self, url: &str) -> bool {
        self.trusted_registries.iter().any(|u| u == url)
    }

    /// Trust the registry at `url`.
    pub fn trust_registry(&mut self, url: &str) {
        if !self.is_registry_trusted(url) {
            self.trusted_registries.push(url.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_add_remove() {
        let mut cfg = Config::default();

        cfg.add_registry("mybuilds", "https://example.com/godot")
            .unwrap();
        assert_eq!(
            cfg.registry_url("mybuilds"),
            Some("https://example.com/godot")
        );
        assert_eq!(
            cfg.registry_pairs(),
            vec![(
                "mybuilds".to_string(),
                "https://example.com/godot".to_string()
            )]
        );

        cfg.add_registry("mybuilds", "https://example.com/godot2")
            .unwrap();
        assert_eq!(
            cfg.registry_url("mybuilds"),
            Some("https://example.com/godot2")
        );

        cfg.remove_registry("mybuilds").unwrap();
        assert!(cfg.registry_url("mybuilds").is_none());
        assert!(cfg.remove_registry("mybuilds").is_err());
    }

    #[test]
    fn test_registry_validation() {
        let mut cfg = Config::default();
        assert!(cfg.add_registry("official", "https://example.com").is_err());
        assert!(cfg.add_registry("a/b", "https://example.com").is_err());
        assert!(cfg.add_registry("..", "https://example.com").is_err());
        assert!(cfg.add_registry("a:b", "https://example.com").is_err());
        assert!(cfg.add_registry("ok", "ftp://example.com").is_err());
        assert!(cfg.add_registry("ok", "https://example.com").is_ok());
        assert!(cfg.add_registry("local", "file:///tmp/reg").is_ok());
    }

    #[test]
    fn test_trusting_a_registry_is_idempotent() {
        let mut cfg = Config::default();

        assert!(!cfg.is_registry_trusted("https://example.com/godot"));

        cfg.trust_registry("https://example.com/godot");
        cfg.trust_registry("https://example.com/godot");

        assert!(cfg.is_registry_trusted("https://example.com/godot"));
        assert_eq!(cfg.trusted_registries.len(), 1);
    }

    #[test]
    fn test_registries_toml_roundtrip() {
        let mut cfg = Config::default();
        cfg.add_registry("mybuilds", "https://example.com/godot")
            .unwrap();

        let parsed: Config = toml::from_str(&toml::to_string(&cfg).unwrap()).unwrap();

        assert_eq!(
            parsed.registry_url("mybuilds"),
            Some("https://example.com/godot")
        );
    }
}
