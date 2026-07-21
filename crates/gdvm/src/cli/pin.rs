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

use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use gdvm::app::Gdvm;
use gdvm::run_version_resolver::warn_project_version_mismatch;
use gdvm::version::{self, Variant};
use gdvm::{println_i18n, terr};

use super::VersionRequest;

/// Handle the 'pin' subcommand
pub(crate) async fn sub_pin(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let request = VersionRequest::from_matches(matches)?;
    request.prepare(gdvm, matches).await?;
    let variant = request.variant();
    let registry = request.registry();
    let version = request.required_filter().clone();

    warn_project_version_mismatch::<_, &Path>(gdvm, &version, true, None).await;

    let include_pre = matches.get_flag("include-pre");

    let resolved_version = gdvm
        .installer()
        .auto_install_version(&version, variant, registry, include_pre)
        .await?;

    let resolved_variant = Variant::from_option(variant);
    let display = version::display_version(&resolved_version, &resolved_variant, registry);

    let skip_gdvmrc = matches.get_flag("no-legacy");

    gdvm.defaults()
        .pin_version(&resolved_version, &resolved_variant, registry, skip_gdvmrc)
        .map_err(|_| terr!("error-pin-version-not-found", version = display.clone()))?;

    println_i18n!("pinned-success", version = &display);
    Ok(())
}
