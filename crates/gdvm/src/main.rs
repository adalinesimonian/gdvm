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

use gdvm::config::{self, ConfigOps};
use gdvm::godot_manager::{GodotManager, InstallOutcome};
use gdvm::i18n::I18n;
use gdvm::run_version_resolver::{
    RunResolutionRequest, RunVersionResolver, warn_project_version_mismatch,
};
use gdvm::version_utils::{self, GodotVersion, VersionSpec, VersionTarget};
use gdvm::{eprintln_i18n, println_i18n, t};

use anyhow::{Result, anyhow};
use clap::{Arg, ArgMatches, Command, value_parser};
use dotenvy::dotenv;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn refresh_flag(i18n: &I18n) -> Arg {
    Arg::new("refresh")
        .long("refresh")
        .num_args(0)
        .help(t!(i18n, "help-refresh-flag"))
}

fn include_pre_flag(i18n: &I18n) -> Arg {
    Arg::new("include-pre")
        .long("include-pre")
        .visible_alias("pre")
        .short('p')
        .num_args(0)
        .help(t!(i18n, "help-include-pre"))
}

fn deprecated_csharp_flag(i18n: &I18n) -> Arg {
    Arg::new("csharp")
        .long("csharp")
        .num_args(0)
        .hide(true)
        .help(t!(i18n, "help-csharp"))
}

fn deprecated_csharp_flag_with_value(i18n: &I18n) -> Arg {
    Arg::new("csharp")
        .long("csharp")
        .value_parser(value_parser!(bool))
        .num_args(0..=1)
        .default_missing_value("true")
        .default_value("false")
        .require_equals(true)
        .hide(true)
        .help(t!(i18n, "help-csharp"))
        .long_help(t!(i18n, "help-run-csharp-long"))
}

/// Check if the deprecated `--csharp` flag was explicitly provided.
fn check_deprecated_csharp_flag(
    i18n: &I18n,
    matches: &ArgMatches,
    spec_variant: Option<String>,
) -> Option<String> {
    let explicitly_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    if !explicitly_given {
        return spec_variant;
    }
    eprintln_i18n!(i18n, "warning-deprecated-csharp-flag");

    // If the new variant field was used, it takes precedence.
    if spec_variant.is_some() {
        return spec_variant;
    }

    if matches.get_flag("csharp") {
        Some("csharp".to_string())
    } else {
        None
    }
}

async fn refresh_cache_if_requested(manager: &GodotManager<'_>, refresh: bool) -> Result<()> {
    if refresh {
        manager.refresh_cache().await?;
    }

    Ok(())
}

/// Convert a keyword to a `GodotVersion` filter for resolution.
fn keyword_to_version_filter(keyword: &str) -> GodotVersion {
    if keyword == "stable" {
        GodotVersion {
            release_type: Some("stable".to_string()),
            ..Default::default()
        }
    } else {
        GodotVersion::default()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenv().ok();
    let i18n = I18n::new(100)?;
    let manager = GodotManager::new(&i18n).await?;

    // Detect if running through "godot" or "godot_console" shim.
    let exe_name = std::env::var("GDVM_ALIAS").ok().unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_default()
            .to_lowercase()
    });

    if exe_name.contains("godot") {
        // Forward all args (skip clap) and treat it like "gdvm run"

        #[cfg(target_os = "windows")]
        let console_mode = exe_name.contains("console");

        #[cfg(not(target_os = "windows"))]
        let console_mode = true;

        // Pass all arguments to Godot
        let args: Vec<String> = std::env::args().skip(1).collect();

        if let Err(err) = sub_run_inner(RunConfig {
            i18n: &i18n,
            manager: &GodotManager::new(&i18n).await?,
            version_input: None,
            variant: None,
            console: console_mode,
            raw_args: &args,
            force_on_mismatch: false,
            include_pre: false,
        })
        .await
        {
            eprintln!("{err}");

            // Wait for 5 seconds before exiting on Windows to allow the user to read the error.
            // On other platforms, the wrapper script will display the error message in a dialog.
            #[cfg(target_os = "windows")]
            std::thread::sleep(std::time::Duration::from_secs(5));

            std::process::exit(1);
        }

        return Ok(());
    }

    let matches = Command::new("gdvm")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Adaline Simonian <adalinesimonian@gmail.com>")
        .about(t!(i18n, "help-about"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .max_term_width(100)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(clap::ArgAction::Help)
                .global(true)
                .help(t!(i18n, "help-help")),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(t!(i18n, "help-gdvm-version")),
        )
        .subcommand(
            Command::new("install")
                .about(t!(i18n, "help-install"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!(i18n, "help-version"))
                        .long_help(t!(i18n, "help-version-long")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!(i18n, "help-force")),
                )
                .arg(
                    Arg::new("redownload")
                        .long("redownload")
                        .num_args(0)
                        .help(t!(i18n, "help-redownload")),
                )
                .arg(
                    Arg::new("launch-shortcut")
                        .long("launch-shortcut")
                        .num_args(0)
                        .help(t!(i18n, "help-launch-shortcut")),
                )
                .arg(deprecated_csharp_flag(&i18n))
                .arg(include_pre_flag(&i18n))
                .arg(refresh_flag(&i18n)),
        )
        .subcommand(Command::new("list").about(t!(i18n, "help-list")))
        .subcommand(
            Command::new("run")
                .about(t!(i18n, "help-run"))
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!(i18n, "help-version-installed")),
                )
                .arg(
                    Arg::new("console")
                        .short('c')
                        .long("console")
                        .value_parser(value_parser!(bool))
                        .num_args(0..=1)
                        .default_missing_value("true")
                        .default_value(
                            #[cfg(target_os = "windows")]
                            "false",
                            #[cfg(not(target_os = "windows"))]
                            "true",
                        )
                        .require_equals(true)
                        .help(t!(i18n, "help-console")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!(i18n, "help-run-force"))
                        .long_help(t!(i18n, "help-run-force-long")),
                )
                // Allow any number of command line arguments to be passed to the Godot executable after "--"
                .arg(
                    Arg::new("args")
                        .num_args(0..)
                        .last(true)
                        .help(t!(i18n, "help-run-args")),
                )
                .arg(deprecated_csharp_flag_with_value(&i18n))
                .arg(include_pre_flag(&i18n))
                .arg(refresh_flag(&i18n)),
        )
        .subcommand(
            Command::new("show")
                .about(t!(i18n, "help-show"))
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!(i18n, "help-version-installed")),
                )
                .arg(
                    Arg::new("console")
                        .short('c')
                        .long("console")
                        .value_parser(value_parser!(bool))
                        .num_args(0..=1)
                        .default_missing_value("true")
                        .default_value(
                            #[cfg(target_os = "windows")]
                            "false",
                            #[cfg(not(target_os = "windows"))]
                            "true",
                        )
                        .require_equals(true)
                        .help(t!(i18n, "help-console")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!(i18n, "help-run-force"))
                        .long_help(t!(i18n, "help-run-force-long")),
                )
                .arg(deprecated_csharp_flag_with_value(&i18n))
                .arg(include_pre_flag(&i18n))
                .arg(refresh_flag(&i18n)),
        )
        .subcommand(
            Command::new("link")
                .about(t!(i18n, "help-link"))
                .override_usage("gdvm link [OPTIONS] [version] <linkpath>")
                .allow_missing_positional(true)
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!(i18n, "help-link-version")),
                )
                .arg({
                    let platform = {
                        #[cfg(target_os = "windows")]
                        {
                            "windows"
                        }
                        #[cfg(target_os = "macos")]
                        {
                            "macos"
                        }
                        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
                        {
                            "other"
                        }
                    };

                    Arg::new("linkpath")
                        .required(true)
                        .value_name("linkpath")
                        .help(t!(i18n, "help-link-path", platform = platform))
                })
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!(i18n, "help-link-force")),
                )
                .arg(
                    Arg::new("copy")
                        .long("copy")
                        .num_args(0)
                        .help(t!(i18n, "help-link-copy")),
                )
                .arg(deprecated_csharp_flag_with_value(&i18n)),
        )
        .subcommand(
            Command::new("remove")
                .about(t!(i18n, "help-remove"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!(i18n, "help-version-installed")),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .num_args(0)
                        .help(t!(i18n, "help-yes")),
                )
                .arg(deprecated_csharp_flag(&i18n)),
        )
        .subcommand(
            Command::new("search")
                .about(t!(i18n, "help-search"))
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .num_args(1)
                        .help(t!(i18n, "help-filter")),
                )
                .arg(
                    Arg::new("include-pre")
                        .long("include-pre")
                        .visible_alias("pre")
                        .short('p')
                        .num_args(0)
                        .help(t!(i18n, "help-include-pre")),
                )
                .arg(
                    Arg::new("cache-only")
                        .long("cache-only")
                        .num_args(0)
                        .help(t!(i18n, "help-cache-only")),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .num_args(1)
                        .default_value("10")
                        .value_parser(clap::value_parser!(usize))
                        .help(t!(i18n, "help-limit")),
                )
                .arg(refresh_flag(&i18n)),
        )
        .subcommand(Command::new("clear-cache").about(t!(i18n, "help-clear-cache")))
        .subcommand(Command::new("refresh").about(t!(i18n, "help-refresh")))
        .subcommand(
            Command::new("use")
                .about(t!(i18n, "help-default"))
                .arg(
                    Arg::new("version")
                        .help(t!(i18n, "help-default-version"))
                        .required(false),
                )
                .arg(deprecated_csharp_flag(&i18n))
                .arg(include_pre_flag(&i18n))
                .arg(refresh_flag(&i18n)),
        )
        .subcommand(
            Command::new("upgrade").about(t!(i18n, "help-upgrade")).arg(
                Arg::new("major")
                    .long("major")
                    .short('m')
                    .num_args(0)
                    .help(t!(i18n, "help-upgrade-major")),
            ),
        )
        .subcommand(
            Command::new("pin")
                .about(t!(i18n, "help-pin"))
                .long_about(t!(i18n, "help-pin-long"))
                .arg(
                    Arg::new("version")
                        .help(t!(i18n, "help-pin-version"))
                        .required(true),
                )
                .arg(deprecated_csharp_flag(&i18n))
                .arg(include_pre_flag(&i18n))
                .arg(refresh_flag(&i18n))
                .arg(
                    Arg::new("no-legacy")
                        .long("no-legacy")
                        .num_args(0)
                        .help(t!(i18n, "help-no-legacy")),
                ),
        )
        .subcommand(
            Command::new("config")
                .about(t!(i18n, "help-config"))
                .subcommand(
                    Command::new("get").about(t!(i18n, "help-config-get")).arg(
                        Arg::new("key")
                            .required(true)
                            .help(t!(i18n, "help-config-key")),
                    ),
                )
                .subcommand(
                    Command::new("set")
                        .about(t!(i18n, "help-config-set"))
                        .arg(
                            Arg::new("key")
                                .required(true)
                                .help(t!(i18n, "help-config-key")),
                        )
                        .arg(
                            Arg::new("value")
                                .required(false)
                                .help(t!(i18n, "help-config-value")),
                        ),
                )
                .subcommand(
                    Command::new("unset")
                        .about(t!(i18n, "help-config-unset"))
                        .arg(
                            Arg::new("key")
                                .required(true)
                                .help(t!(i18n, "help-config-unset-key")),
                        ),
                )
                .subcommand(
                    Command::new("list")
                        .about(t!(i18n, "help-config-list"))
                        .arg(
                            Arg::new("show-sensitive")
                                .long("show-sensitive")
                                .action(clap::ArgAction::SetTrue)
                                .help(t!(i18n, "help-config-show-sensitive")),
                        )
                        .arg(
                            Arg::new("available")
                                .long("available")
                                .short('a')
                                .action(clap::ArgAction::SetTrue)
                                .help(t!(i18n, "help-config-available")),
                        ),
                ),
        )
        .get_matches();

    // Match the subcommand and call the appropriate function
    match matches.subcommand() {
        Some(("install", sub_m)) => sub_install(&i18n, &manager, sub_m).await?,
        Some(("list", _)) => sub_list(&i18n, &manager)?,
        Some(("run", sub_m)) => sub_run(&i18n, &manager, sub_m).await?,
        Some(("show", sub_m)) => sub_show(&i18n, &manager, sub_m).await?,
        Some(("link", sub_m)) => sub_link(&i18n, &manager, sub_m).await?,
        Some(("remove", sub_m)) => sub_remove(&i18n, &manager, sub_m).await?,
        Some(("search", sub_m)) => sub_search(&i18n, &manager, sub_m).await?,
        Some(("clear-cache", _)) => sub_clear_cache(&i18n, &manager)?,
        Some(("refresh", _)) => sub_refresh(&i18n, &manager).await?,
        Some(("use", sub_m)) => sub_use(&i18n, &manager, sub_m).await?,
        Some(("upgrade", sub_m)) => sub_upgrade(&manager, sub_m).await?,
        Some(("pin", sub_m)) => sub_pin(&i18n, &manager, sub_m).await?,
        Some(("config", sub_m)) => sub_config(&i18n, sub_m)?,
        _ => {}
    }

    Ok(())
}

/// Handle the 'install' subcommand
async fn sub_install(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let force_reinstall = matches.get_flag("force");
    let redownload = matches.get_flag("redownload");
    let refresh = matches.get_flag("refresh");
    let launch_shortcut = matches.get_flag("launch-shortcut")
        || config::Config::load(i18n)?.global_launch_shortcut.is_some();
    let include_pre = matches.get_flag("include-pre");

    refresh_cache_if_requested(manager, refresh).await?;

    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(i18n, matches, spec.variant);
    let variant = variant.as_deref();

    let requested_version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let gv = manager
        .resolve_available_version(&requested_version, variant, include_pre, false)
        .await?
        .ok_or_else(|| anyhow!(t!(i18n, "error-version-not-found")))?;

    let display = version_utils::display_with_variant(&gv.to_display_str(), variant);

    // Print a message indicating the start of the installation process
    println_i18n!(i18n, "installing-version", version = &display);

    match manager
        .install(&gv, force_reinstall, redownload, launch_shortcut)
        .await?
    {
        InstallOutcome::Installed => {
            // Print a message indicating the successful installation
            println_i18n!(i18n, "installed-success", version = &display);
        }
        InstallOutcome::AlreadyInstalled => {
            // Print a message indicating the version is already installed
            println_i18n!(i18n, "version-already-installed", version = &display);
            return Ok(());
        }
    }

    Ok(())
}

/// Handle the 'list' subcommand
fn sub_list(i18n: &I18n, manager: &GodotManager) -> Result<()> {
    let versions = manager.list_installed()?;
    if versions.is_empty() {
        println_i18n!(i18n, "no-versions-installed");
    } else {
        println_i18n!(i18n, "installed-versions");
        for (v, variant) in &versions {
            println!(
                "- {}",
                version_utils::display_with_variant(&v.to_display_str(), variant.as_deref())
            );
        }
    }
    Ok(())
}

/// Handle the 'run' subcommand
async fn sub_run(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    // Capture args after "--" to pass directly to child
    let raw_args = match std::env::args().position(|x| x == "--") {
        Some(pos) => std::env::args().skip(pos + 1).collect(),
        None => Vec::new(),
    };

    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let spec_variant = version_input
        .map(|v| VersionSpec::parse(v))
        .transpose()?
        .and_then(|s| s.variant);

    let variant = check_deprecated_csharp_flag(i18n, matches, spec_variant);
    let include_pre = matches.get_flag("include-pre");

    sub_run_inner(RunConfig {
        i18n,
        manager,
        version_input,
        variant,
        console,
        raw_args: &raw_args,
        force_on_mismatch,
        include_pre,
    })
    .await
}

/// Handle the 'show' subcommand
async fn sub_show(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let raw_args: Vec<String> = Vec::new();

    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let possible_paths = collect_possible_paths(&raw_args);

    let spec = version_input.map(|v| VersionSpec::parse(v)).transpose()?;
    let spec_variant = spec.as_ref().and_then(|s| s.variant.clone());
    let variant = check_deprecated_csharp_flag(i18n, matches, spec_variant);

    let explicit_version = match spec.as_ref().map(|s| &s.target) {
        Some(VersionTarget::Pattern(gv)) => Some(gv.clone()),
        Some(VersionTarget::Keyword(kw)) => Some(keyword_to_version_filter(kw)),
        None => None,
    };

    let include_pre = matches.get_flag("include-pre");

    let resolver = RunVersionResolver::new(manager, i18n);
    let resolved = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            variant,
            include_pre,
            possible_paths: &possible_paths,
            force_on_mismatch,
            install_if_missing: false,
        })
        .await?;

    let exe_path =
        manager.get_executable_path(&resolved.version, resolved.variant.as_deref(), console)?;
    println!("{}", exe_path.display());

    Ok(())
}

/// Handle the 'link' subcommand
async fn sub_link(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version");
    let link_path_raw = matches
        .get_one::<String>("linkpath")
        .map(|s| s.as_str())
        .ok_or_else(|| unreachable!("clap should prevent missing required arg"))?;

    let spec = version_input.map(|v| VersionSpec::parse(v)).transpose()?;
    let spec_variant = spec.as_ref().and_then(|s| s.variant.clone());
    let variant = check_deprecated_csharp_flag(i18n, matches, spec_variant);

    let explicit_version = match spec.as_ref().map(|s| &s.target) {
        Some(VersionTarget::Pattern(gv)) => Some(gv.clone()),
        Some(VersionTarget::Keyword(kw)) => Some(keyword_to_version_filter(kw)),
        None => None,
    };

    let force = matches.get_flag("force");
    let copy = matches.get_flag("copy");

    let resolver = RunVersionResolver::new(manager, i18n);
    let resolved = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            variant,
            include_pre: false,
            possible_paths: &[],
            force_on_mismatch: force,
            install_if_missing: false,
        })
        .await?;

    let primary_exe =
        manager.get_executable_path(&resolved.version, resolved.variant.as_deref(), false)?;

    #[cfg(target_os = "windows")]
    let console_exe = {
        let console_exe =
            manager.get_executable_path(&resolved.version, resolved.variant.as_deref(), true)?;
        if console_exe != primary_exe {
            Some(console_exe)
        } else {
            None
        }
    };

    let display = version_utils::display_with_variant(
        &resolved.version.to_display_str(),
        resolved.variant.as_deref(),
    );

    let link_path = PathBuf::from(link_path_raw);
    if let Some(parent) = link_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    #[cfg(target_os = "macos")]
    if let Some(bundle_target) = macos_bundle_from_executable(&primary_exe) {
        prepare_link_path(&link_path, force, i18n)?;
        link_or_copy_dir(&bundle_target, &link_path, copy, i18n)?;

        if copy {
            println_i18n!(
                i18n,
                "copy-created",
                version = display,
                path = link_path.display().to_string()
            );
        } else {
            println_i18n!(
                i18n,
                "link-created",
                version = display,
                path = link_path.display().to_string()
            );
        }

        return Ok(());
    }

    prepare_link_path(&link_path, force, i18n)?;
    link_or_copy_file(&primary_exe, &link_path, copy, i18n)?;

    #[cfg(target_os = "windows")]
    if let Some(console_exe) = console_exe
        && let Some(console_link) =
            build_console_link_path(&link_path).filter(|console_link| console_link != &link_path)
    {
        prepare_link_path(&console_link, force, i18n)?;
        link_or_copy_file(&console_exe, &console_link, copy, i18n)?;
    }

    if resolved.variant.as_deref() == Some("csharp") {
        let godotsharp_target = find_godotsharp_dir(&primary_exe)
            .ok_or_else(|| anyhow!(t!(i18n, "error-link-godotsharp-missing")))?;

        let godotsharp_link = link_path
            .parent()
            .map(|p| p.join("GodotSharp"))
            .ok_or_else(|| anyhow!(t!(i18n, "error-link-godotsharp-target")))?;

        if godotsharp_link.exists() {
            if !force {
                return Err(anyhow!(t!(
                    i18n,
                    "error-link-exists",
                    path = godotsharp_link.display().to_string()
                )));
            }
            if godotsharp_link.is_dir() {
                fs::remove_dir_all(&godotsharp_link)?;
            } else {
                fs::remove_file(&godotsharp_link)?;
            }
        }

        link_or_copy_dir(&godotsharp_target, &godotsharp_link, copy, i18n)?;
    }

    if copy {
        println_i18n!(
            i18n,
            "copy-created",
            version = display,
            path = link_path.display().to_string()
        );
    } else {
        println_i18n!(
            i18n,
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
fn collect_possible_paths(raw_args: &[String]) -> Vec<&str> {
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

fn find_godotsharp_dir(primary_exe: &Path) -> Option<PathBuf> {
    let parent = primary_exe.parent()?;

    #[cfg(target_os = "macos")]
    {
        // macOS mono builds keep GodotSharp under Contents/Resources.
        if let Some(contents_dir) = parent.parent() {
            let resources_dir = contents_dir.join("Resources").join("GodotSharp");
            if resources_dir.exists() {
                return Some(resources_dir);
            }
        }
    }

    let sibling = parent.join("GodotSharp");
    if sibling.exists() {
        return Some(sibling);
    }

    None
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

fn link_or_copy_file(target: &Path, link: &Path, copy: bool, i18n: &I18n) -> Result<()> {
    if copy {
        fs::copy(target, link)
            .map_err(|e| anyhow!(t!(i18n, "error-link-copy", error = e.to_string())))?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).map_err(|e| {
            anyhow!(t!(
                i18n,
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
                i18n,
                "error-link-symlink",
                error = e.to_string(),
                target = target.display().to_string(),
                link = link.display().to_string()
            ))
        })?;
    }

    Ok(())
}

fn link_or_copy_dir(target: &Path, link: &Path, copy: bool, i18n: &I18n) -> Result<()> {
    if copy {
        copy_dir_recursive(target, link)?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link).map_err(|e| {
            anyhow!(t!(
                i18n,
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
                i18n,
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

fn prepare_link_path(link_path: &Path, force: bool, i18n: &I18n) -> Result<()> {
    if link_path.exists() {
        if !force {
            return Err(anyhow!(t!(
                i18n,
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

/// Configuration for the `sub_run_inner` function
struct RunConfig<'a> {
    i18n: &'a I18n,
    manager: &'a GodotManager<'a>,
    version_input: Option<&'a String>,
    variant: Option<String>,
    console: bool,
    raw_args: &'a Vec<String>,
    force_on_mismatch: bool,
    include_pre: bool,
}

/// Run the Godot executable
async fn sub_run_inner(config: RunConfig<'_>) -> Result<()> {
    let RunConfig {
        i18n,
        manager,
        version_input,
        variant,
        console,
        raw_args,
        force_on_mismatch,
        include_pre,
    } = config;

    // Try to see if a path was given in raw_args. First, by checking if the --path flag was given
    // and then by checking if the first argument is a path. Prefer the --path flag if both are
    // given.
    let possible_paths = collect_possible_paths(raw_args);

    let (explicit_version, resolved_variant) = if let Some(v) = version_input {
        let spec = VersionSpec::parse(v)?;
        let var = spec.variant.or(variant);
        let ver = match spec.target {
            VersionTarget::Pattern(gv) => Some(gv),
            VersionTarget::Keyword(ref kw) => Some(keyword_to_version_filter(kw)),
        };
        (ver, var)
    } else {
        (None, variant)
    };

    let resolver = RunVersionResolver::new(manager, i18n);
    let resolved = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            variant: resolved_variant,
            include_pre,
            possible_paths: &possible_paths,
            force_on_mismatch,
            install_if_missing: true,
        })
        .await?;

    let display = version_utils::display_with_variant(
        &resolved.version.to_display_str(),
        resolved.variant.as_deref(),
    );

    eprintln_i18n!(i18n, "running-version", version = &display);

    manager.run(
        &resolved.version,
        resolved.variant.as_deref(),
        console,
        raw_args,
    )?;

    Ok(())
}

/// Handle the 'remove' subcommand
async fn sub_remove(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(i18n, matches, spec.variant);
    let variant = variant.as_deref();

    let requested_version = match &spec.target {
        VersionTarget::Keyword(_) => {
            return Err(anyhow!(t!(i18n, "error-version-not-found")));
        }
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let resolved_versions = manager
        .resolve_installed_version(&requested_version, variant)
        .await?;

    match resolved_versions.len() {
        0 => {
            eprintln_i18n!(i18n, "error-version-not-found");
        }
        1 => {
            let (gv, var) = &resolved_versions[0];
            let display = version_utils::display_with_variant(&gv.to_display_str(), var.as_deref());

            println_i18n!(i18n, "removing-version", version = &display);

            if !matches.get_flag("yes") {
                println_i18n!(i18n, "confirm-remove");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != t!(i18n, "confirm-yes") {
                    println_i18n!(i18n, "remove-cancelled");
                    return Ok(());
                }
            }
            manager.remove(gv, var.as_deref())?;
            println_i18n!(i18n, "removed-version", version = &display);
        }
        _ => {
            eprintln_i18n!(i18n, "error-multiple-versions-found");
            for (v, var) in &resolved_versions {
                println!(
                    "- {}",
                    version_utils::display_with_variant(&v.to_display_str(), var.as_deref())
                );
            }
        }
    }

    Ok(())
}

/// Handle the 'search' subcommand
async fn sub_search(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let filter = matches.get_one::<String>("filter").map(|s| s.as_str());
    let include_pre = matches.get_flag("include-pre");
    let cache_only = matches.get_flag("cache-only");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let requested_version = match filter {
        Some(filter) => Some(GodotVersion::from_match_str(filter)?),
        None => None,
    };

    let mut releases = manager
        .fetch_available_releases(&requested_version, cache_only)
        .await?;

    // Default to showing only stable releases unless `--include-pre` is specified
    if !include_pre {
        releases.retain(|r| r.is_stable());
    }

    let limit = matches.get_one::<usize>("limit").unwrap();
    if *limit != 0 {
        releases = releases.into_iter().take(*limit).collect();
    }

    if releases.is_empty() {
        println_i18n!(i18n, "no-matching-releases");
    } else {
        println_i18n!(i18n, "available-releases");
        for r in releases {
            println!("- {r}");
        }
    }
    Ok(())
}

/// Handle the 'clear-cache' subcommand
fn sub_clear_cache(i18n: &I18n, manager: &GodotManager) -> Result<()> {
    manager.clear_cache()?;
    println_i18n!(i18n, "cache-cleared");
    Ok(())
}

/// Handle the 'refresh' subcommand
async fn sub_refresh(i18n: &I18n, manager: &GodotManager<'_>) -> Result<()> {
    manager.refresh_cache().await?;
    println_i18n!(i18n, "cache-refreshed");
    Ok(())
}

/// Handle the 'use' subcommand
async fn sub_use(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let refresh = matches.get_flag("refresh");

    let version_input = match matches.get_one::<String>("version") {
        Some(v) => v,
        None => {
            println_i18n!(i18n, "provide-version-or-unset");
            return Ok(());
        }
    };

    if version_input == "unset" {
        manager.unset_default()?;
        println_i18n!(i18n, "default-unset-success");
        return Ok(());
    }

    refresh_cache_if_requested(manager, refresh).await?;

    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(i18n, matches, spec.variant);
    let variant = variant.as_deref();

    let requested_version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let include_pre = matches.get_flag("include-pre");

    let resolved_version = manager
        .auto_install_version(&requested_version, variant, include_pre)
        .await?;

    manager.set_default(&resolved_version, variant)?;
    let display = version_utils::display_with_variant(&resolved_version.to_display_str(), variant);
    println_i18n!(i18n, "default-set-success", version = &display);

    Ok(())
}

/// Handle the 'upgrade' subcommand
async fn sub_upgrade(manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let allow_major = matches.get_flag("major");
    manager.upgrade(allow_major).await
}

/// Handle the 'pin' subcommand
async fn sub_pin(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let version_str = matches.get_one::<String>("version").unwrap();
    let refresh = matches.get_flag("refresh");
    let spec = VersionSpec::parse(version_str)?;
    let variant = check_deprecated_csharp_flag(i18n, matches, spec.variant);
    let variant = variant.as_deref();

    refresh_cache_if_requested(manager, refresh).await?;

    let version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    warn_project_version_mismatch::<_, &Path>(manager, i18n, &version, true, None).await;

    let include_pre = matches.get_flag("include-pre");

    let resolved_version = manager
        .auto_install_version(&version, variant, include_pre)
        .await?;

    let display = version_utils::display_with_variant(&resolved_version.to_display_str(), variant);

    let skip_gdvmrc = matches.get_flag("no-legacy");

    match manager.pin_version(&resolved_version, variant, skip_gdvmrc) {
        Ok(()) => println_i18n!(i18n, "pinned-success", version = &display),
        Err(_) => eprintln_i18n!(i18n, "error-pin-version-not-found", version = &display),
    }
    Ok(())
}

/// Handle the 'config' subcommand
fn sub_config(i18n: &I18n, matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let mut config = config::Config::load(i18n)?;
    match matches.subcommand() {
        Some(("get", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            if let Some(value) = config.get_value(key) {
                println!("{value}");
            } else {
                println!("{}", t!(i18n, "config-key-not-set"));
            }
        }
        Some(("set", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            // If the value argument is not provided, prompt the user.
            let value: String = if let Some(v) = sub_m.get_one::<String>("value") {
                v.clone()
            } else {
                // Build the prompt message from the Fluent bundle.
                let prompt = t!(i18n, "config-set-prompt", key = key.as_str());
                eprint!("{prompt} ");
                if config.is_sensitive_key(key) {
                    // Mask input for sensitive values.
                    match rpassword::prompt_password("") {
                        Ok(input) => input,
                        Err(err) => {
                            eprintln!("{}: {}", t!(i18n, "error-reading-input"), err);
                            return Ok(());
                        }
                    }
                } else {
                    // For non-sensitive values, read normally.
                    std::io::stdout().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            if config.is_sensitive_key(key) {
                eprintln_i18n!(i18n, "warning-setting-sensitive");
            }
            match config.set_value(key, &value) {
                Ok(()) => {
                    config.save(i18n)?;
                    println_i18n!(i18n, "config-set-success");
                }
                Err(_) => eprintln_i18n!(i18n, "error-unknown-config-key"),
            }
        }
        Some(("unset", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            match config.unset_value(key) {
                Ok(()) => {
                    config.save(i18n)?;
                    println_i18n!(i18n, "config-unset-success", key = key);
                }
                Err(_) => eprintln_i18n!(i18n, "error-unknown-config-key"),
            }
        }
        Some(("list", sub_m)) => {
            let show_sensitive = sub_m.get_flag("show-sensitive");
            let available = sub_m.get_flag("available");
            if available {
                // List all known keys whether set or not.
                for key in config::KNOWN_KEYS {
                    let value_opt = config.get_value(key);
                    let display_value =
                        match (value_opt, config.is_sensitive_key(key), show_sensitive) {
                            (Some(_), true, false) => "********".to_string(),
                            (Some(val), _, _) => val,
                            (None, _, _) => "<not set>".to_string(),
                        };
                    println!("{key} = {display_value}");
                }
            } else {
                // List only keys that are set.
                for (key, value, sensitive) in config.list_set_keys() {
                    let display_value = if sensitive && !show_sensitive {
                        "********".to_string()
                    } else {
                        value
                    };
                    println!("{key} = {display_value}");
                }
            }
        }
        _ => eprintln!("{}", t!(i18n, "error-invalid-config-subcommand")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::collect_possible_paths;

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
}
