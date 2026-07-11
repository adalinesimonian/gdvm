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
use gdvm::version_utils::{Variant, VersionSpec, VersionTarget};
use gdvm::{println_i18n, t};

use anyhow::{Result, anyhow};
use clap::ArgMatches;

use super::{
    check_deprecated_csharp_flag, ensure_registry_trusted, keyword_to_version_filter,
    refresh_cache_if_requested,
};

/// Print the path to the cached download archive for a resolved version.
pub(crate) async fn sub_cache_path(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
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

    let gv = gdvm
        .catalogs()
        .resolve_available_version(&requested_version, variant, registry, include_pre, false)
        .await?
        .ok_or_else(|| anyhow!(t!("error-version-not-found")))?;

    let resolved_variant = Variant::from_option(variant);
    let path = gdvm
        .installer()
        .cached_archive_path(&gv, &resolved_variant, registry)
        .await?;
    println!("{}", path.display());

    Ok(())
}

/// Handle the 'clear-cache' subcommand
pub(crate) fn sub_clear_cache(gdvm: &Gdvm) -> Result<()> {
    gdvm.clear_cache()?;
    println_i18n!("cache-cleared");
    Ok(())
}

/// Handle the 'refresh' subcommand
pub(crate) async fn sub_refresh(gdvm: &Gdvm) -> Result<()> {
    gdvm.catalogs().refresh_cache().await?;
    println_i18n!("cache-refreshed");
    Ok(())
}
