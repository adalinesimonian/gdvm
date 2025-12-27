use crate::config::get_home_dir;
use crate::i18n::I18n;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Centralizes filesystem layout for GDVM under the user home directory.
pub struct GdvmPaths {
    base: PathBuf,
    installs: PathBuf,
    cache_dir: PathBuf,
    cache_index: PathBuf,
    bin_dir: PathBuf,
}

impl GdvmPaths {
    /// Construct paths rooted at the GDVM base directory, ~/.gdvm, and ensure the base, installs,
    /// cache, and bin directories exist.
    pub fn new(i18n: &I18n) -> Result<Self> {
        let base = get_home_dir(i18n)?.join(".gdvm");
        let installs = base.join("installs");
        let cache_dir = base.join("cache");
        let cache_index = base.join("cache.json");
        let bin_dir = base.join("bin");

        fs::create_dir_all(&installs)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&bin_dir)?;

        Ok(Self {
            base,
            installs,
            cache_dir,
            cache_index,
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

    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
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
        let bin_dir = base.join("bin");

        fs::create_dir_all(&installs)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&bin_dir)?;

        Ok(Self {
            base,
            installs,
            cache_dir,
            cache_index,
            bin_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
        assert_eq!(
            paths.current_godot_symlink(),
            paths.bin_dir().join("current_godot")
        );

        Ok(())
    }
}
