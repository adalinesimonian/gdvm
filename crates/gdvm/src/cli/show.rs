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

use gdvm::godot_manager::GodotManager;
use gdvm::run_version_resolver::{RunResolutionRequest, RunVersionResolver};
use gdvm::version_utils::{VersionSpec, VersionTarget};

use anyhow::Result;
use clap::ArgMatches;

use super::link::collect_possible_paths;
use super::{check_deprecated_csharp_flag, keyword_to_version_filter, refresh_cache_if_requested};

/// Handle the 'show' subcommand
pub(crate) async fn sub_show(manager: &GodotManager, matches: &ArgMatches) -> Result<()> {
    let raw_args: Vec<String> = Vec::new();

    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(manager, refresh).await?;

    let possible_paths = collect_possible_paths(&raw_args);

    let spec = version_input.map(|v| VersionSpec::parse(v)).transpose()?;
    let spec_variant = spec.as_ref().and_then(|s| s.variant.clone());
    let variant = check_deprecated_csharp_flag(matches, spec_variant);
    let registry = spec.as_ref().and_then(|s| s.registry.clone());

    let explicit_version = match spec.as_ref().map(|s| &s.target) {
        Some(VersionTarget::Pattern(gv)) => Some(gv.clone()),
        Some(VersionTarget::Keyword(kw)) => Some(keyword_to_version_filter(kw)),
        None => None,
    };

    let include_pre = matches.get_flag("include-pre");

    let resolver = RunVersionResolver::new(manager);
    let resolved = resolver
        .resolve(RunResolutionRequest {
            explicit: explicit_version,
            variant,
            registry,
            include_pre,
            possible_paths: &possible_paths,
            force_on_mismatch,
            install_if_missing: false,
        })
        .await?;

    let exe_path = manager.get_executable_path(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
        console,
    )?;
    println!("{}", exe_path.display());

    Ok(())
}
