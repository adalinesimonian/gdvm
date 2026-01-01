use gdvm::config::{self, ConfigOps};
use gdvm::godot_manager::{GodotManager, InstallOutcome};
use gdvm::i18n::I18n;
use gdvm::run_version_resolver::{
    RunResolutionRequest, RunVersionResolver, warn_project_version_mismatch,
};
use gdvm::version_utils::{self, GodotVersion};
use gdvm::{eprintln_i18n, println_i18n, t};

use anyhow::{Result, anyhow};
use clap::{Arg, ArgMatches, Command, value_parser};
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

async fn refresh_cache_if_requested(manager: &GodotManager<'_>, refresh: bool) -> Result<()> {
    if refresh {
        manager.refresh_cache().await?;
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
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
            csharp_given: false,
            csharp_flag: false,
            console: console_mode,
            raw_args: &args,
            force_on_mismatch: false,
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
                )
                .arg(refresh_flag(&i18n)),
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
                )
                .arg(refresh_flag(&i18n)),
        )
        .subcommand(
            Command::new("show")
                .about(t!(i18n, "help-show"))
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
                        .value_parser(version_utils::validate_remote_version)
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
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(t!(i18n, "help-csharp")),
                )
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
                .arg(
                    Arg::new("csharp")
                        .long("csharp")
                        .num_args(0)
                        .help(t!(i18n, "help-csharp")),
                )
                .arg(refresh_flag(&i18n)),
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

    refresh_cache_if_requested(manager, refresh).await?;

    let requested_version = GodotVersion::from_match_str(version_input)?;
    let mut gv = manager
        .resolve_available_version(&requested_version, false)
        .await?
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

    match manager.install(&gv, force_reinstall, redownload).await? {
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
            println!("- {v}");
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

    // specifically check if --csharp was provided as a flag or if we're reading the default value
    let csharp_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    let csharp_flag = matches.get_flag("csharp");
    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

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
    .await
}

/// Handle the 'show' subcommand
async fn sub_show(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let raw_args: Vec<String> = Vec::new();

    let csharp_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    let csharp_flag = matches.get_flag("csharp");
    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let possible_paths = collect_possible_paths(&raw_args);

    let explicit_version = if let Some(v) = version_input {
        Some(GodotVersion::from_match_str(v)?)
    } else {
        None
    };

    let resolver = RunVersionResolver::new(manager, i18n);
    let resolved_version = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            possible_paths: &possible_paths,
            csharp_given,
            csharp_flag,
            force_on_mismatch,
            install_if_missing: false,
        })
        .await?;

    let exe_path = manager.get_executable_path(&resolved_version, console)?;
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

    let explicit_version = if let Some(v) = version_input {
        Some(GodotVersion::from_match_str(v)?)
    } else {
        None
    };

    let csharp_given =
        matches.value_source("csharp") != Some(clap::parser::ValueSource::DefaultValue);
    let csharp_flag = matches.get_flag("csharp");
    let force = matches.get_flag("force");
    let copy = matches.get_flag("copy");

    let resolver = RunVersionResolver::new(manager, i18n);
    let resolved_version = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            possible_paths: &[],
            csharp_given,
            csharp_flag,
            force_on_mismatch: force,
            install_if_missing: false,
        })
        .await?;

    let primary_exe = manager.get_executable_path(&resolved_version, false)?;

    #[cfg(target_os = "windows")]
    let console_exe = {
        let console_exe = manager.get_executable_path(&resolved_version, true)?;
        if console_exe != primary_exe {
            Some(console_exe)
        } else {
            None
        }
    };

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
                version = resolved_version.to_display_str(),
                path = link_path.display().to_string()
            );
        } else {
            println_i18n!(
                i18n,
                "link-created",
                version = resolved_version.to_display_str(),
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

    if resolved_version.is_csharp.unwrap_or(false) {
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
            version = resolved_version.to_display_str(),
            path = link_path.display().to_string()
        );
    } else {
        println_i18n!(
            i18n,
            "link-created",
            version = resolved_version.to_display_str(),
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
        std::os::windows::fs::symlink_file(target, link)
            .map_err(|e| anyhow!(t!(i18n, "error-link-symlink", error = e.to_string())))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
            .map_err(|e| anyhow!(t!(i18n, "error-link-symlink", error = e.to_string())))?;
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
        std::os::windows::fs::symlink_dir(target, link)
            .map_err(|e| anyhow!(t!(i18n, "error-link-symlink", error = e.to_string())))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
            .map_err(|e| anyhow!(t!(i18n, "error-link-symlink", error = e.to_string())))?;
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
    csharp_given: bool,
    csharp_flag: bool,
    console: bool,
    raw_args: &'a Vec<String>,
    force_on_mismatch: bool,
}

/// Run the Godot executable
async fn sub_run_inner(config: RunConfig<'_>) -> Result<()> {
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
    let possible_paths = collect_possible_paths(raw_args);

    let explicit_version = if let Some(v) = version_input {
        Some(GodotVersion::from_match_str(v)?)
    } else {
        None
    };

    let resolver = RunVersionResolver::new(manager, i18n);
    let resolved_version = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            possible_paths: &possible_paths,
            csharp_given,
            csharp_flag,
            force_on_mismatch,
            install_if_missing: true,
        })
        .await?;

    eprintln_i18n!(
        i18n,
        "running-version",
        version = &resolved_version.to_display_str()
    );

    manager.run(&resolved_version, console, raw_args)?;

    Ok(())
}

/// Handle the 'remove' subcommand
async fn sub_remove(i18n: &I18n, manager: &GodotManager<'_>, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let csharp = matches.get_flag("csharp");
    let mut requested_version = GodotVersion::from_match_str(version_input)?;

    requested_version.is_csharp = Some(csharp);

    let resolved_versions = manager
        .resolve_installed_version(&requested_version)
        .await?;

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
    let csharp = matches.get_flag("csharp");
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

    let mut requested_version = GodotVersion::from_match_str(version_input)?;

    requested_version.is_csharp = Some(csharp);

    let resolved_version = manager.auto_install_version(&requested_version).await?;

    manager.set_default(&resolved_version)?;
    println_i18n!(
        i18n,
        "default-set-success",
        version = &resolved_version.to_display_str(),
    );

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
    let csharp = matches.get_flag("csharp");
    let refresh = matches.get_flag("refresh");
    let mut version = GodotVersion::from_match_str(version_str)?;

    refresh_cache_if_requested(manager, refresh).await?;

    version.is_csharp = Some(csharp);

    warn_project_version_mismatch::<_, &Path>(manager, i18n, &version, true, None).await;

    let resolved_version = manager.auto_install_version(&version).await?;

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
