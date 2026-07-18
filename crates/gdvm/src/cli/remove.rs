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
use gdvm::t;

use super::VersionRequest;

/// Handle the 'remove' subcommand
pub(crate) async fn sub_remove(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let request = VersionRequest::from_matches(matches)?;
    let variant = request.variant();
    let registry = request.registry();
    let requested_version = request.installed_filter()?;

    let installed = gdvm
        .library()
        .resolve_installed_one(requested_version, variant, registry)
        .await?;

    let display = installed.display();

    gdvm::ui::milestone(t!("status-removing"), &display);
    gdvm.library().remove(
        &installed.version,
        &installed.variant,
        installed.registry.as_deref(),
    )?;
    gdvm::ui::milestone(t!("status-removed"), &display);

    Ok(())
}
