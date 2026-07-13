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
use gdvm::println_i18n;
use gdvm::version::{self, Variant, VersionSpec, VersionTarget};

use super::{
    check_deprecated_csharp_flag, ensure_registry_trusted, keyword_to_version_filter,
    refresh_cache_if_requested,
};

/// Handle the 'use' subcommand
pub(crate) async fn sub_use(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let refresh = matches.get_flag("refresh");

    let version_input = match matches.get_one::<String>("version") {
        Some(v) => v,
        None => {
            println_i18n!("provide-version-or-unset");
            return Ok(());
        }
    };

    if version_input == "unset" {
        gdvm.defaults().unset_default()?;
        println_i18n!("default-unset-success");
        return Ok(());
    }

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

    let include_pre = matches.get_flag("include-pre");

    let resolved_version = gdvm
        .installer()
        .auto_install_version(&requested_version, variant, registry, include_pre)
        .await?;

    let resolved_variant = Variant::from_option(variant);
    gdvm.defaults()
        .set_default(&resolved_version, &resolved_variant, registry)?;
    let display = version::display_version(&resolved_version, &resolved_variant, registry);
    println_i18n!("default-set-success", version = &display);

    Ok(())
}
