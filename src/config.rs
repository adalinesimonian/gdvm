use crate::{eprintln_i18n, i18n};
use anyhow::{Result, anyhow};
use directories::BaseDirs;
use i18n::I18n;
use serde::{Deserialize, Serialize};
use std::fs;

/// A list of known configuration keys.
pub const KNOWN_KEYS: &[&str] = &["github.token"];

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub github_token: Option<String>,
}

/// Get/set operations for configuration keys.
pub trait ConfigOps {
    /// Retrieve the value of a configuration key, if set.
    fn get_value(&self, key: &str) -> Option<String>;

    /// Set a configuration key to the provided value.
    fn set_value(&mut self, key: &str, value: &str) -> Result<(), ()>;

    /// Unset (remove) a configuration key's value.
    fn unset_value(&mut self, key: &str) -> Result<(), ()>;

    /// Return true if the key is considered sensitive.
    fn is_sensitive_key(&self, key: &str) -> bool;

    /// List all set configuration keys as (key, value, is_sensitive) tuples.
    fn list_set_keys(&self) -> Vec<(String, String, bool)>;
}

impl ConfigOps for Config {
    fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "github.token" => self.github_token.clone(),
            _ => None,
        }
    }

    fn set_value(&mut self, key: &str, value: &str) -> Result<(), ()> {
        match key {
            "github.token" => {
                self.github_token = Some(value.to_string());
                Ok(())
            }
            _ => Err(()),
        }
    }

    fn unset_value(&mut self, key: &str) -> Result<(), ()> {
        match key {
            "github.token" => {
                self.github_token = None;
                Ok(())
            }
            _ => Err(()),
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
        entries
    }
}

impl Config {
    /// Load configuration from ~/.gdvm/config.toml.
    pub fn load(i18n: &I18n) -> Result<Self> {
        let base_dirs = BaseDirs::new().ok_or(anyhow!(i18n.t_w("error-find-user-dirs")))?;
        let home = base_dirs.home_dir();
        let config_path = home.join(".gdvm").join("config.toml");
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path).expect("Failed to read config.toml");
            match toml::from_str(&contents) {
                Ok(config) => Ok(config),
                Err(e) => {
                    eprintln_i18n!(
                        i18n,
                        "error-parse-config",
                        [("error", fluent_bundle::FluentValue::from(e.to_string()))]
                    );
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
        let base_dirs = BaseDirs::new().ok_or(anyhow!(i18n.t_w("error-find-user-dirs")))?;
        let home = base_dirs.home_dir();
        let config_dir = home.join(".gdvm");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        let toml_str = toml::to_string(self).expect("Failed to serialize config");
        fs::write(config_path, toml_str)?;
        Ok(())
    }
}
