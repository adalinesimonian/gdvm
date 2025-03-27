mod config;
mod download_utils;
mod godot_manager;
mod i18n;
mod project_version_detector;
mod version_utils;
mod zip_utils;

use anyhow::{Result, anyhow};
use clap::{Arg, ArgMatches, Command, value_parser};
use config::ConfigOps;
use godot_manager::{GodotManager, InstallOutcome};
use i18n::I18n;
use std::{
    io::{self, Write},
    path::Path,
};

use version_utils::GodotVersion;

fn main() -> Result<()> {
    let i18n = I18n::new(100)?;
    let manager = GodotManager::new(&i18n)?;

    // Detect if running through "godot" or "godot_console" symlink
    let exe_name = std::env::current_exe()?
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

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
            manager: &GodotManager::new(&i18n)?,
            version_input: None,
            csharp_given: false,
            csharp_flag: false,
            console: console_mode,
            raw_args: &args,
            force_on_mismatch: false,
        }) {
            eprintln!("{}", err);

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
                        .value_parser(version_utils::validate_remote_version)
                        .help(t!(i18n, "help-version"))
                        .long_help(t!(i18n, "help-version-long")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(t!(i18n, "help-csharp")),
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
                ),
        )
        .subcommand(Command::new("list").about(t!(i18n, "help-list")))
        .subcommand(
            Command::new("run")
                .about(t!(i18n, "help-run"))
                .arg(
                    Arg::new("version")
                        .required(false)
                        .value_parser(version_utils::validate_remote_version)
                        .help(t!(i18n, "help-version-installed")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .value_parser(value_parser!(bool))
                        .num_args(0..=1)
                        .default_missing_value("true")
                        .default_value("false")
                        .require_equals(true)
                        .help(t!(i18n, "help-csharp"))
                        .long_help(t!(i18n, "help-run-csharp-long")),
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
                ),
        )
        .subcommand(
            Command::new("remove")
                .about(t!(i18n, "help-remove"))
                .arg(
                    Arg::new("version")
                        .required(true)
                        .value_parser(version_utils::validate_remote_version)
                        .help(t!(i18n, "help-version-installed")),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(t!(i18n, "help-csharp")),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .num_args(0)
                        .help(t!(i18n, "help-yes")),
                ),
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
                ),
        )
        .subcommand(Command::new("clear-cache").about(t!(i18n, "help-clear-cache")))
        .subcommand(
            Command::new("use")
                .about(t!(i18n, "help-default"))
                .arg(
                    Arg::new("version")
                        .help(t!(i18n, "help-default-version"))
                        .required(false),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(t!(i18n, "help-csharp")),
                ),
        )
        .subcommand(Command::new("upgrade").about(t!(i18n, "help-upgrade")))
        .subcommand(
            Command::new("pin")
                .about(t!(i18n, "help-pin"))
                .long_about(t!(i18n, "help-pin-long"))
                .arg(
                    Arg::new("version")
                        .help(t!(i18n, "help-pin-version"))
                        .required(true),
                )
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(t!(i18n, "help-csharp")),
                ),
        )
        .subcommand(
            Command::new("config")
                .about(t!(i18n, "help-config"))
                .subcommand(
                    Command::new("get")
                        .about(t!(i18n, "help-config-get"))
                        .arg(
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
        Some(("install", sub_m)) => sub_install(&i18n, &manager, sub_m)?,
        Some(("list", _)) => sub_list(&i18n, &manager)?,
        Some(("run", sub_m)) => sub_run(&i18n, &manager, sub_m)?,
        Some(("remove", sub_m)) => sub_remove(&i18n, &manager, sub_m)?,
        Some(("search", sub_m)) => sub_search(&i18n, &manager, sub_m)?,
        Some(("clear-cache", _)) => sub_clear_cache(&i18n, &manager)?,
        Some(("use", sub_m)) => sub_use(&i18n, &manager, sub_m)?,
        Some(("upgrade", _)) => sub_upgrade(&manager)?,
        Some(("pin", sub_m)) => sub_pin(&i18n, &manager, sub_m)?,
        Some(("config", sub_m)) => sub_config(&i18n, sub_m)?,
        _ => {}
    }

    Ok(())
}

/// Handle the 'install' subcommand
fn sub_install(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let force_reinstall = matches.get_flag("force");
    let redownload = matches.get_flag("redownload");

    let requested_version = GodotVersion::from_match_str(version_input)?;
    let mut gv = manager
        .resolve_available_version(&requested_version, false)
        .ok_or_else(|| anyhow!(t!(i18n, "error-version-not-found")))?;

    let is_csharp = matches.get_flag("csharp");
    gv.is_csharp = Some(is_csharp);

    // Print a message indicating the start of the installation process
    println_i18n!(
        i18n,
        "installing-version",
        //[("version", &gv.to_display_str())]
        version = &gv.to_display_str()
    );

    match manager.install(&gv, force_reinstall, redownload)? {
        InstallOutcome::Installed => {
            // Print a message indicating the successful installation
            println_i18n!(i18n, "installed-success", version = &gv.to_display_str());
        }
        InstallOutcome::AlreadyInstalled => {
            // Print a message indicating the version is already installed
            println_i18n!(
                i18n,
                "version-already-installed",
                version = &gv.to_display_str()
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
    // Capture args after "--" to pass directly to child
    let raw_args = match std::env::args().position(|x| x == "--") {
        Some(pos) => std::env::args().skip(pos + 1).collect(),
        None => Vec::new(),
    };

    // specifically check if --csharp was provided as a flag or if we're reading the default value
    let csharp_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    let csharp_flag = matches.get_flag("csharp");
    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");

    sub_run_inner(RunConfig {
        i18n,
        manager,
        version_input,
        csharp_given,
        csharp_flag,
        console,
        raw_args: &raw_args,
        force_on_mismatch,
    })
}

/// Configuration for the `sub_run_inner` function
struct RunConfig<'a> {
    i18n: &'a I18n,
    manager: &'a GodotManager<'a>,
    version_input: Option<&'a String>,
    csharp_given: bool,
    csharp_flag: bool,
    console: bool,
    raw_args: &'a Vec<String>,
    force_on_mismatch: bool,
}

/// Run the Godot executable
fn sub_run_inner(config: RunConfig) -> Result<()> {
    let RunConfig {
        i18n,
        manager,
        version_input,
        csharp_given,
        csharp_flag,
        console,
        raw_args,
        force_on_mismatch,
    } = config;

    // Try to see if a path was given in raw_args. First, by checking if the --path flag was given
    // and then by checking if the first argument is a path. Prefer the --path flag if both are
    // given.
    let mut possible_paths: Vec<&str> = Vec::new();
    for arg in raw_args.iter() {
        if arg == "--path" {
            if let Some(p) = raw_args.get(raw_args.iter().position(|x| x == "--path").unwrap() + 1)
            {
                possible_paths.clear();
                possible_paths.push(p);
                break;
            }
        } else if arg.starts_with('-') {
            continue;
        } else {
            possible_paths.push(arg);
        }
    }

    let resolved_version = if let Some(v) = version_input {
        let mut requested_version = GodotVersion::from_match_str(v)?;

        requested_version.is_csharp = Some(csharp_flag);

        if warn_project_version_mismatch(
            i18n,
            manager,
            &requested_version,
            false,
            Some(&possible_paths),
        ) {
            if force_on_mismatch {
                eprintln_i18n!(
                    i18n,
                    "warning-project-version-mismatch-force",
                    requested_version = requested_version.to_display_str(),
                    pinned = 0,
                );
            } else {
                return Err(anyhow!(t_w!(
                    i18n,
                    "error-project-version-mismatch",
                    pinned = 0,
                )));
            }
        }

        manager.auto_install_version(&requested_version)?
    } else if let Some(pinned) = manager.get_pinned_version() {
        if warn_project_version_mismatch::<&Path>(i18n, manager, &pinned, true, None) {
            if force_on_mismatch {
                eprintln_i18n!(
                    i18n,
                    "warning-project-version-mismatch-force",
                    requested_version = pinned.to_display_str(),
                    pinned = 1,
                );
            } else {
                return Err(anyhow!(t_w!(
                    i18n,
                    "error-project-version-mismatch",
                    pinned = 1
                )));
            }
        }

        manager.auto_install_version(&pinned)?
    } else if let Some(project_version) = possible_paths
        .iter()
        .find_map(|p| manager.determine_version(Some(p)))
    {
        eprintln_i18n!(
            i18n,
            "warning-using-project-version",
            version = project_version.to_display_str()
        );
        manager.auto_install_version(&project_version)?
    } else if let Some(mut default_ver) = manager.get_default()? {
        if csharp_given {
            default_ver.is_csharp = Some(csharp_flag);
        }

        default_ver
    } else {
        return Err(anyhow!(t!(i18n, "no-default-set")));
    };

    eprintln_i18n!(
        i18n,
        "running-version",
        version = &resolved_version.to_display_str()
    );

    manager.run(&resolved_version, console, raw_args)?;

    Ok(())
}

/// Show a warning if the project version is different from the pinned version
fn warn_project_version_mismatch<P: AsRef<Path>>(
    i18n: &I18n,
    manager: &GodotManager,
    requested: &GodotVersion,
    is_pin: bool,
    paths: Option<&[P]>,
) -> bool {
    let determined_version = if let Some(paths) = paths {
        paths
            .iter()
            .find_map(|p| manager.determine_version(Some(p)))
    } else {
        manager.determine_version::<P>(None)
    };

    if let Some(project_version) = determined_version {
        // Check if they don't match (project versions at most specify major.minor or
        // major.minor.patch, and if .patch is not specified, it's assumed to allow any patch)
        if project_version.major.is_some() && requested.major.is_some() && project_version.major != requested.major // Check major if both are Some
            || project_version.minor.is_some() && requested.minor.is_some() && project_version.minor != requested.minor // Check minor if both are Some
            // Allow either both to be None or both to be Some, but if both are Some, they must match
            || (project_version.patch.is_some() && requested.patch.is_some()
                && project_version.patch != requested.patch)
            // If the project version is C#, the pinned version must also be C#, and vice versa
            || project_version.is_csharp.unwrap_or(false) != requested.is_csharp.unwrap_or(false)
        {
            eprintln_i18n!(
                i18n,
                "warning-project-version-mismatch",
                project_version = project_version.to_display_str(),
                requested_version = requested.to_display_str(),
                pinned = is_pin as i32,
            );
            eprintln!();

            return true;
        }
    }

    false
}

/// Handle the 'remove' subcommand
fn sub_remove(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let csharp = matches.get_flag("csharp");
    let mut requested_version = GodotVersion::from_match_str(version_input)?;

    requested_version.is_csharp = Some(csharp);

    let resolved_versions = manager.resolve_installed_version(&requested_version)?;

    match resolved_versions.len() {
        0 => {
            eprintln_i18n!(i18n, "error-version-not-found");
        }
        1 => {
            let gv = &resolved_versions[0];

            println_i18n!(i18n, "removing-version", version = gv.to_display_str());

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
            manager.remove(gv)?;
            println_i18n!(i18n, "removed-version", version = gv.to_display_str());
        }
        _ => {
            eprintln_i18n!(i18n, "error-multiple-versions-found");
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

    let mut requested_version = GodotVersion::from_match_str(version_input)?;

    requested_version.is_csharp = Some(csharp);

    let resolved_version = manager.auto_install_version(&requested_version)?;

    manager.set_default(&resolved_version)?;
    println_i18n!(
        i18n,
        "default-set-success",
        version = &resolved_version.to_display_str(),
    );

    Ok(())
}

/// Handle the 'upgrade' subcommand
fn sub_upgrade(manager: &GodotManager) -> Result<()> {
    manager.upgrade()
}

/// Handle the 'pin' subcommand
fn sub_pin(i18n: &I18n, manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let version_str = matches.get_one::<String>("version").unwrap();
    let csharp = matches.get_flag("csharp");
    let mut version = GodotVersion::from_match_str(version_str)?;

    version.is_csharp = Some(csharp);

    warn_project_version_mismatch::<&Path>(i18n, manager, &version, true, None);

    let resolved_version = manager.auto_install_version(&version)?;

    match manager.pin_version(&resolved_version) {
        Ok(()) => println_i18n!(
            i18n,
            "pinned-success",
            version = &resolved_version.to_display_str(),
        ),
        Err(_) => eprintln_i18n!(
            i18n,
            "error-pin-version-not-found",
            version = &resolved_version.to_display_str(),
        ),
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
                println!("{}", value);
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
                eprint!("{} ", prompt);
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
                    println!("{} = {}", key, display_value);
                }
            } else {
                // List only keys that are set.
                for (key, value, sensitive) in config.list_set_keys() {
                    let display_value = if sensitive && !show_sensitive {
                        "********".to_string()
                    } else {
                        value
                    };
                    println!("{} = {}", key, display_value);
                }
            }
        }
        _ => eprintln!("{}", t!(i18n, "error-invalid-config-subcommand")),
    }
    Ok(())
}
