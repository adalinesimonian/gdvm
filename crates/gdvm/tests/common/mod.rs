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

// Number of tests have cfg(feature = "integration-tests") so portions of this
// file are unused when that feature is disabled.
#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use gdvm::app::Gdvm;
use gdvm::usage_tracker::{UsageState, UsageTracker};
use gdvm::version::{ResolvedVersion, VersionQuery};
use tempfile::TempDir;

/// Temporary fake home directory for a test.
pub struct TestHome {
    home: TempDir,
    project: Option<TempDir>,
    prev_home: Option<std::ffi::OsString>,
    prev_cwd: Option<PathBuf>,
}

impl TestHome {
    pub fn new() -> Self {
        Self::create(false)
    }

    /// Create a temporary home directory and also a temporary project directory
    /// as the current working directory.
    pub fn with_project() -> Self {
        Self::create(true)
    }

    fn create(with_project: bool) -> Self {
        let home = TempDir::new().unwrap();
        let project = with_project.then(|| TempDir::new().unwrap());
        let prev_home = std::env::var_os("GDVM_TEST_HOME");
        let prev_cwd = std::env::current_dir().ok();
        let gdvm_dir = home.path().join(".gdvm");

        fs::create_dir_all(&gdvm_dir).unwrap();
        fs::write(
            gdvm_dir.join("cache.json"),
            format!(
                r#"{{"gdvm":{{"last_update_check":{},"new_version":null,"new_major_version":null}},"registries":{{}}}}"#,
                now_secs()
            ),
        )
        .unwrap();

        unsafe {
            std::env::set_var("GDVM_TEST_HOME", home.path());
        }

        if let Some(project) = &project {
            std::env::set_current_dir(project.path()).unwrap();
        }

        Self {
            home,
            project,
            prev_home,
            prev_cwd,
        }
    }

    /// The home directory.
    pub fn path(&self) -> &Path {
        self.home.path()
    }

    /// The project directory.
    pub fn project_dir(&self) -> &Path {
        self.project
            .as_ref()
            .expect("this TestHome was created without a project directory")
            .path()
    }

    /// The `.gdvm` folder inside the home directory.
    pub fn gdvm_dir(&self) -> PathBuf {
        self.path().join(".gdvm")
    }

    /// The install store directory.
    pub fn installs(&self) -> PathBuf {
        self.gdvm_dir().join("installs")
    }

    /// The artifact cache directory.
    pub fn cache(&self) -> PathBuf {
        self.gdvm_dir().join("cache")
    }

    /// The usage tracking state file.
    pub fn usage_path(&self) -> PathBuf {
        self.gdvm_dir().join("usage.json")
    }

    /// The lock directory.
    pub fn locks_path(&self) -> PathBuf {
        self.gdvm_dir().join("locks")
    }

    /// Create an install at `key` with a file inside.
    pub fn make_install(&self, key: &str) -> PathBuf {
        let dir = self.installs().join(key);

        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Godot"), b"fake-binary").unwrap();
        dir
    }

    /// Create a cached archive file with the given name and contents.
    pub fn make_cache_file(&self, name: &str, contents: &[u8]) -> PathBuf {
        let dir = self.cache();

        fs::create_dir_all(&dir).unwrap();

        let path = dir.join(name);

        fs::write(&path, contents).unwrap();
        path
    }

    /// Save usage tracking state.
    pub fn write_usage(&self, state: &UsageState) {
        UsageTracker::new(self.usage_path(), self.locks_path())
            .save(state)
            .unwrap();
    }

    /// Load usage tracking state.
    pub fn read_usage(&self) -> UsageState {
        UsageTracker::new(self.usage_path(), self.locks_path())
            .load()
            .unwrap()
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        // Leave the temp project folder so that it can be deleted.
        if let Some(cwd) = &self.prev_cwd {
            let _ = std::env::set_current_dir(cwd);
        }

        unsafe {
            match &self.prev_home {
                Some(v) => std::env::set_var("GDVM_TEST_HOME", v),
                None => std::env::remove_var("GDVM_TEST_HOME"),
            }
        }
    }
}

/// Enter a directory, restoring the previous working directory on drop.
pub struct CwdGuard(Option<PathBuf>);

impl CwdGuard {
    pub fn enter(dir: &Path) -> Self {
        let prev = std::env::current_dir().ok();
        std::env::set_current_dir(dir).unwrap();
        CwdGuard(prev)
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Some(prev) = &self.0 {
            let _ = std::env::set_current_dir(prev);
        }
    }
}

/// Get a gdvm instance with the test home directory set up.
pub async fn gdvm() -> Gdvm {
    Gdvm::new().await.unwrap()
}

/// Get a resolved version from an install string.
pub fn resolved(install_str: &str) -> ResolvedVersion {
    VersionQuery::from_install_str(install_str)
        .unwrap()
        .to_resolved()
}

/// Get unix timestamp for now.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Get the host platform key.
pub fn host_platform() -> String {
    let host = gdvm::host::detect_host().unwrap();

    format!(
        "{}-{}",
        gdvm::registry::registry_os_key(host),
        gdvm::registry::registry_arch_key(host)
    )
}

/// Write a zip archive at `path` containing a single `entry` with
/// `contents`.
pub fn make_zip(path: &Path, entry: &str, contents: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    zip.start_file(entry, zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(contents).unwrap();
    zip.finish().unwrap();
}
