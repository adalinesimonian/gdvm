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

use super::VersionRequest;
use super::format::{OutputFormat, print_json};

/// Handle the 'show' subcommand
pub(crate) async fn sub_show(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let include_pre = matches.get_flag("include-pre");

    let request = VersionRequest::from_matches(matches)?;
    request.prepare(gdvm, matches).await?;

    let resolved = request
        .resolve_selection(gdvm, include_pre, false, force_on_mismatch)
        .await?;

    let exe_path = gdvm.library().get_executable_path(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
        console,
    )?;

    if OutputFormat::is_json(matches) {
        #[derive(serde::Serialize)]
        struct Shown {
            version: String,
            variant: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            registry: Option<String>,
            path: String,
        }
        return print_json(&Shown {
            version: resolved.version.to_display_str(),
            variant: resolved.variant.as_str().to_string(),
            registry: resolved.registry.clone(),
            path: exe_path.display().to_string(),
        });
    }

    println!("{}", exe_path.display());

    Ok(())
}
