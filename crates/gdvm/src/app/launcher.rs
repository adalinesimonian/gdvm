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
#[cfg(target_family = "unix")]
use daemonize::Daemonize;

use super::*;
use crate::paths::GdvmPaths;
use crate::usage_tracker::UsageTracker;
use crate::version::{ResolvedVersion, Variant};

/// Searches for the Godot executable within the given directory.
///
/// ## Arguments
///
/// - `version_dir` - A reference to the directory path where the search is performed.
/// - `console` - A boolean indicating whether to search for the console version (relevant for Windows).
///
/// ## Returns
///
/// - `Ok(Some(PathBuf))` containing the path to the Godot executable if found.
/// - `Ok(None)` if no executable is found.
/// - `Err(io::Error)` if there is an error reading the directory.
#[allow(unused_variables)]
pub fn find_godot_executable(version_dir: &Path, console: bool) -> Result<Option<PathBuf>> {
    // Collect all entries (files/folders) under version_dir
    let entries: Vec<_> = fs::read_dir(version_dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    #[cfg(target_os = "windows")]
    {
        // If console is requested, try to find a "_console" executable first
        if console {
            let console_candidate = entries.iter().find_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("_console.exe") {
                    Some(entry.path())
                } else {
                    None
                }
            });

            // If found, return it.
            if console_candidate.is_some() {
                return Ok(console_candidate);
            }
        }

        // Prefer the non-console executable when available.
        let gui_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".exe") && !name.ends_with("_console.exe") {
                Some(entry.path())
            } else {
                None
            }
        });

        if gui_candidate.is_some() {
            return Ok(gui_candidate);
        }

        // Fall back to any .exe if nothing else matches.
        let exe_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".exe") {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(exe_candidate)
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, prefer an app bundle but return the executable inside it.
        let app_candidate = entries.iter().find_map(|entry| {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.ends_with(".app") {
                Some(entry.path())
            } else {
                None
            }
        });

        if let Some(app_path) = app_candidate
            && let Some(exe) = find_macos_app_executable(&app_path)
        {
            return Ok(Some(exe));
        }

        // Fall back to a Godot binary directly under the install dir.
        let binary_candidate = entries.iter().find_map(|entry| {
            let Ok(file_type) = entry.file_type() else {
                return None;
            };
            if !file_type.is_file() {
                return None;
            }

            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.starts_with("Godot") {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(binary_candidate)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // For Linux or other Unix-likes
        // Look for a few known suffixes or naming patterns
        let unix_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("Godot_v")
                || name.ends_with(".x86_64")
                || name.ends_with(".x86_32")
                || name.ends_with(".arm64")
            {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(unix_candidate)
    }
}

#[cfg(target_os = "macos")]
fn find_macos_app_executable(app_path: &Path) -> Option<PathBuf> {
    let macos_dir = app_path.join("Contents/MacOS");

    // Prefer known Godot binaries.
    let preferred = ["Godot", "Godot_mono"];
    for name in preferred {
        let candidate = macos_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    // Fall back to first regular file in Contents/MacOS.
    let entries = fs::read_dir(&macos_dir).ok()?;
    for entry in entries.flatten() {
        if entry.file_type().ok()?.is_file() {
            return Some(entry.path());
        }
    }

    None
}

#[derive(Clone, Copy)]
pub struct Launcher<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) usage_tracker: &'a UsageTracker,
    pub(super) catalogs: Catalogs<'a>,
    pub(super) dotenv_vars: &'a [(String, String)],
}

impl<'a> Launcher<'a> {
    fn library(&self) -> Library<'a> {
        Library {
            paths: self.paths,
            usage_tracker: self.usage_tracker,
            catalogs: self.catalogs,
        }
    }

    /// Run a specified Godot version
    pub fn run(
        &self,
        gv: &ResolvedVersion,
        variant: &Variant,
        registry: Option<&str>,
        console: bool,
        godot_args: &[String],
    ) -> Result<()> {
        let path = self
            .library()
            .get_executable_path(gv, variant, registry, console)?;

        let dotenv_vars = self
            .dotenv_vars
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()));

        if console {
            // Run the process attached to the terminal and wait for it to exit
            std::process::Command::new(&path)
                .args(godot_args)
                .envs(dotenv_vars)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;
        } else {
            // Detached process configuration
            #[cfg(target_family = "unix")]
            {
                Daemonize::new()
                    .start()
                    .map_err(|e| crate::terr!("error-starting-godot").with_source(e))?;
                std::process::Command::new(&path)
                    .args(godot_args)
                    .envs(dotenv_vars)
                    .spawn()?;
            }

            #[cfg(target_family = "windows")]
            {
                crate::process_utils::spawn_detached(
                    std::process::Command::new(&path)
                        .args(godot_args)
                        .envs(dotenv_vars),
                )?;
            }
        }

        Ok(())
    }
}
