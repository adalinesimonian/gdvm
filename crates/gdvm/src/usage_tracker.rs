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

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::date_utils::now_unix_secs;

/// Current schema version of the usage state file.
pub const USAGE_SCHEMA_VERSION: u32 = 1;

/// Recorded usage of a single cached archive, keyed by its file name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveUsage {
    /// Unix timestamp of the most recent use.
    pub last_used: u64,
}

/// Recorded usage of a single install, keyed by its install subpath relative to
/// the installs directory (e.g. `official-abc123/default/4.3-stable`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallUsage {
    /// Unix timestamp of the most recent use.
    pub last_used: u64,
}

/// A link that gdvm created on the user's behalf, pointing into an install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRecord {
    /// The install subpath the link points into.
    pub install_key: String,
    /// Unix timestamp when the link was created or last refreshed.
    pub last_used: u64,
}

/// The full on-disk usage state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageState {
    /// Schema version.
    pub schema: u32,
    /// Archive usage keyed by archive file name.
    #[serde(default)]
    pub archives: HashMap<String, ArchiveUsage>,
    /// Install usage keyed by install subpath.
    #[serde(default)]
    pub installs: HashMap<String, InstallUsage>,
    /// Links keyed by the absolute link path.
    #[serde(default)]
    pub links: HashMap<String, LinkRecord>,
}

impl Default for UsageState {
    fn default() -> Self {
        Self {
            schema: USAGE_SCHEMA_VERSION,
            archives: HashMap::new(),
            installs: HashMap::new(),
            links: HashMap::new(),
        }
    }
}

/// Loads, updates, and persists the usage state file.
pub struct UsageTracker {
    path: PathBuf,
}

impl UsageTracker {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load the state from disk, falling back to an empty state if the file is
    /// missing or unparseable.
    pub fn load(&self) -> Result<UsageState> {
        if !self.path.exists() {
            return Ok(UsageState::default());
        }
        let data = fs::read_to_string(&self.path)?;
        match serde_json::from_str::<UsageState>(&data) {
            Ok(state) => Ok(state),
            Err(_) => Ok(UsageState::default()),
        }
    }

    /// Persist the state to disk.
    pub fn save(&self, state: &UsageState) -> Result<()> {
        let data = serde_json::to_string(state)?;
        atomic_write(&self.path, data)
    }

    /// Update the state and persist to disk.
    fn update<F>(&self, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut UsageState),
    {
        let mut state = self.load()?;
        state.schema = USAGE_SCHEMA_VERSION;
        mutate(&mut state);
        self.save(&state)
    }

    /// Record that the archive with the given file name was used now.
    pub fn record_archive(&self, file_name: &str) -> Result<()> {
        let now = now_unix_secs();
        self.update(|state| {
            state
                .archives
                .insert(file_name.to_string(), ArchiveUsage { last_used: now });
        })
    }

    /// Record that the install with the given subpath key was used now.
    pub fn record_install(&self, install_key: &str) -> Result<()> {
        let now = now_unix_secs();
        self.update(|state| {
            state
                .installs
                .insert(install_key.to_string(), InstallUsage { last_used: now });
        })
    }

    /// Record that a link at `link_path` pointing into `install_key` was made.
    pub fn record_link(&self, link_path: &Path, install_key: &str) -> Result<()> {
        let now = now_unix_secs();
        let key = link_key(link_path);
        let install_key = install_key.to_string();
        self.update(|state| {
            state
                .installs
                .insert(install_key.clone(), InstallUsage { last_used: now });
            state.links.insert(
                key,
                LinkRecord {
                    install_key,
                    last_used: now,
                },
            );
        })
    }

    /// Forget a single install and any links recorded against it.
    pub fn forget_install(&self, install_key: &str) -> Result<()> {
        self.update(|state| {
            state.installs.remove(install_key);
            state.links.retain(|_, rec| rec.install_key != install_key);
        })
    }

    /// Forget a single archive by file name.
    pub fn forget_archive(&self, file_name: &str) -> Result<()> {
        self.update(|state| {
            state.archives.remove(file_name);
        })
    }
}

/// Normalize a link path into a key.
fn link_key(link_path: &Path) -> String {
    let absolute = if link_path.is_absolute() {
        link_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(link_path))
            .unwrap_or_else(|_| link_path.to_path_buf())
    };
    absolute.to_string_lossy().to_string()
}

/// Write data to a file atomically by writing to a temp file in the same folder
/// and renaming it into place.
fn atomic_write(path: &Path, data: String) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("Invalid usage state path"))?;

    fs::create_dir_all(parent)?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(data.as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tracker() -> (TempDir, UsageTracker) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("usage.json");
        (tmp, UsageTracker::new(path))
    }

    #[test]
    fn load_missing_returns_default() {
        let (_tmp, t) = tracker();
        let state = t.load().unwrap();
        assert!(state.archives.is_empty());
        assert!(state.installs.is_empty());
        assert!(state.links.is_empty());
        assert_eq!(state.schema, USAGE_SCHEMA_VERSION);
    }

    #[test]
    fn record_archive_and_install_roundtrip() {
        let (_tmp, t) = tracker();
        t.record_archive("abc123.zip").unwrap();
        t.record_install("official-x/default/4.3-stable").unwrap();

        let state = t.load().unwrap();
        assert!(state.archives.contains_key("abc123.zip"));
        assert!(state.installs.contains_key("official-x/default/4.3-stable"));
    }

    #[test]
    fn record_link_stores_install_key() {
        let (_tmp, t) = tracker();
        let link = if cfg!(windows) {
            std::path::PathBuf::from(r"C:\tmp\some\link")
        } else {
            std::path::PathBuf::from("/tmp/some/link")
        };
        t.record_link(&link, "official-x/default/4.3-stable")
            .unwrap();

        let state = t.load().unwrap();
        assert_eq!(state.links.len(), 1, "exactly one link should be recorded");
        let rec = state.links.values().next().expect("link recorded");
        assert_eq!(rec.install_key, "official-x/default/4.3-stable");
    }

    #[test]
    fn forget_install_removes_links() {
        let (_tmp, t) = tracker();
        let link = if cfg!(windows) {
            std::path::PathBuf::from(r"C:\tmp\some\link")
        } else {
            std::path::PathBuf::from("/tmp/some/link")
        };
        t.record_install("k").unwrap();
        t.record_link(&link, "k").unwrap();
        t.forget_install("k").unwrap();

        let state = t.load().unwrap();
        assert!(!state.installs.contains_key("k"));
        assert!(state.links.is_empty());
    }

    #[test]
    fn corrupt_file_loads_as_default() {
        let (tmp, t) = tracker();
        std::fs::write(tmp.path().join("usage.json"), "not json").unwrap();
        let state = t.load().unwrap();
        assert!(state.installs.is_empty());
    }
}
