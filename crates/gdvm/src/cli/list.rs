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

use gdvm::app::Gdvm;
use gdvm::println_i18n;
use gdvm::version::{self};

use anyhow::Result;
use clap::ArgMatches;

use super::format::{OutputFormat, VersionEntry, print_json};

/// Handle the 'list' subcommand
pub(crate) fn sub_list(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let versions = gdvm.library().list_installed()?;

    if OutputFormat::from_matches(matches) == OutputFormat::Json {
        let entries: Vec<VersionEntry> = versions
            .iter()
            .map(|v| VersionEntry {
                version: v.version.to_display_str(),
                variant: Some(v.variant.as_str().to_string()),
                registry: v.registry.clone(),
            })
            .collect();
        return print_json(&entries);
    }

    if versions.is_empty() {
        println_i18n!("no-versions-installed");
    } else {
        println_i18n!("installed-versions");
        for v in &versions {
            println!(
                "- {}",
                version::display_version(
                    &v.version.to_display_str(),
                    &v.variant,
                    v.registry.as_deref(),
                )
            );
        }
    }
    Ok(())
}
