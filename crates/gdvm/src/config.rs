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


use path_clean::PathClean;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf, absolute};

use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::{t, terr};

/// A list of known configuration keys.
pub const KNOWN_KEYS: &[&str] = &["prune.max-age-days", "install.path", "cache.path"];

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_path: Option<PathBuf>,
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
            "install.path" => self
                .install_path
                .clone()
                .map(|p| p.to_string_lossy().into_owned()),
            "cache.path" => self
                .cache_path
                .clone()
                .map(|p| p.to_string_lossy().into_owned()),
            "prune.max-age-days" => self.prune_max_age_days.map(|d| d.to_string()),
            _ => None,
        }
    }

    fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "install.path" => {
                let value = normalize_and_validate_path(Path::new(value), key, self.cache_path.as_ref())?;
                if value.exists() && !value.read_dir()?.next().is_none() {
                    return Err(terr!("error-config-dir-not-empty", path = value.display().to_string()).into());
                }
                self.install_path = Some(value);
                Ok(())
            }
            "cache.path" => {
                let value = normalize_and_validate_path(Path::new(value), key, self.install_path.as_ref())?;
                if value.exists() && !value.read_dir()?.next().is_none() {
                    return Err(terr!("error-config-dir-not-empty", path = value.display().to_string()).into());
                }
                self.cache_path = Some(value);
                Ok(())
            }
            "prune.max-age-days" => {
                let days: u64 = value
                    .parse()
                    .map_err(|_| terr!("error-config-invalid-number", key = key, value = value))?;
                self.prune_max_age_days = Some(days);
                Ok(())
            }
            _ => Err(terr!("error-config-unknown-key", key = key).into()),
        }
    }

    fn unset_value(&mut self, key: &str) -> Result<()> {
        match key {
            "install.path" => {
                self.install_path = None;
                Ok(())
            }
            "cache.path" => {
                self.cache_path = None;
                Ok(())
            }
            "prune.max-age-days" => {
                self.prune_max_age_days = None;
                Ok(())
            }
            _ => Err(terr!("error-config-unknown-key", key = key).into()),
        }
    }

    fn is_sensitive_key(&self, _key: &str) -> bool {
        // No configuration keys are currently sensitive.
        false
    }

    fn list_set_keys(&self) -> Vec<(String, String, bool)> {
        let mut entries = Vec::new();
        if let Some(install_path) = self.install_path.as_ref() {
            entries.push((
                "install.path".to_string(),
                install_path.to_string_lossy().into_owned(),
                false,
            ));
        }
        if let Some(cache_path) = self.cache_path.as_ref() {
            entries.push((
                "cache.path".to_string(),
                cache_path.to_string_lossy().into_owned(),
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

/// The gdvm data directory (~/.gdvm).
pub fn gdvm_dir() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(".gdvm"))
}

pub fn get_home_dir() -> Result<PathBuf> {
    #[cfg(feature = "integration-tests")]
    {
        // Override home directory for testing purposes.
        if let Ok(override_dir) = std::env::var("GDVM_TEST_HOME") {
            return Ok(PathBuf::from(override_dir));
        }
    }

    let base_dirs = BaseDirs::new().ok_or(terr!("error-find-user-dirs"))?;
    Ok(base_dirs.home_dir().to_path_buf())
}

fn is_reserved_path(path: &Path) -> bool {
    let resolved_path = path.clean();
    let candidate = resolved_path;
    let gdvm_base = gdvm_dir().expect("Cannot get gdvm path");
    candidate == gdvm_base
        || candidate.starts_with(&gdvm_base)
        || gdvm_base.starts_with(&candidate)
}

fn normalize_and_validate_path(path: &Path, key: &str, existing: Option<&PathBuf>) -> Result<PathBuf> {
    if path.to_string_lossy().trim().is_empty() {
        return Err(terr!("error-config-path-empty").into());
    }

    let path = absolute(path.clean())?;

    if path.is_file() {
        return Err(terr!("error-config-path-file", path = path.display().to_string()).into());
    }

    if is_reserved_path(&path) {
        return Err(terr!("error-config-path-reserved", path = path.display().to_string()).into());
    }

    if let Some(existing_path) = existing {
        let existing_path = absolute(existing_path)?;
        if path == existing_path
            || path.starts_with(&existing_path)
            || existing_path.starts_with(&path)
        {
            return Err(terr!("error-config-path-overlap", key = key, path = existing_path.display().to_string()).into());
        }
    }

    Ok(path)
}

impl Config {
    /// Load configuration from ~/.gdvm/config.toml.
    pub fn load() -> Result<Self> {
        let config_path = gdvm_dir()?.join("config.toml");
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path).expect("Failed to read config.toml");
            match toml::from_str::<Config>(&contents) {
                Ok(mut config) => {
                    if !config.install_path.is_none() {
                        config.install_path = Some(normalize_and_validate_path(
                            config.install_path.as_ref().unwrap(),
                            "install.path",
                            config.cache_path.as_ref(),
                        )?);
                    }
                    if !config.cache_path.is_none() {
                        config.cache_path = Some(normalize_and_validate_path(
                            config.cache_path.as_ref().unwrap(),
                            "cache.path",
                            config.install_path.as_ref(),
                        )?);
                    }
                    Ok(config)
                },
                Err(e) => {
                    crate::ui::report_error(&terr!("error-parse-config").with_source(e).into());
                    crate::ui::warn(t!("error-parse-config-using-default"));
                    Ok(Self::default())
                }
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to ~/.gdvm/config.toml.
    pub fn modify<T>(f: impl FnOnce(&mut Config) -> Result<T>) -> Result<T> {
        let _lock = crate::locks::Lock::acquire(
            &gdvm_dir()?.join("locks"),
            crate::locks::Resource::Config,
        )?;
        let mut config = Self::load()?;
        let out = f(&mut config)?;
        config.save()?;
        Ok(out)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = gdvm_dir()?;
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        let toml_str = toml::to_string(self).expect("Failed to serialize config");
        crate::fs_utils::atomic_write(&config_path, &toml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_unset_list() {
        let mut cfg = Config::default();
        assert!(cfg.get_value("prune.max-age-days").is_none());
        assert!(cfg.set_value("prune.max-age-days", "5").is_ok());
        assert_eq!(cfg.get_value("prune.max-age-days"), Some("5".to_string()));
        assert!(!cfg.is_sensitive_key("prune.max-age-days"));
        let listed = cfg.list_set_keys();
        assert_eq!(
            listed,
            vec![("prune.max-age-days".to_string(), "5".to_string(), false)]
        );
        assert!(cfg.unset_value("prune.max-age-days").is_ok());
        assert!(cfg.get_value("prune.max-age-days").is_none());
        assert!(cfg.set_value("unknown", "val").is_err());
        assert!(cfg.unset_value("unknown").is_err());
        assert!(cfg.get_value("github.token").is_none());
        assert!(cfg.set_value("github.token", "abc").is_err());
        assert!(!KNOWN_KEYS.contains(&"github.token"));
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
    #[test]
    fn test_normalize_and_validate_path_normalizes_relative_paths() {
        let config = Config::default();
        let path = normalize_and_validate_path(Path::new("../godot"), "install.path", config.cache_path.as_ref()).unwrap();

        assert!(path.is_absolute());
        assert!(
            !path
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        );
        assert_eq!(path, std::env::current_dir().unwrap().join("../godot").clean());
    }

    #[test]
    fn test_normalize_and_validate_path_rejects_empty_strings() {
        let config = Config::default();
        assert!(normalize_and_validate_path(Path::new(""), "install.path", config.cache_path.as_ref()).is_err());
        assert!(normalize_and_validate_path(Path::new("   "), "install.path", config.cache_path.as_ref()).is_err());
    }

    #[test]
    fn test_normalize_and_validate_path_rejects_files() {
        let config = Config::default();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("not_a_dir.txt");
        std::fs::write(&file_path, "data").unwrap();

        assert!(normalize_and_validate_path(Path::new(&file_path.display().to_string()), "install.path", config.cache_path.as_ref()).is_err());
    }

    #[test]
    fn test_normalize_and_validate_path_rejects_reserved_paths() {
        let config = Config::default();
        let gdvm_base = gdvm_dir().unwrap();

        assert!(normalize_and_validate_path(&gdvm_base, "install.path", config.cache_path.as_ref()).is_err());
        assert!(normalize_and_validate_path(&gdvm_base.join("subdir"), "install.path", config.cache_path.as_ref()).is_err());
        assert!(normalize_and_validate_path(gdvm_base.parent().unwrap(), "install.path", config.cache_path.as_ref()).is_err());
    }
}
