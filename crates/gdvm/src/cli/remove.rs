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
use gdvm::version::{self, VersionSpec, VersionTarget};
use gdvm::{t, terr};

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
            return Err(terr!("error-version-not-found"));
        }
        VersionTarget::Pattern(gv) => gv.clone(),
    };

    let resolved_versions = gdvm
        .library()
        .resolve_installed_version(&requested_version, variant, registry)
        .await?;

    match resolved_versions.len() {
        0 => {
            return Err(terr!("error-version-not-found"));
        }
        1 => {
            let installed = &resolved_versions[0];
            let display = version::display_version(
                &installed.version,
                &installed.variant,
                installed.registry.as_deref(),
            );

            gdvm::ui::milestone(t!("status-removing"), &display);

            gdvm.library().remove(
                &installed.version,
                &installed.variant,
                installed.registry.as_deref(),
            )?;
            gdvm::ui::milestone(t!("status-removed"), &display);
        }
        _ => {
            for installed in &resolved_versions {
                println!(
                    "- {}",
                    version::display_version(
                        &installed.version,
                        &installed.variant,
                        installed.registry.as_deref(),
                    )
                );
            }
            return Err(terr!("error-multiple-versions-found"));
        }
    }

    Ok(())
}
