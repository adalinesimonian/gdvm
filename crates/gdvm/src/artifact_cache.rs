use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages the on-disk cache directory for downloaded Godot artifacts.
pub struct ArtifactCache {
    dir: PathBuf,
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

    pub fn cached_zip_path(&self, archive_name: &str) -> PathBuf {
        self.dir.join(archive_name)
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
