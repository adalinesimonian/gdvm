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

use crate::{eprintln_i18n, i18n, t};
use anyhow::{Result, anyhow};
use directories::BaseDirs;
use i18n::I18n;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A list of known configuration keys.
pub const KNOWN_KEYS: &[&str] = &[
    "github.token",
    "prune.max-age-days",
    "install.path",
];

/// The default maximum age, in days, before an unused asset becomes eligible
/// for pruning, unless `prune.max-age-days` is configured.
pub const DEFAULT_PRUNE_MAX_AGE_DAYS: u64 = 30;

/// A machine-level registry config.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryConfig {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub github_token: Option<String>,
    #[serde(default)]
    pub global_installs_location: Option<PathBuf>,
    /// Maximum age, in days, before an unused asset becomes eligible for
    /// pruning. When unset, `DEFAULT_PRUNE_MAX_AGE_DAYS` is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prune_max_age_days: Option<u64>,
    /// Registry configs keyed by alias.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub registries: HashMap<String, RegistryConfig>,
    /// Base URLs of unofficial registries the user has confirmed they trust,
    /// keyed by URL.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trusted_registries: Vec<String>,
}

/// Get/set operations for configuration keys.
pub trait ConfigOps {
    /// Retrieve the value of a configuration key, if set.
    fn get_value(&self, key: &str) -> Option<String>;

    /// Set a configuration key to the provided value.
    fn set_value(&mut self, key: &str, value: &str) -> Result<()>;

    /// Unset (remove) a configuration key's value.
    fn unset_value(&mut self, key: &str) -> Result<()>;

    /// Return true if the key is considered sensitive.
    fn is_sensitive_key(&self, key: &str) -> bool;

    /// List all set configuration keys as (key, value, is_sensitive) tuples.
    fn list_set_keys(&self) -> Vec<(String, String, bool)>;
}

impl ConfigOps for Config {
    fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "github.token" => self.github_token.clone(),
            "global.installs_location" => self
                .global_installs_location
                .clone()
                .map(|p| p.to_string_lossy().into_owned()),
            "prune.max-age-days" => self.prune_max_age_days.map(|d| d.to_string()),
            _ => None,
        }
    }

    fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "github.token" => {
                self.github_token = Some(value.to_string());
                Ok(())
            }
            "global.installs_location" => {
                self.global_installs_location = Some(PathBuf::from(value));
                Ok(())
            }
            "prune.max-age-days" => {
                let days: u64 = value
                    .parse()
                    .map_err(|_| anyhow!("Invalid value for {key}: {value} (expected a number)"))?;
                self.prune_max_age_days = Some(days);
                Ok(())
            }
            _ => Err(anyhow!("Unknown configuration key: {key}")),
        }
    }

    fn unset_value(&mut self, key: &str) -> Result<()> {
        match key {
            "github.token" => {
                self.github_token = None;
                Ok(())
            }
            "global.installs_location" => {
                self.global_installs_location = None;
                Ok(())
            }
            "prune.max-age-days" => {
                self.prune_max_age_days = None;
                Ok(())
            }
            _ => Err(anyhow!("Unknown configuration key: {key}")),
        }
    }

    fn is_sensitive_key(&self, key: &str) -> bool {
        matches!(key, "github.token")
    }

    fn list_set_keys(&self) -> Vec<(String, String, bool)> {
        let mut entries = Vec::new();
        if let Some(token) = self.github_token.as_ref() {
            entries.push(("github.token".to_string(), token.clone(), true));
        }
        if let Some(installs_location) = self.global_installs_location.as_ref() {
            entries.push((
                "global.installs_location".to_string(),
                installs_location.to_string_lossy().into_owned(),
                false,
            ));
        }
        if let Some(days) = self.prune_max_age_days {
            entries.push(("prune.max-age-days".to_string(), days.to_string(), false));
        }
        entries
    }
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
        return Err(anyhow!("Invalid registry name: {name}"));
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
            return Err(anyhow!("Registry '{name}' is not configured"));
        }
        Ok(())
    }

    /// True when the registry at `url` has been trusted by the user.
    pub fn is_registry_trusted(&self, url: &str) -> bool {
        self.trusted_registries.iter().any(|u| u == url)
    }

    /// The configured prune max age in days, or the default when unset.
    pub fn prune_max_age_days(&self) -> u64 {
        self.prune_max_age_days
            .unwrap_or(DEFAULT_PRUNE_MAX_AGE_DAYS)
    }

    /// Trust the registry at `url`.
    pub fn trust_registry(&mut self, url: &str) {
        if !self.is_registry_trusted(url) {
            self.trusted_registries.push(url.to_string());
        }
    }
}

pub fn get_home_dir(i18n: &I18n) -> Result<PathBuf> {
    #[cfg(feature = "integration-tests")]
    {
        // Override home directory for testing purposes.
        if let Ok(override_dir) = std::env::var("GDVM_TEST_HOME") {
            return Ok(PathBuf::from(override_dir));
        }
    }

    let base_dirs = BaseDirs::new().ok_or(anyhow!(t!(i18n, "error-find-user-dirs")))?;
    Ok(base_dirs.home_dir().to_path_buf())
}

impl Config {
    /// Load configuration from ~/.gdvm/config.toml.
    pub fn load(i18n: &I18n) -> Result<Self> {
        let home = get_home_dir(i18n)?;
        let config_path = home.join(".gdvm").join("config.toml");
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path).expect("Failed to read config.toml");
            match toml::from_str(&contents) {
                Ok(config) => Ok(config),
                Err(e) => {
                    eprintln_i18n!(i18n, "error-parse-config", error = e.to_string());
                    eprintln_i18n!(i18n, "error-parse-config-using-default");
                    Ok(Self::default())
                }
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to ~/.gdvm/config.toml.
    pub fn save(&self, i18n: &I18n) -> Result<()> {
        let home = get_home_dir(i18n)?;
        let config_dir = home.join(".gdvm");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        let toml_str = toml::to_string(self).expect("Failed to serialize config");
        fs::write(config_path, toml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_unset_list() {
        let mut cfg = Config::default();
        assert!(cfg.get_value("github.token").is_none());
        assert!(cfg.set_value("github.token", "abc").is_ok());
        assert_eq!(cfg.get_value("github.token"), Some("abc".to_string()));
        assert!(cfg.is_sensitive_key("github.token"));
        let listed = cfg.list_set_keys();
        assert_eq!(
            listed,
            vec![("github.token".to_string(), "abc".to_string(), true)]
        );
        assert!(cfg.unset_value("github.token").is_ok());
        assert!(cfg.get_value("github.token").is_none());
        assert!(cfg.set_value("unknown", "val").is_err());
        assert!(cfg.unset_value("unknown").is_err());
    }

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
    fn test_prune_max_age_days_get_set_unset_and_default() {
        let mut cfg = Config::default();

        assert!(cfg.get_value("prune.max-age-days").is_none());
        assert_eq!(cfg.prune_max_age_days(), DEFAULT_PRUNE_MAX_AGE_DAYS);

        assert!(cfg.set_value("prune.max-age-days", "7").is_ok());
        assert_eq!(cfg.get_value("prune.max-age-days"), Some("7".to_string()));
        assert_eq!(cfg.prune_max_age_days(), 7);
        assert!(!cfg.is_sensitive_key("prune.max-age-days"));
        assert!(cfg.list_set_keys().contains(&(
            "prune.max-age-days".to_string(),
            "7".to_string(),
            false
        )));

        assert!(cfg.set_value("prune.max-age-days", "soon").is_err());

        assert!(cfg.unset_value("prune.max-age-days").is_ok());
        assert!(cfg.get_value("prune.max-age-days").is_none());
        assert_eq!(cfg.prune_max_age_days(), DEFAULT_PRUNE_MAX_AGE_DAYS);

        assert!(KNOWN_KEYS.contains(&"prune.max-age-days"));
    }

    #[test]
    fn test_prune_max_age_days_toml_roundtrip() {
        let mut cfg = Config::default();
        cfg.set_value("prune.max-age-days", "14").unwrap();
        let toml_str = toml::to_string(&cfg).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.prune_max_age_days, Some(14));
        assert_eq!(parsed.prune_max_age_days(), 14);
    }

    #[test]
    fn test_registries_toml_roundtrip() {
        let mut cfg = Config::default();
        cfg.add_registry("mybuilds", "https://example.com/godot")
            .unwrap();

        let toml_str = toml::to_string(&cfg).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            parsed.registry_url("mybuilds"),
            Some("https://example.com/godot")
        );
    }
}
