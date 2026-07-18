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
use gdvm::t;

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

    if OutputFormat::is_json(matches) {
        return print_json(&report);
    }

    if report.is_empty() {
        gdvm::ui::note(if report.dry_run {
            t!("prune-nothing-dry-run")
        } else {
            t!("prune-nothing-removed")
        });
    }

    if report.preserved_by_link > 0 {
        gdvm::ui::note(t!(
            "prune-preserved-by-link",
            count = report.preserved_by_link
        ));
    }

    if !report.is_empty() {
        let (value, unit) = byte_display_args(report.freed_bytes);
        let freed_label = if report.dry_run {
            t!("status-would-free")
        } else {
            t!("status-freed")
        };
        gdvm::ui::milestone(freed_label, t!("size-display", value = value, unit = unit));
    }

    Ok(())
}
