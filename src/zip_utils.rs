use crate::i18n::I18n;
use anyhow::{anyhow, Result};
use fluent_bundle::FluentValue;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn extract_zip(zip_path: &Path, extract_to: &Path, i18n: &I18n) -> Result<()> {
    // Print extracting message
    println!("{}", i18n.t("operation-extracting"));

    // First pass: Calculate total uncompressed size and collect top-level entries
    let file = fs::File::open(zip_path).map_err(|e| {
        anyhow!(i18n.t_args(
            "error-open-zip",
            &[
                ("path", FluentValue::from(zip_path.display().to_string())),
                ("error", FluentValue::from(e.to_string()))
            ]
        ))
    })?;
    let mut archive = ZipArchive::new(&file).map_err(|e| {
        anyhow!(i18n.t_args(
            "error-read-zip",
            &[
                ("path", FluentValue::from(zip_path.display().to_string())),
                ("error", FluentValue::from(e.to_string()))
            ]
        ))
    })?;

    let mut total_size: u64 = 0;
    let mut top_level_entries = HashSet::new();
    let mut top_level_dirs = HashSet::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| {
            anyhow!(i18n.t_args(
                "error-access-file",
                &[
                    ("index", FluentValue::from(i)),
                    ("error", FluentValue::from(e.to_string()))
                ]
            ))
        })?;

        if let Some(enclosed) = file.enclosed_name() {
            if let Some(first_component) = enclosed.components().next() {
                let entry = first_component.as_os_str().to_string_lossy().to_string();
                top_level_entries.insert(entry.clone());

                if file.is_dir() {
                    top_level_dirs.insert(entry);
                }
            }
        } else {
            return Err(anyhow!(i18n.t("error-invalid-file-name")));
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
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Second pass: Extract files and update progress
    // Re-open the ZIP archive for extraction
    let file = fs::File::open(zip_path).map_err(|e| {
        anyhow!(i18n.t_args(
            "error-reopen-zip",
            &[
                ("path", FluentValue::from(zip_path.display().to_string())),
                ("error", FluentValue::from(e.to_string()))
            ]
        ))
    })?;
    let mut archive = ZipArchive::new(&file).map_err(|e| {
        anyhow!(i18n.t_args(
            "error-read-zip",
            &[
                ("path", FluentValue::from(zip_path.display().to_string())),
                ("error", FluentValue::from(e.to_string()))
            ]
        ))
    })?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            anyhow!(i18n.t_args(
                "error-access-file",
                &[
                    ("index", FluentValue::from(i)),
                    ("error", FluentValue::from(e.to_string()))
                ]
            ))
        })?;
        let path = file
            .enclosed_name()
            .ok_or_else(|| anyhow!(i18n.t("error-invalid-file-name")))?;

        // Skip empty or root entries to avoid creating files at extract_to
        if path.as_os_str().is_empty() || path == Path::new(".") {
            continue;
        }

        // Determine the output path, stripping the common prefix if it exists
        let out_path = if let Some(ref prefix) = common_prefix {
            // Ensure the file path starts with the prefix
            let stripped_path = path
                .strip_prefix(prefix)
                .map_err(|_| anyhow!(i18n.t("error-strip-prefix")))?;

            // Prevent extracting files outside the target directory
            if stripped_path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                return Err(anyhow!(i18n.t("error-invalid-file-name")));
            }

            extract_to.join(stripped_path)
        } else {
            extract_to.join(path)
        };

        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| {
                anyhow!(i18n.t_args(
                    "error-create-dir",
                    &[
                        ("path", FluentValue::from(out_path.display().to_string())),
                        ("error", FluentValue::from(e.to_string()))
                    ]
                ))
            })?;
            continue;
        }

        // Ensure the parent directory exists
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                anyhow!(i18n.t_args(
                    "error-create-dir",
                    &[
                        ("path", FluentValue::from(parent.display().to_string())),
                        ("error", FluentValue::from(e.to_string()))
                    ]
                ))
            })?;
        }

        // Create the output file
        let mut outfile = fs::File::create(&out_path).map_err(|e| {
            anyhow!(i18n.t_args(
                "error-create-file",
                &[
                    ("path", FluentValue::from(out_path.display().to_string())),
                    ("error", FluentValue::from(e.to_string()))
                ]
            ))
        })?;

        // Read from the ZIP file and write to the output file in chunks
        let mut buffer = [0; 8192]; // 8 KB buffer
        let reader = &mut file;
        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| {
                anyhow!(i18n.t_args(
                    "error-read-zip-file",
                    &[
                        ("file", FluentValue::from(reader.name().to_string())),
                        ("error", FluentValue::from(e.to_string()))
                    ]
                ))
            })?;
            if bytes_read == 0 {
                break; // Extraction complete for this file
            }
            outfile.write_all(&buffer[..bytes_read]).map_err(|e| {
                anyhow!(i18n.t_args(
                    "error-write-file",
                    &[
                        ("path", FluentValue::from(out_path.display().to_string())),
                        ("error", FluentValue::from(e.to_string()))
                    ]
                ))
            })?;
            pb.inc(bytes_read as u64);
        }

        // Set executable permissions if applicable (Unix-like systems only)
        #[cfg(target_family = "unix")]
        if let Some(mode) = file.unix_mode() {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            let permissions = Permissions::from_mode(mode);
            fs::set_permissions(&out_path, permissions).map_err(|e| {
                anyhow!(i18n.t_args(
                    "error-set-permissions",
                    &[
                        ("path", FluentValue::from(out_path.display().to_string())),
                        ("error", FluentValue::from(e.to_string()))
                    ]
                ))
            })?;
        }
    }

    pb.finish_with_message(i18n.t("operation-extract-complete"));
    Ok(())
}
