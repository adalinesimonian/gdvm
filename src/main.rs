mod godot_manager;
mod i18n;
mod version_utils;
mod zip_utils;

use anyhow::{anyhow, Result};
use clap::{value_parser, Arg, ArgMatches, Command};
use fluent_bundle::FluentValue;
use godot_manager::{GodotManager, InstallOutcome};
use i18n::I18n;
use regex::Regex;
use std::io::{self, Write};

use version_utils::{GodotBranch, GodotVersion};

fn validate_godot_version(s: &str) -> Result<String, String> {
    let re = Regex::new(r"^\d+(\.\d+){0,3}(-[A-Za-z0-9]+)?$").unwrap();
    if re.is_match(s) {
        Ok(s.to_string())
    } else {
        Err(String::from("error-invalid-godot-version"))
    }
}

fn validate_remote_version(s: &str) -> Result<String, String> {
    if s == "stable" {
        return Ok(s.to_string());
    }
    validate_godot_version(s).map_err(|_| String::from("error-invalid-remote-version"))
}

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
                        .value_parser(validate_remote_version)
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
                        .value_parser(validate_godot_version)
                        .help(i18n.t("help-version-installed")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(i18n.t("help-csharp")),
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
                        .help(i18n.t("help-console")),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about(i18n.t("help-remove"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(validate_godot_version)
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

    let version_branch = if version_input == "stable" {
        // Fetch the latest stable version
        manager.get_latest_stable_version()?
    } else {
        // Validate that the version exists
        match manager.resolve_available_version(version_input, false) {
            Some(v) => v,
            None => {
                println!("{}", i18n.t("error-version-not-found"));
                return Ok(());
            }
        }
    };

    let is_csharp = matches.get_flag("csharp");

    // Split version and branch if present
    let parts: Vec<&str> = version_branch.split('-').collect();
    let version_str = parts[0];
    let branch_str = parts.get(1).unwrap_or(&"stable");

    // Determine the branch type based on the input string
    let branch = match *branch_str {
        "stable" => GodotBranch::Stable,
        other => GodotBranch::PreRelease(other.to_string()),
    };

    let gv = GodotVersion {
        version: version_str.to_string(),
        branch,
        is_csharp,
    };

    // Print a message indicating the start of the installation process
    println!(
        "{}",
        i18n.t_args(
            "installing-version",
            &[("version", FluentValue::from(gv.to_string()))]
        )
    );

    match manager.install(&gv, force_reinstall, redownload)? {
        InstallOutcome::Installed => {
            // Print a message indicating the successful installation
            println!(
                "{}",
                i18n.t_args(
                    "installed-success",
                    &[("version", FluentValue::from(gv.to_string()))]
                )
            );
        }
        InstallOutcome::AlreadyInstalled => {
            // Print a message indicating the version is already installed
            println!(
                "{}",
                i18n.t_args(
                    "version-already-installed",
                    &[("version", FluentValue::from(gv.to_string()))]
                )
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
        println!("{}", i18n.t("no-versions-installed"));
    } else {
        println!("{}", i18n.t("installed-versions"));
        for v in versions {
            println!("- {}", v);
        }
    }
    Ok(())
}

/// Handle the 'run' subcommand
fn sub_run(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version");
    let version_to_run = if let Some(v) = version_input {
        v.to_string()
    } else if let Some(default_ver) = manager.get_default()? {
        default_ver
    } else {
        return Err(anyhow!(i18n.t("no-default-set")));
    };

    let csharp = matches.get_flag("csharp");
    let console = matches.get_flag("console");
    let resolved_versions = manager.resolve_installed_version(&version_to_run, csharp)?;

    match resolved_versions.len() {
        0 => {
            println!("{}", i18n.t("error-version-not-found"));
        }
        1 => {
            println!(
                "{}",
                i18n.t_args(
                    "running-version",
                    &[(
                        "version",
                        FluentValue::from(version_utils::friendly_installed_version(
                            &resolved_versions[0]
                        ))
                    )]
                )
            );
            manager.run(&resolved_versions[0], console)?;
        }
        _ => {
            println!("{}", i18n.t("error-multiple-versions-found"));
            for v in resolved_versions {
                println!("- {}", version_utils::friendly_installed_version(&v));
            }
        }
    }

    Ok(())
}

/// Handle the 'remove' subcommand
fn sub_remove(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let csharp = matches.get_flag("csharp");
    let resolved_versions = manager.resolve_installed_version(version_input, csharp)?;

    match resolved_versions.len() {
        0 => {
            println!("{}", i18n.t("error-version-not-found"));
        }
        1 => {
            let version_str = &resolved_versions[0];
            // Parse GodotVersion from version_str
            let parts: Vec<&str> = version_str.split('-').collect();
            let version = parts[0].to_string();
            let branch = if parts.len() > 1 && parts[1] != "csharp" {
                GodotBranch::PreRelease(parts[1].to_string())
            } else {
                GodotBranch::Stable
            };
            let is_csharp = version_str.ends_with("-csharp");

            let gv = GodotVersion {
                version,
                branch,
                is_csharp,
            };

            println!(
                "{}",
                i18n.t_args(
                    "removing-version",
                    &[("version", FluentValue::from(gv.to_string()))]
                )
            );

            if !matches.get_flag("yes") {
                print!("{}", i18n.t("confirm-remove"));
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != i18n.t("confirm-yes") {
                    println!("{}", i18n.t("remove-cancelled"));
                    return Ok(());
                }
            }
            manager.remove(&gv)?;
            println!(
                "{}",
                i18n.t_args(
                    "removed-version",
                    &[("version", FluentValue::from(gv.to_string()))]
                )
            );
        }
        _ => {
            println!("{}", i18n.t("error-multiple-versions-found"));
            for v in resolved_versions {
                println!("- {}", version_utils::friendly_installed_version(&v));
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
    let mut releases = manager.fetch_available_releases(filter, cache_only)?;

    // Default to showing only stable releases unless `--include-pre` is specified
    if !include_pre {
        releases.retain(|r| {
            let version = version_utils::normalize_version(r);
            version
                .map(|v| v.pre.is_empty() || v.pre.as_str().starts_with("stable"))
                .unwrap_or(false)
        });
    }

    let limit = matches.get_one::<usize>("limit").unwrap();
    if *limit != 0 {
        releases = releases.into_iter().take(*limit).collect();
    }

    if releases.is_empty() {
        println!("{}", i18n.t("no-matching-releases"));
    } else {
        println!("{}", i18n.t("available-releases"));
        for r in releases {
            println!("- {}", r);
        }
    }
    Ok(())
}

/// Handle the 'clear-cache' subcommand
fn sub_clear_cache(i18n: &I18n, manager: &GodotManager) -> Result<()> {
    manager.clear_cache()?;
    println!("{}", i18n.t("cache-cleared"));
    Ok(())
}

/// Handle the 'use' subcommand
fn sub_use(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let csharp = matches.get_flag("csharp");
    if let Some(ver) = matches.get_one::<String>("version") {
        if ver == "unset" {
            manager.unset_default()?;
            println!("{}", i18n.t("default-unset-success"));
        } else {
            let resolved_versions = manager.resolve_installed_version(ver, csharp)?;

            match resolved_versions.len() {
                0 => {
                    println!("{}", i18n.t("error-version-not-found"));
                }
                1 => {
                    // Set the default version
                    manager.set_default(&resolved_versions[0])?;
                    println!(
                        "{}",
                        i18n.t_args(
                            "default-set-success",
                            &[(
                                "version",
                                FluentValue::from(version_utils::friendly_installed_version(
                                    &resolved_versions[0]
                                ))
                            )]
                        )
                    );
                }
                _ => {
                    println!("{}", i18n.t("error-multiple-versions-found"));
                    for v in resolved_versions {
                        println!("- {}", version_utils::friendly_installed_version(&v));
                    }
                }
            }
        }
    } else {
        println!("{}", i18n.t("provide-version-or-unset"));
    }
    Ok(())
}
