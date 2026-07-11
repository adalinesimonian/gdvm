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
use gdvm::version_utils::GodotVersion;

use anyhow::Result;
use clap::ArgMatches;

use super::{ensure_registry_trusted, refresh_cache_if_requested};

/// Handle the 'search' subcommand
pub(crate) async fn sub_search(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let filter = matches.get_one::<String>("filter").map(|s| s.as_str());
    let include_pre = matches.get_flag("include-pre");
    let cache_only = matches.get_flag("cache-only");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(gdvm, refresh).await?;

    let (registry, version_filter) = match filter {
        Some(f) => match f.split_once('/') {
            Some((reg, rest)) if !reg.is_empty() => (Some(reg), Some(rest)),
            _ => (None, Some(f)),
        },
        None => (None, None),
    };

    ensure_registry_trusted(gdvm, registry, matches.get_flag("yes")).await?;

    let requested_version = match version_filter {
        Some(filter) => Some(GodotVersion::from_match_str(filter)?),
        None => None,
    };

    let mut releases = gdvm
        .catalogs()
        .fetch_available_releases(registry, &requested_version, cache_only)
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
        println_i18n!("no-matching-releases");
    } else {
        println_i18n!("available-releases");
        for r in releases {
            println!("- {r}");
        }
    }
    Ok(())
}
