// SPDX-FileCopyrightText: Copyright (C) 2026 Adaline Simonian
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
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{eprintln_i18n, hash_utils};

/// Resources whose write operations gdvm serializes across processes.
pub enum Resource<'a> {
    /// An install operation for a specific registry, variant, and version.
    Install(&'a str),
    /// An operation on a specific archive file in the cache.
    Archive(&'a str),
    /// Any change to the `default` symlink.
    Defaults,
    /// Change to the usage tracking file.
    Usage,
    /// Change to the config file.
    Config,
    /// Migrations of gdvm's local data.
    Migrations,
    /// Actions run after gdvm is upgraded.
    PostUpgrade,
    /// Upgrade of gdvm.
    SelfUpgrade,
}

impl Resource<'_> {
    /// Get the key identifying the resource.
    fn key(&self) -> String {
        match self {
            Resource::Install(subpath) => format!("install:{subpath}"),
            Resource::Archive(file_name) => format!("archive:{file_name}"),
            Resource::Defaults => "defaults".to_string(),
            Resource::Usage => "usage".to_string(),
            Resource::Config => "config".to_string(),
            Resource::Migrations => "migrations".to_string(),
            Resource::PostUpgrade => "post-upgrade".to_string(),
            Resource::SelfUpgrade => "self-upgrade".to_string(),
        }
    }
}

/// An exclusive lock on a `Resource`.
#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct Lock {
    _file: fs::File,
}

impl Lock {
    /// Acquire the lock, blocking if another process holds it.
    pub fn acquire(locks_dir: &Path, resource: Resource) -> Result<Lock> {
        let file = open_lock_file(locks_dir, &resource)?;

        match file.try_lock() {
            Ok(()) => return Ok(Lock { _file: file }),
            Err(fs::TryLockError::WouldBlock) => {}
            Err(fs::TryLockError::Error(e)) => return Err(e.into()),
        }

        eprintln_i18n!("lock-waiting", resource = resource.key());
        file.lock()?;

        Ok(Lock { _file: file })
    }

    /// Acquire the lock only if no other process holds it.
    pub fn try_acquire(locks_dir: &Path, resource: Resource) -> Result<Option<Lock>> {
        let file = open_lock_file(locks_dir, &resource)?;

        match file.try_lock() {
            Ok(()) => Ok(Some(Lock { _file: file })),
            Err(fs::TryLockError::WouldBlock) => Ok(None),
            Err(fs::TryLockError::Error(e)) => Err(e.into()),
        }
    }
}

/// Open the lock file for a resource.
fn open_lock_file(locks_dir: &Path, resource: &Resource) -> io::Result<fs::File> {
    fs::create_dir_all(locks_dir)?;
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(lock_file_path(locks_dir, &resource.key()))
}

/// Get the path to the lock file for a resource.
fn lock_file_path(locks_dir: &Path, key: &str) -> PathBuf {
    let prefix: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '-'
            }
        })
        .take(64)
        .collect();
    let digest = hash_utils::to_hex(&Sha256::digest(key.as_bytes())[..8]);
    locks_dir.join(format!("{prefix}-{digest}.lock"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_lock_blocks_and_drop_releases() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let held = Lock::acquire(dir.path(), Resource::Config)?;

        assert!(Lock::try_acquire(dir.path(), Resource::Config)?.is_none());
        // Unrelated resources should be unaffected.
        assert!(Lock::try_acquire(dir.path(), Resource::Defaults)?.is_some());

        drop(held);
        assert!(Lock::try_acquire(dir.path(), Resource::Config)?.is_some());

        Ok(())
    }

    #[test]
    fn install_locks_are_per_unit() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let _a = Lock::acquire(dir.path(), Resource::Install("official/default/4.3-stable"))?;

        assert!(
            Lock::try_acquire(dir.path(), Resource::Install("official/default/4.4-stable"))?
                .is_some()
        );
        assert!(
            Lock::try_acquire(dir.path(), Resource::Install("official/default/4.3-stable"))?
                .is_none()
        );

        Ok(())
    }

    #[test]
    fn keys_with_path_separators_get_distinct_flat_file_names() {
        let dir = Path::new("/locks");
        let a = lock_file_path(dir, "install:a/b");
        let b = lock_file_path(dir, "install:a-b");
        assert_ne!(a, b);
        assert_eq!(a.parent(), Some(dir));
        assert!(a.file_name().unwrap().to_string_lossy().ends_with(".lock"));
    }
}
