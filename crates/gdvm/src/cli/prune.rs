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

use anyhow::Result;
use clap::ArgMatches;
use gdvm::app::{Gdvm, PruneOptions};
use gdvm::config::{self};
use gdvm::println_i18n;

use super::format::{OutputFormat, byte_display_args, print_json};

/// Handle the 'prune' subcommand
pub(crate) fn sub_prune(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let opts = PruneOptions {
        all: matches.get_flag("all"),
        force: matches.get_flag("force"),
        dry_run: matches.get_flag("dry-run"),
    };

    let config = config::Config::load().unwrap_or_default();
    let max_age_secs = config.prune_max_age_days().saturating_mul(24 * 60 * 60);

    let report = gdvm.pruner().prune(max_age_secs, opts)?;

    if OutputFormat::from_matches(matches) == OutputFormat::Json {
        return print_json(&report);
    }

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
            let (value, unit) = byte_display_args(item.freed_bytes);
            println_i18n!(
                "prune-item",
                label = item.label.as_str(),
                value = value,
                unit = unit
            );
        }
    }

    if !report.archives.is_empty() {
        println_i18n!("prune-archives-header");
        for item in &report.archives {
            let (value, unit) = byte_display_args(item.freed_bytes);
            println_i18n!(
                "prune-item",
                label = item.label.as_str(),
                value = value,
                unit = unit
            );
        }
    }

    if report.preserved_by_link > 0 {
        println_i18n!("prune-preserved-by-link", count = report.preserved_by_link);
    }

    let (value, unit) = byte_display_args(report.freed_bytes);
    if report.dry_run {
        println_i18n!("prune-would-free", value = value, unit = unit);
    } else {
        println_i18n!("prune-freed", value = value, unit = unit);
    }

    Ok(())
}
