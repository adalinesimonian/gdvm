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
use gdvm::i18n::I18n;

use anyhow::Result;

mod cli;

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

        if let Err(err) = cli::sub_run_inner(cli::RunConfig {
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

    let matches = cli::build_cli().get_matches();

    // Match the subcommand and call the appropriate function
    match matches.subcommand() {
        Some(("install", sub_m)) => cli::sub_install(&manager, sub_m).await?,
        Some(("list", _)) => cli::sub_list(&manager)?,
        Some(("run", sub_m)) => cli::sub_run(&manager, sub_m).await?,
        Some(("show", sub_m)) => cli::sub_show(&manager, sub_m).await?,
        Some(("cache-path", sub_m)) => cli::sub_cache_path(&manager, sub_m).await?,
        Some(("link", sub_m)) => cli::sub_link(&manager, sub_m).await?,
        Some(("remove", sub_m)) => cli::sub_remove(&manager, sub_m).await?,
        Some(("search", sub_m)) => cli::sub_search(&manager, sub_m).await?,
        Some(("clear-cache", _)) => cli::sub_clear_cache(&manager)?,
        Some(("refresh", _)) => cli::sub_refresh(&manager).await?,
        Some(("prune", sub_m)) => cli::sub_prune(&manager, sub_m)?,
        Some(("use", sub_m)) => cli::sub_use(&manager, sub_m).await?,
        Some(("upgrade", sub_m)) => cli::sub_upgrade(&manager, sub_m).await?,
        Some(("pin", sub_m)) => cli::sub_pin(&manager, sub_m).await?,
        Some(("config", sub_m)) => cli::sub_config(sub_m)?,
        Some(("registry", sub_m)) => cli::sub_registry(&manager, sub_m).await?,
        _ => {}
    }

    Ok(())
}
