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
use gdvm::version::{self, Variant, VersionSpec, VersionTarget};

use super::{
    check_deprecated_csharp_flag, ensure_registry_trusted, keyword_to_version_filter,
    refresh_cache_if_requested,
};

/// Handle the 'install' subcommand
pub(crate) async fn sub_install(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let force_reinstall = matches.get_flag("force");
    let redownload = matches.get_flag("redownload");
    let refresh = matches.get_flag("refresh");
    let include_pre = matches.get_flag("include-pre");

    refresh_cache_if_requested(gdvm, refresh).await?;

    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    ensure_registry_trusted(gdvm, registry, matches.get_flag("yes")).await?;

    let requested_version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let gv = match gdvm
        .catalogs()
        .resolve_available_version(&requested_version, variant, registry, include_pre, false)
        .await?
    {
        Some(gv) => gv,
        None => {
            return Err(gdvm
                .catalogs()
                .version_not_found_error(&requested_version, variant, registry)
                .await);
        }
    };

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
