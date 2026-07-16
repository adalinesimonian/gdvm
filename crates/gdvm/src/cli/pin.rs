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
use gdvm::version::{self, Variant, VersionSpec, VersionTarget};
use gdvm::{println_i18n, terr};

use super::{
    check_deprecated_csharp_flag, ensure_registry_trusted, keyword_to_version_filter,
    refresh_cache_if_requested,
};

/// Handle the 'pin' subcommand
pub(crate) async fn sub_pin(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let version_str = matches.get_one::<String>("version").unwrap();
    let refresh = matches.get_flag("refresh");
    let spec = VersionSpec::parse(version_str)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    refresh_cache_if_requested(gdvm, refresh).await?;

    ensure_registry_trusted(gdvm, registry, matches.get_flag("yes")).await?;

    let version = match &spec.target {
        VersionTarget::Keyword(kw) => keyword_to_version_filter(kw),
        VersionTarget::Pattern(gv) => gv.clone(),
    };

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
