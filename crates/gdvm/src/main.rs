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
use gdvm::godot_manager::{GodotManager, InstallOutcome, PruneOptions};
use gdvm::i18n::I18n;
use gdvm::registry;
use gdvm::run_version_resolver::{
    RunResolutionRequest, RunVersionResolver, warn_project_version_mismatch,
};
use gdvm::version_utils::{self, GodotVersion, Variant, VersionSpec, VersionTarget};
use gdvm::{eprintln_i18n, println_i18n, t};

use anyhow::{Result, anyhow};
use clap::{Arg, ArgMatches, Command, value_parser};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn refresh_flag() -> Arg {
    Arg::new("refresh")
        .long("refresh")
        .num_args(0)
        .help(t!("help-refresh-flag"))
}

fn yes_flag() -> Arg {
    Arg::new("yes")
        .short('y')
        .long("yes")
        .num_args(0)
        .help(t!("help-yes"))
}

fn include_pre_flag() -> Arg {
    Arg::new("include-pre")
        .long("include-pre")
        .visible_alias("pre")
        .short('p')
        .num_args(0)
        .help(t!("help-include-pre"))
}

fn deprecated_csharp_flag() -> Arg {
    Arg::new("csharp")
        .long("csharp")
        .num_args(0)
        .hide(true)
        .help(t!("help-csharp"))
}

fn deprecated_csharp_flag_with_value() -> Arg {
    Arg::new("csharp")
        .long("csharp")
        .value_parser(value_parser!(bool))
        .num_args(0..=1)
        .default_missing_value("true")
        .default_value("false")
        .require_equals(true)
        .hide(true)
        .help(t!("help-csharp"))
        .long_help(t!("help-run-csharp-long"))
}

/// Check if the deprecated `--csharp` flag was explicitly provided.
fn check_deprecated_csharp_flag(
    matches: &ArgMatches,
    spec_variant: Option<String>,
) -> Option<String> {
    let explicitly_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    if !explicitly_given {
        return spec_variant;
    }
    eprintln_i18n!("warning-deprecated-csharp-flag");

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

async fn refresh_cache_if_requested(manager: &GodotManager, refresh: bool) -> Result<()> {
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
    I18n::init()?;
    let manager = GodotManager::new().await?;

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
            manager: &GodotManager::new().await?,
            version_input: None,
            variant: None,
            console: console_mode,
            raw_args: &args,
            force_on_mismatch: false,
            include_pre: false,
            assume_yes: false,
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
        .about(t!("help-about"))
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
                .help(t!("help-help")),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(t!("help-gdvm-version")),
        )
        .subcommand(
            Command::new("install")
                .about(t!("help-install"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!("help-version"))
                        .long_help(t!("help-version-long")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!("help-force")),
                )
                .arg(
                    Arg::new("redownload")
                        .long("redownload")
                        .num_args(0)
                        .help(t!("help-redownload")),
                )
                .arg(deprecated_csharp_flag())
                .arg(include_pre_flag())
                .arg(refresh_flag())
                .arg(yes_flag()),
        )
        .subcommand(Command::new("list").about(t!("help-list")))
        .subcommand(
            Command::new("run")
                .about(t!("help-run"))
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!("help-version-installed")),
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
                        .help(t!("help-console")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!("help-run-force"))
                        .long_help(t!("help-run-force-long")),
                )
                // Allow any number of command line arguments to be passed to the Godot executable after "--"
                .arg(
                    Arg::new("args")
                        .num_args(0..)
                        .last(true)
                        .help(t!("help-run-args")),
                )
                .arg(deprecated_csharp_flag_with_value())
                .arg(include_pre_flag())
                .arg(refresh_flag())
                .arg(yes_flag()),
        )
        .subcommand(
            Command::new("show")
                .about(t!("help-show"))
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!("help-version-installed")),
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
                        .help(t!("help-console")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!("help-run-force"))
                        .long_help(t!("help-run-force-long")),
                )
                .arg(deprecated_csharp_flag_with_value())
                .arg(include_pre_flag())
                .arg(refresh_flag())
                .arg(yes_flag()),
        )
        .subcommand(
            Command::new("cache-path")
                .about(t!("help-cache-path"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!("help-version")),
                )
                .arg(deprecated_csharp_flag_with_value())
                .arg(include_pre_flag())
                .arg(refresh_flag())
                .arg(yes_flag()),
        )
        .subcommand(
            Command::new("link")
                .about(t!("help-link"))
                .override_usage("gdvm link [OPTIONS] [version] <linkpath>")
                .allow_missing_positional(true)
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!("help-link-version")),
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
                        .help(t!("help-link-path", platform = platform))
                })
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!("help-link-force")),
                )
                .arg(
                    Arg::new("copy")
                        .long("copy")
                        .num_args(0)
                        .help(t!("help-link-copy")),
                )
                .arg(deprecated_csharp_flag_with_value())
                .arg(yes_flag()),
        )
        .subcommand(
            Command::new("remove")
                .about(t!("help-remove"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_version_spec)
                        .help(t!("help-version-installed")),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .num_args(0)
                        .help(t!("help-yes")),
                )
                .arg(deprecated_csharp_flag()),
        )
        .subcommand(
            Command::new("search")
                .about(t!("help-search"))
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .num_args(1)
                        .help(t!("help-filter")),
                )
                .arg(
                    Arg::new("include-pre")
                        .long("include-pre")
                        .visible_alias("pre")
                        .short('p')
                        .num_args(0)
                        .help(t!("help-include-pre")),
                )
                .arg(
                    Arg::new("cache-only")
                        .long("cache-only")
                        .num_args(0)
                        .help(t!("help-cache-only")),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .num_args(1)
                        .default_value("10")
                        .value_parser(clap::value_parser!(usize))
                        .help(t!("help-limit")),
                )
                .arg(refresh_flag())
                .arg(yes_flag()),
        )
        .subcommand(Command::new("clear-cache").about(t!("help-clear-cache")))
        .subcommand(Command::new("refresh").about(t!("help-refresh")))
        .subcommand(
            Command::new("prune")
                .about(t!("help-prune"))
                .long_about(t!(
                    "help-prune-long",
                    default_days = config::DEFAULT_PRUNE_MAX_AGE_DAYS
                ))
                .arg(
                    Arg::new("all")
                        .long("all")
                        .short('a')
                        .num_args(0)
                        .help(t!("help-prune-all")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(t!("help-prune-force")),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .num_args(0)
                        .help(t!("help-prune-dry-run")),
                ),
        )
        .subcommand(
            Command::new("use")
                .about(t!("help-default"))
                .arg(
                    Arg::new("version")
                        .help(t!("help-default-version"))
                        .required(false),
                )
                .arg(deprecated_csharp_flag())
                .arg(include_pre_flag())
                .arg(refresh_flag())
                .arg(yes_flag()),
        )
        .subcommand(
            Command::new("upgrade")
                .about(t!("help-upgrade"))
                .arg(
                    Arg::new("major")
                        .long("major")
                        .short('m')
                        .num_args(0)
                        .help(t!("help-upgrade-major")),
                )
                .arg(
                    Arg::new("pre")
                        .long("pre")
                        .short('p')
                        .num_args(0)
                        .help(t!("help-upgrade-pre")),
                ),
        )
        .subcommand(
            Command::new("pin")
                .about(t!("help-pin"))
                .long_about(t!("help-pin-long"))
                .arg(
                    Arg::new("version")
                        .help(t!("help-pin-version"))
                        .required(true),
                )
                .arg(deprecated_csharp_flag())
                .arg(include_pre_flag())
                .arg(refresh_flag())
                .arg(
                    Arg::new("no-legacy")
                        .long("no-legacy")
                        .num_args(0)
                        .help(t!("help-no-legacy")),
                )
                .arg(yes_flag()),
        )
        .subcommand(
            Command::new("config")
                .about(t!("help-config"))
                .subcommand(
                    Command::new("get")
                        .about(t!("help-config-get"))
                        .arg(Arg::new("key").required(true).help(t!("help-config-key"))),
                )
                .subcommand(
                    Command::new("set")
                        .about(t!("help-config-set"))
                        .arg(Arg::new("key").required(true).help(t!("help-config-key")))
                        .arg(
                            Arg::new("value")
                                .required(false)
                                .help(t!("help-config-value")),
                        ),
                )
                .subcommand(
                    Command::new("unset").about(t!("help-config-unset")).arg(
                        Arg::new("key")
                            .required(true)
                            .help(t!("help-config-unset-key")),
                    ),
                )
                .subcommand(
                    Command::new("list")
                        .about(t!("help-config-list"))
                        .arg(
                            Arg::new("show-sensitive")
                                .long("show-sensitive")
                                .action(clap::ArgAction::SetTrue)
                                .help(t!("help-config-show-sensitive")),
                        )
                        .arg(
                            Arg::new("available")
                                .long("available")
                                .short('a')
                                .action(clap::ArgAction::SetTrue)
                                .help(t!("help-config-available")),
                        ),
                ),
        )
        .subcommand(
            Command::new("registry")
                .about(t!("help-registry"))
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("add")
                        .about(t!("help-registry-add"))
                        .arg(
                            Arg::new("name")
                                .required(true)
                                .help(t!("help-registry-name")),
                        )
                        .arg(Arg::new("url").required(true).help(t!("help-registry-url"))),
                )
                .subcommand(
                    Command::new("remove")
                        .about(t!("help-registry-remove"))
                        .arg(
                            Arg::new("name")
                                .required(true)
                                .help(t!("help-registry-name")),
                        ),
                )
                .subcommand(Command::new("list").about(t!("help-registry-list")))
                .subcommand(
                    Command::new("refresh")
                        .about(t!("help-registry-refresh"))
                        .arg(
                            Arg::new("name")
                                .required(false)
                                .help(t!("help-registry-name")),
                        ),
                )
                .subcommand(
                    Command::new("init")
                        .about(t!("help-registry-init"))
                        .arg(Arg::new("dir").required(true).help(t!("help-registry-dir")))
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .num_args(1)
                                .help(t!("help-registry-init-name")),
                        ),
                )
                .subcommand(
                    Command::new("add-build")
                        .about(t!("help-registry-add-build"))
                        .arg(Arg::new("dir").required(true).help(t!("help-registry-dir")))
                        .arg(
                            Arg::new("version")
                                .long("version")
                                .num_args(1)
                                .required(true)
                                .help(t!("help-registry-build-version")),
                        )
                        .arg(
                            Arg::new("variant")
                                .long("variant")
                                .num_args(1)
                                .help(t!("help-registry-build-variant")),
                        )
                        .arg(
                            Arg::new("platform")
                                .long("platform")
                                .num_args(1)
                                .required(true)
                                .help(t!("help-registry-build-platform")),
                        )
                        .arg(
                            Arg::new("file")
                                .long("file")
                                .num_args(1)
                                .required(false)
                                .help(t!("help-registry-build-file")),
                        )
                        .arg(
                            Arg::new("store")
                                .long("store")
                                .num_args(0)
                                .help(t!("help-registry-build-store")),
                        )
                        .arg(
                            Arg::new("url")
                                .long("url")
                                .num_args(1)
                                .conflicts_with("store")
                                .help(t!("help-registry-build-url")),
                        )
                        .arg(
                            Arg::new("sha512")
                                .long("sha512")
                                .num_args(1)
                                .help(t!("help-registry-build-sha512")),
                        )
                        .arg(
                            Arg::new("size")
                                .long("size")
                                .num_args(1)
                                .value_parser(value_parser!(u64))
                                .help(t!("help-registry-build-size")),
                        ),
                )
                .subcommand(
                    Command::new("remove-build")
                        .about(t!("help-registry-remove-build"))
                        .arg(Arg::new("dir").required(true).help(t!("help-registry-dir")))
                        .arg(
                            Arg::new("version")
                                .long("version")
                                .num_args(1)
                                .required(true)
                                .help(t!("help-registry-build-version")),
                        )
                        .arg(
                            Arg::new("variant")
                                .long("variant")
                                .num_args(1)
                                .help(t!("help-registry-build-variant")),
                        )
                        .arg(
                            Arg::new("platform")
                                .long("platform")
                                .num_args(1)
                                .help(t!("help-registry-build-platform")),
                        ),
                )
                .subcommand(
                    Command::new("validate")
                        .about(t!("help-registry-validate"))
                        .arg(Arg::new("dir").required(true).help(t!("help-registry-dir"))),
                ),
        )
        .get_matches();

    // Match the subcommand and call the appropriate function
    match matches.subcommand() {
        Some(("install", sub_m)) => sub_install(&manager, sub_m).await?,
        Some(("list", _)) => sub_list(&manager)?,
        Some(("run", sub_m)) => sub_run(&manager, sub_m).await?,
        Some(("show", sub_m)) => sub_show(&manager, sub_m).await?,
        Some(("cache-path", sub_m)) => sub_cache_path(&manager, sub_m).await?,
        Some(("link", sub_m)) => sub_link(&manager, sub_m).await?,
        Some(("remove", sub_m)) => sub_remove(&manager, sub_m).await?,
        Some(("search", sub_m)) => sub_search(&manager, sub_m).await?,
        Some(("clear-cache", _)) => sub_clear_cache(&manager)?,
        Some(("refresh", _)) => sub_refresh(&manager).await?,
        Some(("prune", sub_m)) => sub_prune(&manager, sub_m)?,
        Some(("use", sub_m)) => sub_use(&manager, sub_m).await?,
        Some(("upgrade", sub_m)) => sub_upgrade(&manager, sub_m).await?,
        Some(("pin", sub_m)) => sub_pin(&manager, sub_m).await?,
        Some(("config", sub_m)) => sub_config(sub_m)?,
        Some(("registry", sub_m)) => sub_registry(&manager, sub_m).await?,
        _ => {}
    }

    Ok(())
}

/// Handle the 'install' subcommand
async fn sub_install(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let force_reinstall = matches.get_flag("force");
    let redownload = matches.get_flag("redownload");
    let refresh = matches.get_flag("refresh");
    let include_pre = matches.get_flag("include-pre");

    refresh_cache_if_requested(manager, refresh).await?;

    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    ensure_registry_trusted(manager, registry, matches.get_flag("yes")).await?;

    let requested_version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let gv = manager
        .resolve_available_version(&requested_version, variant, registry, include_pre, false)
        .await?
        .ok_or_else(|| anyhow!(t!("error-version-not-found")))?;

    let resolved_variant = Variant::from_option(variant);
    let display = version_utils::display_version(&gv.to_display_str(), &resolved_variant, registry);

    // Print a message indicating the start of the installation process
    println_i18n!("installing-version", version = &display);

    match manager
        .install(
            &gv,
            &resolved_variant,
            registry,
            force_reinstall,
            redownload,
        )
        .await?
    {
        InstallOutcome::Installed => {
            // Print a message indicating the successful installation
            println_i18n!("installed-success", version = &display);
        }
        InstallOutcome::AlreadyInstalled => {
            // Print a message indicating the version is already installed
            println_i18n!("version-already-installed", version = &display);
            return Ok(());
        }
    }

    Ok(())
}

/// Handle the 'list' subcommand
fn sub_list(manager: &GodotManager) -> Result<()> {
    let versions = manager.list_installed()?;
    if versions.is_empty() {
        println_i18n!("no-versions-installed");
    } else {
        println_i18n!("installed-versions");
        for v in &versions {
            println!(
                "- {}",
                version_utils::display_version(
                    &v.version.to_display_str(),
                    &v.variant,
                    v.registry.as_deref(),
                )
            );
        }
    }
    Ok(())
}

/// Handle the 'run' subcommand
async fn sub_run(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
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

    let variant = check_deprecated_csharp_flag(matches, spec_variant);
    let include_pre = matches.get_flag("include-pre");

    sub_run_inner(RunConfig {
        manager,
        version_input,
        variant,
        console,
        raw_args: &raw_args,
        force_on_mismatch,
        include_pre,
        assume_yes: matches.get_flag("yes"),
    })
    .await
}

/// Handle the 'show' subcommand
async fn sub_show(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let raw_args: Vec<String> = Vec::new();

    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let possible_paths = collect_possible_paths(&raw_args);

    let spec = version_input.map(|v| VersionSpec::parse(v)).transpose()?;
    let spec_variant = spec.as_ref().and_then(|s| s.variant.clone());
    let variant = check_deprecated_csharp_flag(matches, spec_variant);
    let registry = spec.as_ref().and_then(|s| s.registry.clone());

    let explicit_version = match spec.as_ref().map(|s| &s.target) {
        Some(VersionTarget::Pattern(gv)) => Some(gv.clone()),
        Some(VersionTarget::Keyword(kw)) => Some(keyword_to_version_filter(kw)),
        None => None,
    };

    let include_pre = matches.get_flag("include-pre");

    let resolver = RunVersionResolver::new(manager);
    let resolved = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            variant,
            registry,
            include_pre,
            possible_paths: &possible_paths,
            force_on_mismatch,
            install_if_missing: false,
        })
        .await?;

    let exe_path = manager.get_executable_path(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
        console,
    )?;
    println!("{}", exe_path.display());

    Ok(())
}

/// Print the path to the cached download archive for a resolved version.
async fn sub_cache_path(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let refresh = matches.get_flag("refresh");
    let include_pre = matches.get_flag("include-pre");

    refresh_cache_if_requested(manager, refresh).await?;

    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    ensure_registry_trusted(manager, registry, matches.get_flag("yes")).await?;

    let requested_version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let gv = manager
        .resolve_available_version(&requested_version, variant, registry, include_pre, false)
        .await?
        .ok_or_else(|| anyhow!(t!("error-version-not-found")))?;

    let resolved_variant = Variant::from_option(variant);
    let path = manager
        .cached_archive_path(&gv, &resolved_variant, registry)
        .await?;
    println!("{}", path.display());

    Ok(())
}

/// Handle the 'link' subcommand
async fn sub_link(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
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

/// Configuration for the `sub_run_inner` function
struct RunConfig<'a> {
    manager: &'a GodotManager,
    version_input: Option<&'a String>,
    variant: Option<String>,
    console: bool,
    raw_args: &'a Vec<String>,
    force_on_mismatch: bool,
    include_pre: bool,
    assume_yes: bool,
}

/// Run the Godot executable
async fn sub_run_inner(config: RunConfig<'_>) -> Result<()> {
    let RunConfig {
        manager,
        version_input,
        variant,
        console,
        raw_args,
        force_on_mismatch,
        include_pre,
        assume_yes,
    } = config;

    // Try to see if a path was given in raw_args. First, by checking if the --path flag was given
    // and then by checking if the first argument is a path. Prefer the --path flag if both are
    // given.
    let possible_paths = collect_possible_paths(raw_args);

    let (explicit_version, resolved_variant, resolved_registry) = if let Some(v) = version_input {
        let spec = VersionSpec::parse(v)?;
        let var = spec.variant.or(variant);
        let reg = spec.registry;
        let ver = match spec.target {
            VersionTarget::Pattern(gv) => Some(gv),
            VersionTarget::Keyword(ref kw) => Some(keyword_to_version_filter(kw)),
        };
        (ver, var, reg)
    } else {
        (None, variant, None)
    };

    let resolver = RunVersionResolver::new(manager);
    let request = RunResolutionRequest {
        explicit: explicit_version,
        variant: resolved_variant,
        registry: resolved_registry,
        include_pre,
        possible_paths: &possible_paths,
        force_on_mismatch,
        install_if_missing: true,
    };

    let trust_registry = resolver.select(&request).await?.and_then(|s| s.registry);
    ensure_registry_trusted(manager, trust_registry.as_deref(), assume_yes).await?;

    let resolved = resolver.resolve(request).await?;

    let display = version_utils::display_version(
        &resolved.version.to_display_str(),
        &resolved.variant,
        resolved.registry.as_deref(),
    );

    eprintln_i18n!("running-version", version = &display);

    manager.run(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
        console,
        raw_args,
    )?;

    Ok(())
}

/// Handle the 'remove' subcommand
async fn sub_remove(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    let requested_version = match &spec.target {
        VersionTarget::Keyword(_) => {
            return Err(anyhow!(t!("error-version-not-found")));
        }
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let resolved_versions = manager
        .resolve_installed_version(&requested_version, variant, registry)
        .await?;

    match resolved_versions.len() {
        0 => {
            eprintln_i18n!("error-version-not-found");
        }
        1 => {
            let installed = &resolved_versions[0];
            let display = version_utils::display_version(
                &installed.version.to_display_str(),
                &installed.variant,
                installed.registry.as_deref(),
            );

            println_i18n!("removing-version", version = &display);

            if !matches.get_flag("yes") {
                println_i18n!("confirm-remove");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != t!("confirm-yes") {
                    println_i18n!("remove-cancelled");
                    return Ok(());
                }
            }
            manager.remove(
                &installed.version,
                &installed.variant,
                installed.registry.as_deref(),
            )?;
            println_i18n!("removed-version", version = &display);
        }
        _ => {
            eprintln_i18n!("error-multiple-versions-found");
            for installed in &resolved_versions {
                println!(
                    "- {}",
                    version_utils::display_version(
                        &installed.version.to_display_str(),
                        &installed.variant,
                        installed.registry.as_deref(),
                    )
                );
            }
        }
    }

    Ok(())
}

/// Handle the 'search' subcommand
async fn sub_search(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let filter = matches.get_one::<String>("filter").map(|s| s.as_str());
    let include_pre = matches.get_flag("include-pre");
    let cache_only = matches.get_flag("cache-only");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let (registry, version_filter) = match filter {
        Some(f) => match f.split_once('/') {
            Some((reg, rest)) if !reg.is_empty() => (Some(reg), Some(rest)),
            _ => (None, Some(f)),
        },
        None => (None, None),
    };

    ensure_registry_trusted(manager, registry, matches.get_flag("yes")).await?;

    let requested_version = match version_filter {
        Some(filter) => Some(GodotVersion::from_match_str(filter)?),
        None => None,
    };

    let mut releases = manager
        .fetch_available_releases(registry, &requested_version, cache_only)
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
        println_i18n!("no-matching-releases");
    } else {
        println_i18n!("available-releases");
        for r in releases {
            println!("- {r}");
        }
    }
    Ok(())
}

/// Handle the 'clear-cache' subcommand
fn sub_clear_cache(manager: &GodotManager) -> Result<()> {
    manager.clear_cache()?;
    println_i18n!("cache-cleared");
    Ok(())
}

/// Handle the 'refresh' subcommand
async fn sub_refresh(manager: &GodotManager) -> Result<()> {
    manager.refresh_cache().await?;
    println_i18n!("cache-refreshed");
    Ok(())
}

/// Format a byte count into a short user-friendly string.
fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Handle the 'prune' subcommand
fn sub_prune(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let opts = PruneOptions {
        all: matches.get_flag("all"),
        force: matches.get_flag("force"),
        dry_run: matches.get_flag("dry-run"),
    };

    let config = config::Config::load().unwrap_or_default();
    let max_age_secs = config.prune_max_age_days().saturating_mul(24 * 60 * 60);

    let report = manager.prune(max_age_secs, opts)?;

    if report.is_empty() {
        if report.dry_run {
            println_i18n!("prune-nothing-dry-run");
        } else {
            println_i18n!("prune-nothing-removed");
        }
        if report.preserved_by_link > 0 {
            println_i18n!("prune-preserved-by-link", count = report.preserved_by_link);
        }
        return Ok(());
    }

    if report.dry_run {
        println_i18n!("prune-dry-run-header");
    } else {
        println_i18n!("prune-removed-header");
    }

    if !report.installs.is_empty() {
        println_i18n!("prune-installs-header");
        for item in &report.installs {
            println!("- {} ({})", item.label, format_bytes(item.freed_bytes));
        }
    }

    if !report.archives.is_empty() {
        println_i18n!("prune-archives-header");
        for item in &report.archives {
            println!("- {} ({})", item.label, format_bytes(item.freed_bytes));
        }
    }

    if report.preserved_by_link > 0 {
        println_i18n!("prune-preserved-by-link", count = report.preserved_by_link);
    }

    let size = format_bytes(report.freed_bytes);
    if report.dry_run {
        println_i18n!("prune-would-free", size = size);
    } else {
        println_i18n!("prune-freed", size = size);
    }

    Ok(())
}

/// Handle the 'use' subcommand
async fn sub_use(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let refresh = matches.get_flag("refresh");

    let version_input = match matches.get_one::<String>("version") {
        Some(v) => v,
        None => {
            println_i18n!("provide-version-or-unset");
            return Ok(());
        }
    };

    if version_input == "unset" {
        manager.unset_default()?;
        println_i18n!("default-unset-success");
        return Ok(());
    }

    refresh_cache_if_requested(manager, refresh).await?;

    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    ensure_registry_trusted(manager, registry, matches.get_flag("yes")).await?;

    let requested_version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let include_pre = matches.get_flag("include-pre");

    let resolved_version = manager
        .auto_install_version(&requested_version, variant, registry, include_pre)
        .await?;

    let resolved_variant = Variant::from_option(variant);
    manager.set_default(&resolved_version, &resolved_variant, registry)?;
    let display = version_utils::display_version(
        &resolved_version.to_display_str(),
        &resolved_variant,
        registry,
    );
    println_i18n!("default-set-success", version = &display);

    Ok(())
}

/// Handle the 'upgrade' subcommand
async fn sub_upgrade(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let allow_major = matches.get_flag("major");
    let allow_pre = matches.get_flag("pre");
    manager.upgrade(allow_major, allow_pre).await
}

/// Handle the 'pin' subcommand
async fn sub_pin(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_str = matches.get_one::<String>("version").unwrap();
    let refresh = matches.get_flag("refresh");
    let spec = VersionSpec::parse(version_str)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    refresh_cache_if_requested(manager, refresh).await?;

    ensure_registry_trusted(manager, registry, matches.get_flag("yes")).await?;

    let version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    warn_project_version_mismatch::<_, &Path>(manager, &version, true, None).await;

    let include_pre = matches.get_flag("include-pre");

    let resolved_version = manager
        .auto_install_version(&version, variant, registry, include_pre)
        .await?;

    let resolved_variant = Variant::from_option(variant);
    let display = version_utils::display_version(
        &resolved_version.to_display_str(),
        &resolved_variant,
        registry,
    );

    let skip_gdvmrc = matches.get_flag("no-legacy");

    match manager.pin_version(&resolved_version, &resolved_variant, registry, skip_gdvmrc) {
        Ok(()) => println_i18n!("pinned-success", version = &display),
        Err(_) => eprintln_i18n!("error-pin-version-not-found", version = &display),
    }
    Ok(())
}

/// Ensure the user has acknowledged the use of a third-party registry. If the
/// registry is official, this function does nothing. If the registry is not
/// trusted, it prompts the user to confirm that they trust it, unless
/// `assume_yes` is true, in which case it automatically trusts the registry
/// after a brief pause.
async fn ensure_registry_trusted(
    manager: &GodotManager,
    registry: Option<&str>,
    assume_yes: bool,
) -> Result<()> {
    if manager.is_official_registry(registry) {
        return Ok(());
    }

    let name = registry.expect("a non-official registry always has a name");
    let url = manager.registry_base_url(name)?;

    // Always warn when using a third-party registry, even after it has been confirmed.
    eprintln_i18n!(
        "registry-trust-warning",
        registry = name,
        url = url.as_str()
    );

    let mut config = config::Config::load()?;
    if config.is_registry_trusted(&url) {
        return Ok(());
    }

    if assume_yes {
        // Warn and allow the user some time to cancel.
        eprintln_i18n!("registry-trust-bypass", registry = name, url = url.as_str());
        std::thread::sleep(std::time::Duration::from_secs(5));
        config.trust_registry(&url);
        config.save()?;
        return Ok(());
    }

    eprint!("{} ", t!("registry-trust-prompt"));
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != t!("confirm-yes") {
        return Err(anyhow!(t!("registry-trust-aborted")));
    }

    config.trust_registry(&url);
    config.save()?;
    Ok(())
}

/// Download `url` to a unique temporary file and return its path.
async fn download_to_temp(url: &str) -> Result<PathBuf> {
    let tmp = tempfile::Builder::new()
        .prefix("gdvm-add-build-")
        .suffix(".tmp")
        .tempfile()?;
    let mut file = tokio::fs::File::from_std(tmp.as_file().try_clone()?);
    gdvm::download_utils::download_to_file(url, &mut file).await?;
    drop(file);
    let (_file, path) = tmp.keep()?;
    Ok(path)
}

/// Error if an explicit override disagrees with the values measured from the
/// downloaded or local artifact.
fn verify_overrides(
    sha512: &Option<String>,
    size: Option<u64>,
    actual_sha: &str,
    actual_size: u64,
) -> Result<()> {
    if let Some(s) = sha512
        && !s.eq_ignore_ascii_case(actual_sha)
    {
        return Err(anyhow!(
            "{}",
            t!(
                "registry-build-sha-mismatch",
                expected = s.clone(),
                actual = actual_sha.to_string()
            )
        ));
    }
    if let Some(s) = size
        && s != actual_size
    {
        return Err(anyhow!(
            "{}",
            t!(
                "registry-build-size-mismatch",
                expected = s.to_string(),
                actual = actual_size.to_string()
            )
        ));
    }
    Ok(())
}

/// Resolve the SHA-512 and size to record for an `add-build`.
async fn resolve_build_integrity(
    store: bool,
    file: Option<&Path>,
    url: Option<&str>,
    sha512: Option<String>,
    size: Option<u64>,
) -> Result<(Option<String>, Option<u64>)> {
    if store {
        if sha512.is_some() || size.is_some() {
            eprintln_i18n!("registry-build-warn-explicit-store");
        }
        return Ok((sha512, size));
    }

    let Some(url) = url else {
        return Ok((sha512, size));
    };

    if let Some(file) = file {
        eprintln_i18n!("registry-build-warn-local-hash", url = url);
        let (computed_sha, computed_size) = registry::publish::hash_file(file)?;
        verify_overrides(&sha512, size, &computed_sha, computed_size)?;
        return Ok((
            Some(sha512.unwrap_or(computed_sha)),
            Some(size.unwrap_or(computed_size)),
        ));
    }

    if let (Some(sha512), Some(size)) = (sha512.clone(), size) {
        eprintln_i18n!("registry-build-warn-unverified");
        return Ok((Some(sha512), Some(size)));
    }

    eprintln_i18n!("registry-build-downloading", url = url);
    let tmp = download_to_temp(url).await?;
    let resolved = (|| {
        let (computed_sha, computed_size) = registry::publish::hash_file(&tmp)?;
        verify_overrides(&sha512, size, &computed_sha, computed_size)?;
        Ok((Some(computed_sha), Some(computed_size)))
    })();
    let _ = fs::remove_file(&tmp);
    resolved
}

/// Handle the 'registry' subcommand
async fn sub_registry(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("add", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            let url = sub_m.get_one::<String>("url").unwrap();
            let mut config = config::Config::load()?;
            match config.add_registry(name, url) {
                Ok(()) => {
                    config.save()?;
                    println_i18n!(
                        "registry-added",
                        registry = name.as_str(),
                        url = url.as_str()
                    );
                }
                Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
            }
        }
        Some(("remove", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            let mut config = config::Config::load()?;
            match config.remove_registry(name) {
                Ok(()) => {
                    config.save()?;
                    println_i18n!("registry-removed", registry = name.as_str());
                }
                Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
            }
        }
        Some(("list", _)) => {
            let registries = manager.registry_list();
            println_i18n!("registry-list-header");
            for info in registries {
                let mut tags = Vec::new();
                if info.is_official {
                    tags.push(t!("registry-tag-official"));
                }
                let suffix = if tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", tags.join(", "))
                };
                println!("- {} ({}){suffix}", info.name, info.url);
            }
        }
        Some(("refresh", sub_m)) => {
            match sub_m.get_one::<String>("name") {
                Some(name) => manager.refresh_registry_cache(Some(name)).await?,
                None => manager.refresh_all_registry_caches().await?,
            }
            println_i18n!("cache-refreshed");
        }
        Some(("init", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let name = sub_m.get_one::<String>("name").map(|s| s.as_str());
            match registry::publish::init(&dir, name) {
                Ok(name) => println_i18n!(
                    "registry-init-success",
                    name = name,
                    path = dir.display().to_string()
                ),
                Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
            }
        }
        Some(("add-build", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let file = sub_m.get_one::<String>("file").map(PathBuf::from);
            let store = sub_m.get_flag("store");
            let url = sub_m.get_one::<String>("url").cloned();
            let sha512 = sub_m.get_one::<String>("sha512").cloned();
            let size = sub_m.get_one::<u64>("size").copied();

            match resolve_build_integrity(store, file.as_deref(), url.as_deref(), sha512, size)
                .await
            {
                Ok((sha512, size)) => {
                    let args = registry::publish::AddBuild {
                        version: sub_m.get_one::<String>("version").unwrap().clone(),
                        variant: sub_m.get_one::<String>("variant").cloned(),
                        platform: sub_m.get_one::<String>("platform").unwrap().clone(),
                        file,
                        store,
                        url,
                        sha512,
                        size,
                    };
                    let version = args.version.clone();
                    let platform = args.platform.clone();
                    match registry::publish::add_build(&dir, &args) {
                        Ok(()) => println_i18n!(
                            "registry-build-added",
                            version = version,
                            platform = platform
                        ),
                        Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
                    }
                }
                Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
            }
        }
        Some(("remove-build", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let args = registry::publish::RemoveBuild {
                version: sub_m.get_one::<String>("version").unwrap().clone(),
                variant: sub_m.get_one::<String>("variant").cloned(),
                platform: sub_m.get_one::<String>("platform").cloned(),
            };
            let version = args.version.clone();
            match registry::publish::remove_build(&dir, &args) {
                Ok(()) => println_i18n!("registry-build-removed", version = version),
                Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
            }
        }
        Some(("validate", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            match registry::publish::validate(&dir) {
                Ok(report) if report.is_valid() => {
                    println_i18n!("registry-validate-ok", count = report.checked);
                }
                Ok(report) => {
                    eprintln_i18n!("registry-validate-failed");
                    for error in &report.errors {
                        eprintln!("  - {error}");
                    }
                    std::process::exit(1);
                }
                Err(e) => eprintln_i18n!("registry-error", error = e.to_string()),
            }
        }
        _ => eprintln_i18n!(
            "registry-error",
            error = t!("error-invalid-registry-subcommand")
        ),
    }
    Ok(())
}

/// Handle the 'config' subcommand
fn sub_config(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let mut config = config::Config::load()?;
    match matches.subcommand() {
        Some(("get", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            if let Some(value) = config.get_value(key) {
                println!("{value}");
            } else {
                println!("{}", t!("config-key-not-set"));
            }
        }
        Some(("set", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            // If the value argument is not provided, prompt the user.
            let value: String = if let Some(v) = sub_m.get_one::<String>("value") {
                v.clone()
            } else {
                // Build the prompt message from the Fluent bundle.
                let prompt = t!("config-set-prompt", key = key.as_str());
                eprint!("{prompt} ");
                if config.is_sensitive_key(key) {
                    // Mask input for sensitive values.
                    match rpassword::prompt_password("") {
                        Ok(input) => input,
                        Err(err) => {
                            eprintln!("{}: {}", t!("error-reading-input"), err);
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
                eprintln_i18n!("warning-setting-sensitive");
            }
            if !config::KNOWN_KEYS.contains(&key.as_str()) {
                eprintln_i18n!("error-unknown-config-key");
            } else {
                match config.set_value(key, &value) {
                    Ok(()) => {
                        config.save()?;
                        println_i18n!("config-set-success");
                    }
                    Err(_) => eprintln_i18n!("error-invalid-config-value", key = key),
                }
            }
        }
        Some(("unset", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            match config.unset_value(key) {
                Ok(()) => {
                    config.save()?;
                    println_i18n!("config-unset-success", key = key);
                }
                Err(_) => eprintln_i18n!("error-unknown-config-key"),
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
        _ => eprintln!("{}", t!("error-invalid-config-subcommand")),
    }
    Ok(())
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
