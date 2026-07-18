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
use gdvm::app::{Gdvm, InstallOutcome};
use gdvm::println_i18n;
use gdvm::version::{self, Variant};

use super::VersionRequest;

/// Handle the 'install' subcommand
pub(crate) async fn sub_install(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let force_reinstall = matches.get_flag("force");
    let redownload = matches.get_flag("redownload");
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
    let display = version::display_version(&gv, &resolved_variant, registry);

    match gdvm
        .installer()
        .install(
            &gv,
            &resolved_variant,
            registry,
            force_reinstall,
            redownload,
        )
        .await?
    {
        InstallOutcome::Installed => {
            // Final status already printed during installation.
        }
        InstallOutcome::AlreadyInstalled => {
            // Print a message indicating the version is already installed
            println_i18n!("version-already-installed", version = &display);
            return Ok(());
        }
    }

    Ok(())
}
