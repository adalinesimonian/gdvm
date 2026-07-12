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

use crate::config::get_home_dir;

/// Centralizes filesystem layout for GDVM under the user home directory.
pub struct GdvmPaths {
    base: PathBuf,
    installs: PathBuf,
    cache_dir: PathBuf,
    cache_index: PathBuf,
    usage_index: PathBuf,
    bin_dir: PathBuf,
}

impl GdvmPaths {
    /// Construct paths rooted at the GDVM base directory, ~/.gdvm, and ensure the base, installs,
    /// cache, and bin directories exist.
    pub fn new() -> Result<Self> {
        let base = get_home_dir()?.join(".gdvm");
        let installs = base.join("installs");
        let cache_dir = base.join("cache");
        let cache_index = base.join("cache.json");
        let usage_index = base.join("usage.json");
        let bin_dir = base.join("bin");

        fs::create_dir_all(&installs)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&bin_dir)?;

        Ok(Self {
            base,
            installs,
            cache_dir,
            cache_index,
            usage_index,
            bin_dir,
        })
    }

    pub fn base(&self) -> &Path {
        &self.base
    }

    pub fn installs(&self) -> &Path {
        &self.installs
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn cache_index(&self) -> &Path {
        &self.cache_index
    }

    pub fn usage_index(&self) -> &Path {
        &self.usage_index
    }

    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    pub fn locks(&self) -> PathBuf {
        self.base.join("locks")
    }

    pub fn default_file(&self) -> PathBuf {
        self.base.join("default")
    }

    pub fn current_godot_symlink(&self) -> PathBuf {
        self.bin_dir.join("current_godot")
    }

    #[cfg(test)]
    pub fn from_base_for_tests(base: PathBuf) -> Result<Self> {
        let installs = base.join("installs");
        let cache_dir = base.join("cache");
        let cache_index = base.join("cache.json");
        let usage_index = base.join("usage.json");
        let bin_dir = base.join("bin");

        fs::create_dir_all(&installs)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&bin_dir)?;

        Ok(Self {
            base,
            installs,
            cache_dir,
            cache_index,
            usage_index,
            bin_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn creates_directories_and_exposes_paths() -> Result<()> {
        let tmp = TempDir::new()?;
        let paths = GdvmPaths::from_base_for_tests(tmp.path().to_path_buf())?;

        assert!(paths.base().starts_with(tmp.path()));
        assert!(paths.installs().exists());
        assert!(paths.cache_dir().exists());
        assert!(paths.bin_dir().exists());

        // Derived files live under base.
        assert_eq!(paths.default_file(), paths.base().join("default"));
        assert_eq!(paths.usage_index(), paths.base().join("usage.json"));
        assert_eq!(
            paths.current_godot_symlink(),
            paths.bin_dir().join("current_godot")
        );

        Ok(())
    }
}
