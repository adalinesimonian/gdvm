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

use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use clap::ArgMatches;
use gdvm::app::Gdvm;
use gdvm::config::Config;
use gdvm::version::VersionQuery;
use gdvm::{t, terr, ui};

mod args;
mod cache;
mod completions;
mod config;
mod format;
mod info;
mod install;
mod link;
mod list;
mod pin;
mod prune;
mod registry;
mod remove;
mod run;
mod search;
mod show;
mod upgrade;
mod use_cmd;

pub(crate) use args::build_cli;
pub(crate) use cache::{sub_cache_path, sub_clear_cache, sub_refresh};
pub(crate) use completions::sub_completions;
pub(crate) use config::sub_config;
pub(crate) use info::sub_info;
pub(crate) use install::sub_install;
pub(crate) use link::sub_link;
pub(crate) use list::sub_list;
pub(crate) use pin::sub_pin;
pub(crate) use prune::sub_prune;
pub(crate) use registry::sub_registry;
pub(crate) use remove::sub_remove;
pub(crate) use run::{RunConfig, sub_run, sub_run_inner};
pub(crate) use search::sub_search;
pub(crate) use show::sub_show;
pub(crate) use upgrade::sub_upgrade;
pub(crate) use use_cmd::sub_use;

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
    gdvm::ui::warn(t!("warning-deprecated-csharp-flag"));

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

async fn refresh_cache_if_requested(gdvm: &Gdvm, refresh: bool) -> Result<()> {
    if refresh {
        gdvm.catalogs().refresh_cache().await?;
    }

    Ok(())
}

/// Convert a keyword to a `VersionQuery` filter for resolution.
fn keyword_to_version_filter(keyword: &str) -> VersionQuery {
    if keyword == "stable" {
        VersionQuery {
            release_type: Some("stable".to_string()),
            ..Default::default()
        }
    } else {
        VersionQuery::default()
    }
}

/// Ensure the user has acknowledged the use of a third-party registry. If the
/// registry is official, this function does nothing. If the registry is not
/// trusted, it prompts the user to confirm that they trust it, unless
/// `assume_yes` is true, in which case it automatically trusts the registry
/// after a brief pause.
async fn ensure_registry_trusted(
    gdvm: &Gdvm,
    registry: Option<&str>,
    assume_yes: bool,
) -> Result<()> {
    if gdvm.catalogs().is_official_registry(registry) {
        return Ok(());
    }

    let name = registry.expect("a non-official registry always has a name");
    let url = gdvm.catalogs().registry_base_url(name)?;

    // Always warn when using a third-party registry, even after it has been confirmed.
    gdvm::ui::warn(t!(
        "registry-trust-warning",
        registry = name,
        url = url.as_str()
    ));

    if Config::load()?.is_registry_trusted(&url) {
        return Ok(());
    }

    if assume_yes {
        // Warn and allow the user some time to cancel.
        ui::warn(t!(
            "registry-trust-bypass",
            registry = name,
            url = url.as_str()
        ));
        std::thread::sleep(std::time::Duration::from_secs(5));
        return Config::modify(|config| {
            config.trust_registry(&url);
            Ok(())
        });
    }

    if !std::io::stdin().is_terminal() {
        return Err(terr!(
            "error-non-interactive-trust",
            registry = name,
            url = url.as_str()
        ));
    }

    eprint!("{} ", t!("registry-trust-prompt"));
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != t!("confirm-yes") {
        return Err(terr!("registry-trust-aborted"));
    }

    Config::modify(|config| {
        config.trust_registry(&url);
        Ok(())
    })
}
