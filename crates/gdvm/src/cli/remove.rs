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
use gdvm::version::{self, VersionSpec, VersionTarget};
use gdvm::{eprintln_i18n, println_i18n, t};

use anyhow::{Result, anyhow};
use clap::ArgMatches;
use std::io::{self, Write};

use super::check_deprecated_csharp_flag;

/// Handle the 'remove' subcommand
pub(crate) async fn sub_remove(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let version_input = matches.get_one::<String>("version").unwrap();
    let spec = VersionSpec::parse(version_input)?;
    let variant = check_deprecated_csharp_flag(matches, spec.variant);
    let variant = variant.as_deref();
    let registry = spec.registry.as_deref();

    let requested_version = match &spec.target {
        VersionTarget::Keyword(_) => {
            return Err(anyhow!(t!("error-version-not-found")));
        }
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let resolved_versions = gdvm
        .library()
        .resolve_installed_version(&requested_version, variant, registry)
        .await?;

    match resolved_versions.len() {
        0 => {
            eprintln_i18n!("error-version-not-found");
        }
        1 => {
            let installed = &resolved_versions[0];
            let display = version::display_version(
                &installed.version.to_display_str(),
                &installed.variant,
                installed.registry.as_deref(),
            );

            println_i18n!("removing-version", version = &display);

            if !matches.get_flag("yes") {
                println_i18n!("confirm-remove");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != t!("confirm-yes") {
                    println_i18n!("remove-cancelled");
                    return Ok(());
                }
            }
            gdvm.library().remove(
                &installed.version,
                &installed.variant,
                installed.registry.as_deref(),
            )?;
            println_i18n!("removed-version", version = &display);
        }
        _ => {
            eprintln_i18n!("error-multiple-versions-found");
            for installed in &resolved_versions {
                println!(
                    "- {}",
                    version::display_version(
                        &installed.version.to_display_str(),
                        &installed.variant,
                        installed.registry.as_deref(),
                    )
                );
            }
        }
    }

    Ok(())
}
