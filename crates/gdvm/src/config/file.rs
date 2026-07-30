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

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use serde::de::IntoDeserializer;
use toml_edit::DocumentMut;

use super::document::{document_get, document_remove, document_set};
use super::schema::ManagedKey;
use super::{Config, ConfigKey};
use crate::paths::gdvm_dir;
use crate::{t, terr};

/// The table registries live in.
const REGISTRIES: &str = ManagedKey::Registries.as_str();

/// Set when config problems were reported to the user to avoid spamming.
static PROBLEMS_REPORTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileState {
    /// The file was read and parsed.
    Usable,
    /// The file exists but could not be read.
    Unreadable,
    /// The file was read but is not valid TOML.
    Malformed,
}

#[derive(Debug, Clone)]
pub struct ConfigProblem {
    /// The configuration key.
    pub key: String,
    /// Why the value could not be used.
    pub detail: String,
}

/// A key whose value has changed since the file was loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ChangedKey {
    /// A key gdvm owns outright.
    Managed(ManagedKey),
    /// A single registry entry, by alias.
    Registry(String),
}

/// A configuration file.
#[derive(Debug)]
pub struct ConfigFile {
    /// The path to the file.
    path: PathBuf,
    /// The parsed document.
    document: Option<DocumentMut>,
    /// The state of the config file.
    state: ConfigFileState,
    /// Any error encountered while reading or parsing the file.
    error: Option<String>,
    /// Values that could not be used.
    problems: Vec<ConfigProblem>,
    /// The configuration object.
    config: Config,
    /// Keys that have changed since last load.
    changed: Vec<ChangedKey>,
}

impl ConfigFile {
    /// Load configuration from ~/.gdvm/config.toml.
    pub fn load() -> Result<Self> {
        Self::load_from(Self::path()?)
    }

    /// Load configuration from the given path.
    pub fn load_from(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        let mut file = Self {
            path,
            document: None,
            state: ConfigFileState::Usable,
            error: None,
            problems: Vec::new(),
            config: Config::default(),
            changed: Vec::new(),
        };

        if !file.path.exists() {
            return Ok(file);
        }

        let contents = match fs::read_to_string(&file.path) {
            Ok(contents) => contents,
            Err(e) => {
                file.state = ConfigFileState::Unreadable;
                file.error = Some(e.to_string());
                return Ok(file);
            }
        };

        let document: DocumentMut = match contents.parse() {
            Ok(document) => document,
            Err(e) => {
                file.state = ConfigFileState::Malformed;
                file.error = Some(e.to_string());
                return Ok(file);
            }
        };

        for &key in ManagedKey::ALL {
            let Some(item) = document_get(&document, key.as_str()) else {
                continue;
            };

            let result = match item.clone().into_value() {
                Err(_) => Err("expected a value".to_string()),
                Ok(value) => file
                    .config
                    .apply_managed_key(key, value.into_deserializer()),
            };

            if let Err(e) = result {
                file.problems.push(ConfigProblem {
                    key: key.as_str().to_string(),
                    detail: e.trim().replace('\n', " "),
                });
            }
        }

        file.document = Some(document);

        Ok(file)
    }

    /// Modify configuration at ~/.gdvm/config.toml.
    pub fn modify<T>(f: impl FnOnce(&mut ConfigFile) -> Result<T>) -> Result<T> {
        let _lock = crate::locks::Lock::acquire(
            &gdvm_dir()?.join("locks"),
            crate::locks::Resource::Config,
        )?;
        let mut config = Self::load()?;
        let out = f(&mut config)?;
        config.save()?;
        Ok(out)
    }

    /// Get the path to the configuration file.
    pub fn path() -> Result<PathBuf> {
        Ok(gdvm_dir()?.join("config.toml"))
    }

    /// Get the configuration object.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the configuration object and discard the file wrapper.
    pub fn into_config(self) -> Config {
        self.config
    }

    /// Get the config state.
    pub fn state(&self) -> ConfigFileState {
        self.state
    }

    /// Get the error encountered while reading or parsing the file, if any.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Get the list of configuration problems encountered while reading the file.
    pub fn problems(&self) -> &[ConfigProblem] {
        &self.problems
    }

    /// Get whether the configuration file has any problems.
    pub fn has_problems(&self) -> bool {
        self.state != ConfigFileState::Usable || !self.problems.is_empty()
    }

    /// Set a configuration key to the given value.
    pub fn set_value(&mut self, key: ConfigKey, value: &str) -> Result<()> {
        self.config.set_value(key, value)?;
        self.mark(ChangedKey::Managed(ManagedKey::Setting(key)));
        Ok(())
    }

    /// Remove a configuration key's value.
    pub fn unset_value(&mut self, key: ConfigKey) {
        self.config.unset_value(key);
        self.mark(ChangedKey::Managed(ManagedKey::Setting(key)));
    }

    /// Store a registry in config.
    pub fn add_registry(&mut self, name: &str, url: &str) -> Result<()> {
        self.config.add_registry(name, url)?;
        self.mark(ChangedKey::Registry(name.to_string()));
        Ok(())
    }

    /// Remove a registry from config.
    pub fn remove_registry(&mut self, name: &str) -> Result<()> {
        self.config.remove_registry(name)?;
        self.mark(ChangedKey::Registry(name.to_string()));
        Ok(())
    }

    /// Trust the registry at `url`.
    pub fn trust_registry(&mut self, url: &str) {
        if !self.config.is_registry_trusted(url) {
            self.config.trust_registry(url);
            self.mark(ChangedKey::Managed(ManagedKey::TrustedRegistries));
        }
    }

    /// Record a change so that it is written on the next save.
    fn mark(&mut self, key: ChangedKey) {
        if !self.changed.contains(&key) {
            self.changed.push(key);
        }
    }

    /// Save configuration to ~/.gdvm/config.toml.
    pub fn save(&self) -> Result<()> {
        if self.state != ConfigFileState::Usable {
            return Err(terr!(
                "error-config-unusable-not-saving",
                path = self.path.display().to_string()
            )
            .into());
        }

        let mut document = self.document.clone().unwrap_or_default();
        let generated = toml_edit::ser::to_document(&self.config)?;

        for change in &self.changed {
            match change {
                ChangedKey::Managed(key) => write_managed(&mut document, &generated, *key),
                ChangedKey::Registry(name) => self.write_registry(&mut document, &generated, name),
            }
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        crate::fs_utils::atomic_write(&self.path, &document.to_string())?;
        Ok(())
    }

    fn write_registry(&self, document: &mut DocumentMut, generated: &DocumentMut, name: &str) {
        if document
            .get(REGISTRIES)
            .is_some_and(|item| !item.is_table())
        {
            match generated.get(REGISTRIES) {
                Some(item) => document[REGISTRIES] = item.clone(),
                None => {
                    document.remove(REGISTRIES);
                }
            }
            return;
        }

        let Some(registry) = self.config.registries.get(name) else {
            if let Some(table) = document
                .get_mut(REGISTRIES)
                .and_then(|item| item.as_table_mut())
            {
                table.remove(name);

                if table.is_empty() {
                    document.remove(REGISTRIES);
                }
            }
            return;
        };

        let Some(table) = document[REGISTRIES]
            .or_insert(toml_edit::table())
            .as_table_mut()
        else {
            return;
        };

        table.set_implicit(true);

        let is_new = !table.contains_key(name);
        let entry = table[name].or_insert(toml_edit::table());

        entry["url"] = toml_edit::value(registry.url.as_str());

        if is_new && let Some(added) = entry.as_table_mut() {
            added.decor_mut().set_prefix("\n");
        }
    }

    /// Suppress the problem report so it is not printed to the user.
    pub fn suppress_problem_report() {
        PROBLEMS_REPORTED.store(true, Ordering::Relaxed);
    }

    /// Report any problems encountered while reading the configuration file.
    pub fn report_problems(&self) {
        if !self.has_problems() || PROBLEMS_REPORTED.swap(true, Ordering::Relaxed) {
            return;
        }

        let path = self.path.display().to_string();

        match self.state {
            ConfigFileState::Unreadable => {
                crate::ui::warn(t!("config-file-unreadable", path = path.as_str()));
                self.note_error();
                crate::ui::warn(t!("error-parse-config-using-default"));

                return;
            }
            ConfigFileState::Malformed => {
                crate::ui::report_error(&terr!("error-parse-config").into());
                self.note_error();
                crate::ui::warn(t!("error-parse-config-using-default"));

                return;
            }
            ConfigFileState::Usable => {}
        }

        match self.problems.as_slice() {
            [] => {}
            [problem] => crate::ui::warn(t!(
                "config-value-ignored",
                key = problem.key.as_str(),
                detail = problem.detail.as_str()
            )),
            problems => crate::ui::error(t!(
                "config-problems-multiple",
                count = problems.len(),
                path = path.as_str()
            )),
        }
    }

    fn note_error(&self) {
        if let Some(error) = self.error() {
            crate::ui::note(error);
        }
    }
}

impl From<ConfigFile> for Config {
    fn from(file: ConfigFile) -> Self {
        file.into_config()
    }
}

fn write_managed(document: &mut DocumentMut, generated: &DocumentMut, key: ManagedKey) {
    match document_get(generated, key.as_str()) {
        Some(item) => document_set(document, key.as_str(), item.clone()),
        None => document_remove(document, key.as_str()),
    }
}
