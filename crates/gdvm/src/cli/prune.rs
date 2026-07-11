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

use gdvm::app::{Gdvm, PruneOptions};
use gdvm::config::{self};
use gdvm::println_i18n;

use anyhow::Result;
use clap::ArgMatches;

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
pub(crate) fn sub_prune(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let opts = PruneOptions {
        all: matches.get_flag("all"),
        force: matches.get_flag("force"),
        dry_run: matches.get_flag("dry-run"),
    };

    let config = config::Config::load().unwrap_or_default();
    let max_age_secs = config.prune_max_age_days().saturating_mul(24 * 60 * 60);

    let report = gdvm.pruner().prune(max_age_secs, opts)?;

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
