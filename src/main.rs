mod download_utils;
mod godot_manager;
mod i18n;
mod project_version_detector;
mod version_utils;
mod zip_utils;

use anyhow::{anyhow, Result};
use clap::{value_parser, Arg, ArgMatches, Command};
use godot_manager::{GodotManager, InstallOutcome};
use i18n::I18n;
use std::io::{self, Write};

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

        // Search for the first argument that is a valid file and change to its directory
        // This is to make sure that we are using the working directory of the project when checking
        // for the Godot version to run
        if let Some(file) = args.iter().find(|arg| std::path::Path::new(arg).exists()) {
            let file_path = std::path::Path::new(file);

            // Resolve to absolute path
            let abs_path: std::path::PathBuf = if file_path.is_absolute() {
                file_path.to_path_buf()
            } else {
                std::env::current_dir()?.join(file_path)
            };

            // Get the parent directory of the file
            if let Some(file_dir) = abs_path.parent() {
                // Change the current working directory to the file's directory
                std::env::set_current_dir(file_dir)?;
            }
        }

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
        .about(i18n.t("help-about"))
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
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .num_args(0)
                        .help(i18n.t("help-run-force"))
                        .long_help(i18n.t("help-run-force-long")),
                )
                // Allow any number of command line arguments to be passed to the Godot executable after "--"
                .arg(
                    Arg::new("args")
                        .num_args(0..)
                        .last(true)
                        .help(i18n.t("help-run-args")),
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
        .subcommand(Command::new("upgrade").about(i18n.t("help-upgrade")))
        .subcommand(
            Command::new("pin")
                .about(i18n.t("help-pin"))
                .long_about(i18n.t("help-pin-long"))
                .arg(
                    Arg::new("version")
                        .help(i18n.t("help-pin-version"))
                        .required(true),
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
        Some(("upgrade", _)) => sub_upgrade(&manager)?,
        Some(("pin", sub_m)) => sub_pin(&i18n, &manager, sub_m)?,
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

    let resolved_version = if let Some(v) = version_input {
        let mut requested_version = GodotVersion::from_match_str(v)?;

        requested_version.is_csharp = Some(csharp_flag);

        if warn_project_version_mismatch(i18n, manager, &requested_version, false) {
            if force_on_mismatch {
                eprintln_i18n!(
                    i18n,
                    "warning-project-version-mismatch-force",
                    [
                        ("requested_version", requested_version.to_display_str()),
                        ("pinned", 0)
                    ]
                );
            } else {
                return Err(anyhow!(i18n.t_args_w(
                    "error-project-version-mismatch",
                    &[("pinned", fluent_bundle::FluentValue::from(0))]
                )));
            }
        }

        manager.auto_install_version(&requested_version)?
    } else if let Some(pinned) = manager.get_pinned_version() {
        if warn_project_version_mismatch(i18n, manager, &pinned, true) {
            if force_on_mismatch {
                eprintln_i18n!(
                    i18n,
                    "warning-project-version-mismatch-force",
                    [
                        ("requested_version", pinned.to_display_str()),
                        ("pinned", 1)
                    ]
                );
            } else {
                return Err(anyhow!(i18n.t_args_w(
                    "error-project-version-mismatch",
                    &[("pinned", fluent_bundle::FluentValue::from(1))]
                )));
            }
        }

        manager.auto_install_version(&pinned)?
    } else if let Some(project_version) = manager.determine_version() {
        eprintln_i18n!(
            i18n,
            "warning-using-project-version",
            [("version", project_version.to_display_str())]
        );
        manager.auto_install_version(&project_version)?
    } else if let Some(mut default_ver) = manager.get_default()? {
        if csharp_given {
            default_ver.is_csharp = Some(csharp_flag);
        }

        default_ver
    } else {
        return Err(anyhow!(i18n.t("no-default-set")));
    };

    eprintln_i18n!(
        i18n,
        "running-version",
        [("version", &resolved_version.to_display_str())]
    );

    manager.run(&resolved_version, console, raw_args)?;

    Ok(())
}

/// Show a warning if the project version is different from the pinned version
fn warn_project_version_mismatch(
    i18n: &I18n,
    manager: &GodotManager,
    requested: &GodotVersion,
    is_pin: bool,
) -> bool {
    if let Some(project_version) = manager.determine_version() {
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
                [
                    ("project_version", project_version.to_display_str()),
                    ("requested_version", requested.to_display_str()),
                    ("pinned", is_pin as i32)
                ]
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
            manager.remove(gv)?;
            println_i18n!(i18n, "removed-version", [("version", gv.to_display_str())]);
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
        [("version", &resolved_version.to_display_str())]
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

    warn_project_version_mismatch(i18n, manager, &version, true);

    let resolved_version = manager.auto_install_version(&version)?;

    match manager.pin_version(&resolved_version) {
        Ok(()) => println_i18n!(
            i18n,
            "pinned-success",
            [("version", &resolved_version.to_display_str())]
        ),
        Err(_) => eprintln_i18n!(
            i18n,
            "error-pin-version-not-found",
            [("version", &resolved_version.to_display_str())]
        ),
    }
    Ok(())
}
