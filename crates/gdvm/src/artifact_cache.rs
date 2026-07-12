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

    /// Path of the cached archive for content with the given SHA-512.
    pub fn cached_zip_path(&self, sha512: &str) -> PathBuf {
        self.dir.join(format!("{}.zip", archive_key(sha512)))
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
