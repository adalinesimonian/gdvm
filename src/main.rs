mod godot_manager;
mod i18n;
mod version_utils;
mod zip_utils;

use anyhow::{anyhow, Result};
use clap::{value_parser, Arg, ArgMatches, Command};
use fluent_bundle::FluentValue;
use godot_manager::{GodotManager, InstallOutcome};
use i18n::I18n;
use std::io::{self, Write};

use version_utils::GodotVersion;

fn main() -> Result<()> {
    let i18n = I18n::new()?;
    let manager = GodotManager::new(&i18n)?;

    let matches = Command::new("gdvm")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Adaline Simonian <adalinesimonian@gmail.com>")
        .about(i18n.t("help-about"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(clap::ArgAction::Help)
                .global(true)
                .help(i18n.t("help-help")),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(i18n.t("help-gdvm-version")),
        )
        .subcommand(
            Command::new("install")
                .about(i18n.t("help-install"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_remote_version)
                        .help(i18n.t("help-version"))
                        .long_help(i18n.t("help-version-long")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(i18n.t("help-csharp")),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(i18n.t("help-force")),
                )
                .arg(
                    Arg::new("redownload")
                        .long("redownload")
                        .num_args(0)
                        .help(i18n.t("help-redownload")),
                ),
        )
        .subcommand(Command::new("list").about(i18n.t("help-list")))
        .subcommand(
            Command::new("run")
                .about(i18n.t("help-run"))
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_remote_version)
                        .help(i18n.t("help-version-installed")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .value_parser(value_parser!(bool))
                        .num_args(0..=1)
                        .default_missing_value("true")
                        .default_value("false")
                        .require_equals(true)
                        .help(i18n.t("help-csharp"))
                        .long_help(i18n.t("help-run-csharp-long")),
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
                        .help(i18n.t("help-console")),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about(i18n.t("help-remove"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_remote_version)
                        .help(i18n.t("help-version-installed")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(i18n.t("help-csharp")),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .num_args(0)
                        .help(i18n.t("help-yes")),
                ),
        )
        .subcommand(
            Command::new("search")
                .about(i18n.t("help-search"))
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .num_args(1)
                        .help(i18n.t("help-filter")),
                )
                .arg(
                    Arg::new("include-pre")
                        .long("include-pre")
                        .num_args(0)
                        .help(i18n.t("help-include-pre")),
                )
                .arg(
                    Arg::new("cache-only")
                        .long("cache-only")
                        .num_args(0)
                        .help(i18n.t("help-cache-only")),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .num_args(1)
                        .default_value("10")
                        .value_parser(clap::value_parser!(usize))
                        .help(i18n.t("help-limit")),
                ),
        )
        .subcommand(Command::new("clear-cache").about(i18n.t("help-clear-cache")))
        .subcommand(
            Command::new("use")
                .about(i18n.t("help-default"))
                .arg(
                    Arg::new("version")
                        .help(i18n.t("help-default-version"))
                        .required(false),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(i18n.t("help-csharp")),
                ),
        )
        .get_matches();

    // Match the subcommand and call the appropriate function
    match matches.subcommand() {
        Some(("install", sub_m)) => sub_install(&i18n, &manager, sub_m)?,
        Some(("list", _)) => sub_list(&i18n, &manager)?,
        Some(("run", sub_m)) => sub_run(&i18n, &manager, sub_m)?,
        Some(("remove", sub_m)) => sub_remove(&i18n, &manager, sub_m)?,
        Some(("search", sub_m)) => sub_search(&i18n, &manager, sub_m)?,
        Some(("clear-cache", _)) => sub_clear_cache(&i18n, &manager)?,
        Some(("use", sub_m)) => sub_use(&i18n, &manager, sub_m)?,
        _ => {}
    }

    Ok(())
}

/// Handle the 'install' subcommand
fn sub_install(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let force_reinstall = matches.get_flag("force");
    let redownload = matches.get_flag("redownload");

    let requested_version = GodotVersion::from_match_str(&version_input)?;
    let mut gv = manager
        .resolve_available_version(&requested_version, false)
        .ok_or_else(|| anyhow!(i18n.t("error-version-not-found")))?;

    let is_csharp = matches.get_flag("csharp");
    gv.is_csharp = Some(is_csharp);

    // Print a message indicating the start of the installation process
    println_i18n!(
        i18n,
        "installing-version",
        [("version", &gv.to_display_str())]
    );

    match manager.install(&gv, force_reinstall, redownload)? {
        InstallOutcome::Installed => {
            // Print a message indicating the successful installation
            println_i18n!(
                i18n,
                "installed-success",
                [("version", &gv.to_display_str())]
            );
        }
        InstallOutcome::AlreadyInstalled => {
            // Print a message indicating the version is already installed
            println_i18n!(
                i18n,
                "version-already-installed",
                [("version", &gv.to_display_str())]
            );
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
        for v in versions {
            println!("- {}", v);
        }
    }
    Ok(())
}

/// Handle the 'run' subcommand
fn sub_run(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    // specifically check if --csharp was provided as a flag or if we're reading the default value
    let csharp_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    let csharp_flag = matches.get_flag("csharp");
    let version_input = matches.get_one::<String>("version");

    let resolved_version = if let Some(v) = version_input {
        let mut requested_version = GodotVersion::from_match_str(&v)?;

        requested_version.is_csharp = Some(csharp_flag);

        manager.auto_install_version(&requested_version)?
    } else if let Some(mut default_ver) = manager.get_default()? {
        if csharp_given {
            default_ver.is_csharp = Some(csharp_flag);
        }

        default_ver
    } else {
        return Err(anyhow!(i18n.t("no-default-set")));
    };

    let console = matches.get_flag("console");
    println_i18n!(
        i18n,
        "running-version",
        [("version", &resolved_version.to_display_str())]
    );
    manager.run(&resolved_version, console)?;

    Ok(())
}

/// Handle the 'remove' subcommand
fn sub_remove(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let csharp = matches.get_flag("csharp");
    let mut requested_version = GodotVersion::from_match_str(&version_input)?;

    requested_version.is_csharp = Some(csharp);

    let resolved_versions = manager.resolve_installed_version(&requested_version)?;

    match resolved_versions.len() {
        0 => {
            println_i18n!(i18n, "error-version-not-found");
        }
        1 => {
            let gv = &resolved_versions[0];

            println_i18n!(i18n, "removing-version", [("version", gv.to_display_str())]);

            if !matches.get_flag("yes") {
                println_i18n!(i18n, "confirm-remove");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != i18n.t("confirm-yes") {
                    println_i18n!(i18n, "remove-cancelled");
                    return Ok(());
                }
            }
            manager.remove(&gv)?;
            println_i18n!(i18n, "removed-version", [("version", gv.to_display_str())]);
        }
        _ => {
            println_i18n!(i18n, "error-multiple-versions-found");
            for v in resolved_versions {
                println!("- {}", v.to_display_str());
            }
        }
    }

    Ok(())
}

/// Handle the 'search' subcommand
fn sub_search(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let filter = matches.get_one::<String>("filter").map(|s| s.as_str());
    let include_pre = matches.get_flag("include-pre");
    let cache_only = matches.get_flag("cache-only");

    let requested_version = match filter {
        Some(filter) => Some(GodotVersion::from_match_str(filter)?),
        None => None,
    };

    let mut releases = manager.fetch_available_releases(&requested_version, cache_only)?;

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
            println!("- {}", r);
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

/// Handle the 'use' subcommand
fn sub_use(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let csharp = matches.get_flag("csharp");

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

    let mut requested_version = GodotVersion::from_match_str(&version_input)?;

    requested_version.is_csharp = Some(csharp);

    let resolved_version = manager.auto_install_version(&requested_version)?;

    manager.set_default(&resolved_version)?;
    println_i18n!(
        i18n,
        "default-set-success",
        [("version", &resolved_version.to_display_str())]
    );

    Ok(())
}
