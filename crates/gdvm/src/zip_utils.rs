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

use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use zip::ZipArchive;

use crate::{t, terr, ui};

pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<()> {
    let mut file = fs::File::open(zip_path).map_err(|e| {
        terr!("error-open-zip", path = zip_path.display().to_string(),).with_source(e)
    })?;

    let subject = zip_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    extract_zip_from_file(&mut file, zip_path, extract_to, &subject)
}

/// Extract a ZIP archive from the given file handle. `subject` is the text sent
/// to the progress indicator describing what's being extracted.
pub fn extract_zip_from_file(
    file: &mut fs::File,
    zip_path: &Path,
    extract_to: &Path,
    subject: &str,
) -> Result<()> {
    // First pass: Calculate total uncompressed size and collect top-level entries
    file.rewind().map_err(|e| {
        terr!("error-open-zip", path = zip_path.display().to_string(),).with_source(e)
    })?;
    let mut archive = ZipArchive::new(&*file).map_err(|e| {
        terr!("error-read-zip", path = zip_path.display().to_string(),).with_source(e)
    })?;

    let mut total_size: u64 = 0;
    let mut top_level_entries = HashSet::new();
    let mut top_level_dirs = HashSet::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| terr!("error-access-file", index = i).with_source(e))?;

        if let Some(enclosed) = file.enclosed_name() {
            if let Some(first_component) = enclosed.components().next() {
                let entry = first_component.as_os_str().to_string_lossy().to_string();
                top_level_entries.insert(entry.clone());

                if file.is_dir() {
                    top_level_dirs.insert(entry);
                }
            }
        } else {
            return Err(terr!("error-invalid-file-name").into());
        }

        if !file.is_dir() {
            total_size += file.size();
        }
    }

    // Determine if there's a common top-level directory
    let common_prefix: Option<PathBuf> =
        if top_level_entries.len() == 1 && top_level_dirs.len() == 1 {
            let prefix = top_level_dirs.into_iter().next().unwrap();
            if prefix.ends_with(".app") {
                None
            } else {
                Some(PathBuf::from(prefix))
            }
        } else {
            None
        };

    // Initialize progress bar with total uncompressed size
    let task = ui::progress::gauge(t!("status-extracting"), subject, total_size);

    // Second pass: Extract files and update progress
    file.rewind().map_err(|e| {
        terr!("error-reopen-zip", path = zip_path.display().to_string(),).with_source(e)
    })?;
    let mut archive = ZipArchive::new(&*file).map_err(|e| {
        terr!("error-read-zip", path = zip_path.display().to_string(),).with_source(e)
    })?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| terr!("error-access-file", index = i).with_source(e))?;
        let path = file
            .enclosed_name()
            .ok_or_else(|| terr!("error-invalid-file-name"))?;

        // Skip empty or root entries to avoid creating files at extract_to
        if path.as_os_str().is_empty() || path == Path::new(".") {
            continue;
        }

        // Determine the output path, stripping the common prefix if it exists
        let out_path = if let Some(ref prefix) = common_prefix {
            // Ensure the file path starts with the prefix
            let stripped_path = path
                .strip_prefix(prefix)
                .map_err(|e| terr!("error-strip-prefix").with_source(e))?;

            // Prevent extracting files outside the target directory
            if stripped_path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                return Err(terr!("error-invalid-file-name").into());
            }

            extract_to.join(stripped_path)
        } else {
            extract_to.join(path)
        };

        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| {
                terr!("error-create-dir", path = out_path.display().to_string(),).with_source(e)
            })?;
            continue;
        }

        // Ensure the parent directory exists
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                terr!("error-create-dir", path = parent.display().to_string(),).with_source(e)
            })?;
        }

        // Create the output file
        let mut outfile = fs::File::create(&out_path).map_err(|e| {
            terr!("error-create-file", path = out_path.display().to_string(),).with_source(e)
        })?;

        // Read from the ZIP file and write to the output file in chunks
        let declared_size = file.size();
        let mut written: u64 = 0;
        let mut buffer = [0; 8192]; // 8 KB buffer
        let reader = &mut file;
        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| {
                terr!("error-read-zip-file", file = reader.name().to_string(),).with_source(e)
            })?;
            if bytes_read == 0 {
                break; // Extraction complete for this file
            }
            written += bytes_read as u64;
            if written > declared_size {
                return Err(terr!(
                    "error-size-mismatch",
                    file = reader.name().to_string(),
                    expected = declared_size,
                    actual = written
                )
                .into());
            }
            outfile.write_all(&buffer[..bytes_read]).map_err(|e| {
                terr!("error-write-file", path = out_path.display().to_string(),).with_source(e)
            })?;
            task.inc(bytes_read as u64);
        }

        // Set executable permissions if applicable (Unix-like systems only)
        #[cfg(target_family = "unix")]
        if let Some(mode) = file.unix_mode() {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            let permissions = Permissions::from_mode(mode & 0o777);
            fs::set_permissions(&out_path, permissions).map_err(|e| {
                terr!(
                    "error-set-permissions",
                    path = out_path.display().to_string(),
                )
                .with_source(e)
            })?;
        }
    }

    drop(task);
    Ok(())
}
