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

use gdvm::godot_manager::GodotManager;
use gdvm::run_version_resolver::{RunResolutionRequest, RunVersionResolver};
use gdvm::version_utils::{self, VersionSpec, VersionTarget};
use gdvm::{println_i18n, t};

use anyhow::{Result, anyhow};
use clap::ArgMatches;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{check_deprecated_csharp_flag, keyword_to_version_filter};

/// Handle the 'link' subcommand
pub(crate) async fn sub_link(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version");
    let link_path_raw = matches
        .get_one::<String>("linkpath")
        .map(|s| s.as_str())
        .ok_or_else(|| unreachable!("clap should prevent missing required arg"))?;

    let spec = version_input.map(|v| VersionSpec::parse(v)).transpose()?;
    let spec_variant = spec.as_ref().and_then(|s| s.variant.clone());
    let variant = check_deprecated_csharp_flag(matches, spec_variant);
    let registry = spec.as_ref().and_then(|s| s.registry.clone());

    let explicit_version = match spec.as_ref().map(|s| &s.target) {
        Some(VersionTarget::Pattern(gv)) => Some(gv.clone()),
        Some(VersionTarget::Keyword(kw)) => Some(keyword_to_version_filter(kw)),
        None => None,
    };

    let force = matches.get_flag("force");
    let copy = matches.get_flag("copy");

    let resolver = RunVersionResolver::new(manager);
    let resolved = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            variant,
            registry,
            include_pre: false,
            possible_paths: &[],
            force_on_mismatch: force,
            install_if_missing: false,
        })
        .await?;

    let primary_exe = manager.get_executable_path(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
        false,
    )?;

    #[cfg(target_os = "windows")]
    let console_exe = {
        let console_exe = manager.get_executable_path(
            &resolved.version,
            &resolved.variant,
            resolved.registry.as_deref(),
            true,
        )?;
        if console_exe != primary_exe {
            Some(console_exe)
        } else {
            None
        }
    };

    let display = version_utils::display_version(
        &resolved.version.to_display_str(),
        &resolved.variant,
        resolved.registry.as_deref(),
    );

    // Key used to track this link against its install, so prune can preserve
    // installs that still have a live link.
    let install_key = manager.install_key(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
    )?;

    let link_path = PathBuf::from(link_path_raw);
    if let Some(parent) = link_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    #[cfg(target_os = "macos")]
    if let Some(bundle_target) = macos_bundle_from_executable(&primary_exe) {
        prepare_link_path(&link_path, force)?;
        link_or_copy_dir(&bundle_target, &link_path, copy)?;

        if copy {
            println_i18n!(
                "copy-created",
                version = display,
                path = link_path.display().to_string()
            );
        } else {
            manager.record_link(&link_path, &install_key)?;
            println_i18n!(
                "link-created",
                version = display,
                path = link_path.display().to_string()
            );
        }

        return Ok(());
    }

    prepare_link_path(&link_path, force)?;
    link_or_copy_file(&primary_exe, &link_path, copy)?;

    #[cfg(target_os = "windows")]
    if let Some(console_exe) = &console_exe
        && let Some(console_link) =
            build_console_link_path(&link_path).filter(|console_link| console_link != &link_path)
    {
        prepare_link_path(&console_link, force)?;
        link_or_copy_file(console_exe, &console_link, copy)?;
    }

    // Mirror any components that must live beside the executable.
    #[allow(unused_mut)]
    let mut linked_exes = vec![primary_exe.clone()];
    #[cfg(target_os = "windows")]
    if let Some(console_exe) = &console_exe {
        linked_exes.push(console_exe.clone());
    }
    link_sideloaded_components(&primary_exe, &link_path, &linked_exes, force, copy)?;

    if copy {
        println_i18n!(
            "copy-created",
            version = display,
            path = link_path.display().to_string()
        );
    } else {
        manager.record_link(&link_path, &install_key)?;
        println_i18n!(
            "link-created",
            version = display,
            path = link_path.display().to_string()
        );
    }

    Ok(())
}

/// Extract possible project paths from raw arguments passed after "--". If a --path flag is present
/// with a following argument, that single value wins. Otherwise, collect arguments that do not look
/// like flags, i.e. those that do not start with "-".
pub(crate) fn collect_possible_paths(raw_args: &[String]) -> Vec<&str> {
    if let Some(index) = raw_args.iter().position(|arg| arg == "--path")
        && let Some(path_arg) = raw_args.get(index + 1)
    {
        return vec![path_arg.as_str()];
    }

    raw_args
        .iter()
        .filter_map(|arg| {
            if arg.starts_with('-') {
                None
            } else {
                Some(arg.as_str())
            }
        })
        .collect()
}

/// Mirror every component that sits beside the installed executable.
fn link_sideloaded_components(
    primary_exe: &Path,
    link_path: &Path,
    already_linked: &[PathBuf],
    force: bool,
    copy: bool,
) -> Result<()> {
    let (Some(src_dir), Some(dst_dir)) = (primary_exe.parent(), link_path.parent()) else {
        return Ok(());
    };

    let mut entries: Vec<_> = fs::read_dir(src_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| !already_linked.iter().any(|linked| linked == &e.path()))
        .collect();
    // Deterministic order for scripts and tests.
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let src = entry.path();
        let dest = dst_dir.join(entry.file_name());
        prepare_link_path(&dest, force)?;
        if entry.file_type()?.is_dir() {
            link_or_copy_dir(&src, &dest, copy)?;
        } else {
            link_or_copy_file(&src, &dest, copy)?;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_bundle_from_executable(exe: &Path) -> Option<PathBuf> {
    let macos_dir = exe.parent()?;
    if macos_dir.file_name().is_some_and(|n| n != "MacOS") {
        return None;
    }
    let contents_dir = macos_dir.parent()?;
    if contents_dir.file_name().is_some_and(|n| n != "Contents") {
        return None;
    }
    contents_dir.parent().map(|p| p.to_path_buf())
}

fn link_or_copy_file(target: &Path, link: &Path, copy: bool) -> Result<()> {
    if copy {
        fs::copy(target, link)
            .map_err(|e| anyhow!(t!("error-link-copy", error = e.to_string())))?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).map_err(|e| {
            anyhow!(t!(
                "error-link-symlink",
                error = e.to_string(),
                target = target.display().to_string(),
                link = link.display().to_string()
            ))
        })?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| {
            anyhow!(t!(
                "error-link-symlink",
                error = e.to_string(),
                target = target.display().to_string(),
                link = link.display().to_string()
            ))
        })?;
    }

    Ok(())
}

fn link_or_copy_dir(target: &Path, link: &Path, copy: bool) -> Result<()> {
    if copy {
        copy_dir_recursive(target, link)?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link).map_err(|e| {
            anyhow!(t!(
                "error-link-symlink",
                error = e.to_string(),
                target = target.display().to_string(),
                link = link.display().to_string()
            ))
        })?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| {
            anyhow!(t!(
                "error-link-symlink",
                error = e.to_string(),
                target = target.display().to_string(),
                link = link.display().to_string()
            ))
        })?;
    }

    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

fn prepare_link_path(link_path: &Path, force: bool) -> Result<()> {
    if link_path.exists() {
        if !force {
            return Err(anyhow!(t!(
                "error-link-exists",
                path = link_path.display().to_string()
            )));
        }
        if link_path.is_dir() {
            fs::remove_dir_all(link_path)?;
        } else {
            fs::remove_file(link_path)?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn build_console_link_path(base_link: &Path) -> Option<PathBuf> {
    let stem = base_link.file_stem()?.to_string_lossy();
    let mut name = format!("{stem}_console");

    if let Some(ext) = base_link.extension() {
        name.push('.');
        name.push_str(&ext.to_string_lossy());
    }

    Some(base_link.with_file_name(name))
}

#[cfg(test)]
mod tests {

    use super::collect_possible_paths;
    use super::link_sideloaded_components;
    use std::fs;
    use std::path::PathBuf;

    fn to_vec(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn collects_non_flag_arguments() {
        let args = to_vec(&["--verbose", "project", "another"]);
        assert_eq!(collect_possible_paths(&args), vec!["project", "another"]);
    }

    #[test]
    fn prefers_explicit_path_flag() {
        let args = to_vec(&["foo", "--path", "explicit", "bar"]);
        assert_eq!(collect_possible_paths(&args), vec!["explicit"]);
    }

    #[test]
    fn missing_path_value_keeps_existing_candidates() {
        let args = to_vec(&["project", "--path"]);
        assert_eq!(collect_possible_paths(&args), vec!["project"]);
    }

    #[test]
    fn sideloaded_components_are_mirrored_for_any_variant() {
        let tmp = tempfile::TempDir::new().unwrap();

        let install = tmp.path().join("install");
        fs::create_dir_all(install.join("GodotSharp/Api")).unwrap();
        fs::write(install.join("GodotSharp/Api/Core.dll"), b"dll").unwrap();
        fs::create_dir_all(install.join("data")).unwrap();
        fs::write(install.join("data/pack.bin"), b"pack").unwrap();
        fs::write(install.join("extra.txt"), b"hello").unwrap();
        let exe = install.join("Godot_v4.4-stable_linux.x86_64");
        fs::write(&exe, b"binary").unwrap();

        let bindir = tmp.path().join("bin");
        fs::create_dir_all(&bindir).unwrap();
        let link = bindir.join("godot");
        fs::write(&link, b"binary").unwrap();

        let already: Vec<PathBuf> = vec![exe.clone()];
        link_sideloaded_components(&exe, &link, &already, false, true).unwrap();

        assert!(bindir.join("GodotSharp/Api/Core.dll").is_file());
        assert!(bindir.join("data/pack.bin").is_file());
        assert!(bindir.join("extra.txt").is_file());
        assert!(!bindir.join("Godot_v4.4-stable_linux.x86_64").exists());
    }

    #[test]
    fn sideloaded_components_noop_without_siblings() {
        let tmp = tempfile::TempDir::new().unwrap();

        let install = tmp.path().join("install");
        fs::create_dir_all(&install).unwrap();
        let exe = install.join("Godot_v4.4-stable_linux.x86_64");
        fs::write(&exe, b"binary").unwrap();

        let bindir = tmp.path().join("bin");
        fs::create_dir_all(&bindir).unwrap();
        let link = bindir.join("godot");
        fs::write(&link, b"binary").unwrap();

        link_sideloaded_components(&exe, &link, std::slice::from_ref(&exe), false, true).unwrap();

        let extra: Vec<_> = fs::read_dir(&bindir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .filter(|n| n != "godot")
            .collect();
        assert!(extra.is_empty(), "unexpected extra entries: {extra:?}");
    }
}
