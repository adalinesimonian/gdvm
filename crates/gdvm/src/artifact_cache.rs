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
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Number of hex characters of a content hash used to name a cached archive.
const ARCHIVE_KEY_HEX_LENGTH: usize = 16;

/// Prefix for partial download files.
pub const PARTIAL_PREFIX: &str = ".partial-";

/// Manages the on-disk cache directory for downloaded Godot artifacts.
pub struct ArtifactCache {
    dir: PathBuf,
}

/// Short content key derived from an archive's SHA. Safe for filesystem use.
fn archive_key(sha512: &str) -> String {
    sha512
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_lowercase())
        .take(ARCHIVE_KEY_HEX_LENGTH)
        .collect()
}

impl ArtifactCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn exists(&self) -> bool {
        self.dir.exists()
    }

    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        Ok(())
    }

    /// Get a list of left behind partial downloads.
    pub fn partial_downloads(&self) -> Vec<(PathBuf, u64)> {
        let Ok(entries) = std::fs::read_dir(self.dir()) else {
            return Vec::new();
        };
        let mut partials: Vec<(PathBuf, u64)> = entries
            .flatten()
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with(PARTIAL_PREFIX)
                    && e.file_type().is_ok_and(|ft| ft.is_file())
            })
            .map(|e| {
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                (e.path(), size)
            })
            .collect();
        partials.sort();
        partials
    }

    /// Remove partial downloads older than `max_age`. Keeps any paths in `keep`.
    pub fn sweep_stale_partials(&self, max_age: std::time::Duration, keep: &[&Path]) -> u64 {
        let now = std::time::SystemTime::now();
        let mut freed = 0;

        for (path, size) in self.partial_downloads() {
            if keep.contains(&path.as_path()) {
                continue;
            }
            let stale = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .is_some_and(|age| age >= max_age);
            if stale && std::fs::remove_file(&path).is_ok() {
                freed += size;
            }
        }

        freed
    }

    /// Path of the cached archive for content with the given SHA-512.
    pub fn cached_zip_path(&self, sha512: &str) -> PathBuf {
        self.dir.join(format!("{}.zip", archive_key(sha512)))
    }

    /// Get the path for a partial download.
    pub fn partial_zip_path(&self, sha512: &str) -> PathBuf {
        self.dir
            .join(format!("{PARTIAL_PREFIX}{}.zip", archive_key(sha512)))
    }

    /// Get the path for a partial download's metadata file.
    pub fn partial_meta_path(&self, sha512: &str) -> PathBuf {
        self.dir
            .join(format!("{PARTIAL_PREFIX}{}.zip.meta", archive_key(sha512)))
    }

    pub fn clear_files(&self) -> Result<()> {
        if !self.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn partials_are_listed_and_swept_by_age() {
        let tmp = TempDir::new().unwrap();
        let cache = ArtifactCache::new(tmp.path().to_path_buf());
        cache.ensure_dir().unwrap();

        let stale = cache.dir().join(".partial-abc.zip");
        let fresh = cache.dir().join(".partial-def.zip");
        let finished = cache.dir().join("finished.zip");
        std::fs::write(&stale, vec![0u8; 100]).unwrap();
        std::fs::write(&fresh, vec![0u8; 50]).unwrap();
        std::fs::write(&finished, vec![0u8; 25]).unwrap();

        assert_eq!(cache.partial_downloads().len(), 2);

        let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(48 * 3600);
        let file = std::fs::File::options().write(true).open(&stale).unwrap();
        file.set_modified(old_time).unwrap();
        drop(file);

        let freed = cache.sweep_stale_partials(std::time::Duration::from_secs(24 * 3600), &[]);
        assert_eq!(freed, 100);
        assert!(!stale.exists());
        assert!(fresh.exists());
        assert!(finished.exists());
    }

    #[test]
    fn cache_path_is_content_addressed_and_short() {
        let tmp = TempDir::new().unwrap();
        let cache = ArtifactCache::new(tmp.path().to_path_buf());

        let sha_a = "a".repeat(128);
        let sha_b = "b".repeat(128);

        let path_a = cache.cached_zip_path(&sha_a);
        let path_b = cache.cached_zip_path(&sha_b);

        assert_ne!(path_a, path_b);
        assert_eq!(path_a, cache.cached_zip_path(&sha_a));

        let name = path_a.file_name().unwrap().to_string_lossy();
        assert_eq!(name, format!("{}.zip", "a".repeat(ARCHIVE_KEY_HEX_LENGTH)));
        assert!(name.len() <= ARCHIVE_KEY_HEX_LENGTH + 4);
    }

    #[test]
    fn archive_key_lowercases_and_ignores_non_hex() {
        assert_eq!(
            archive_key(&"A".repeat(128)),
            "a".repeat(ARCHIVE_KEY_HEX_LENGTH)
        );
        assert_eq!(archive_key("xx1234567890abcdef1234"), "1234567890abcdef");
    }
}
