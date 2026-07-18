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
use gdvm::app::Gdvm;
use gdvm::version::Variant;

use super::VersionRequest;

/// Print the path to the cached download archive for a resolved version.
pub(crate) async fn sub_cache_path(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let include_pre = matches.get_flag("include-pre");

    let request = VersionRequest::from_matches(matches)?;
    request.prepare(gdvm, matches).await?;
    let variant = request.variant();
    let registry = request.registry();
    let requested_version = request.required_filter().clone();

    let gv = gdvm
        .catalogs()
        .resolve_available_or_not_found(&requested_version, variant, registry, include_pre, false)
        .await?;

    let resolved_variant = Variant::from_option(variant);
    let path = gdvm
        .installer()
        .cached_archive_path(&gv, &resolved_variant, registry)
        .await?;

    if super::format::OutputFormat::is_json(matches) {
        #[derive(serde::Serialize)]
        struct CachePath {
            path: String,
        }

        return super::format::print_json(&CachePath {
            path: path.display().to_string(),
        });
    }

    println!("{}", path.display());

    Ok(())
}

/// Handle the 'clear-cache' subcommand
pub(crate) fn sub_clear_cache(gdvm: &Gdvm) -> Result<()> {
    gdvm.clear_cache()?;
    gdvm::ui::milestone(gdvm::t!("status-cleared"), gdvm::t!("subject-cache"));
    Ok(())
}

/// Handle the 'refresh' subcommand
pub(crate) async fn sub_refresh(gdvm: &Gdvm) -> Result<()> {
    gdvm.catalogs().refresh_cache().await?;
    gdvm::ui::milestone(gdvm::t!("status-refreshed"), gdvm::t!("subject-cache"));
    Ok(())
}
