use crate::{eprintln_i18n, i18n, t_w};
use anyhow::{Result, anyhow};
use directories::BaseDirs;
use i18n::I18n;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
            _ => None,
        }
    }

    fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "github.token" => {
                self.github_token = Some(value.to_string());
                Ok(())
            }
            _ => Err(anyhow!("Unknown configuration key: {}", key)),
        }
    }

    fn unset_value(&mut self, key: &str) -> Result<()> {
        match key {
            "github.token" => {
                self.github_token = None;
                Ok(())
            }
            _ => Err(anyhow!("Unknown configuration key: {}", key)),
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

fn get_home_dir(i18n: &I18n) -> Result<PathBuf> {
    #[cfg(feature = "integration-tests")]
    {
        // Override home directory for testing purposes.
        if let Ok(override_dir) = std::env::var("GDVM_TEST_HOME") {
            return Ok(PathBuf::from(override_dir));
        }
    }

    let base_dirs = BaseDirs::new().ok_or(anyhow!(t_w!(i18n, "error-find-user-dirs")))?;
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
}
