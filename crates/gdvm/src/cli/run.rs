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
use gdvm::eprintln_i18n;
use gdvm::run_version_resolver::{RunResolutionRequest, RunVersionResolver};
use gdvm::version::{self, VersionSpec, VersionTarget};

use super::link::collect_possible_paths;
use super::{
    check_deprecated_csharp_flag, ensure_registry_trusted, keyword_to_version_filter,
    refresh_cache_if_requested,
};

/// Handle the 'run' subcommand
pub(crate) async fn sub_run(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    // Capture args after "--" to pass directly to child
    let raw_args = match std::env::args().position(|x| x == "--") {
        Some(pos) => std::env::args().skip(pos + 1).collect(),
        None => Vec::new(),
    };

    let version_input = matches.get_one::<String>("version");
    let console = matches.get_flag("console");
    let force_on_mismatch = matches.get_flag("force");
    let refresh = matches.get_flag("refresh");

    refresh_cache_if_requested(gdvm, refresh).await?;

    let spec_variant = version_input
        .map(|v| VersionSpec::parse(v))
        .transpose()?
        .and_then(|s| s.variant);

    let variant = check_deprecated_csharp_flag(matches, spec_variant);
    let include_pre = matches.get_flag("include-pre");

    sub_run_inner(RunConfig {
        gdvm,
        version_input,
        variant,
        console,
        raw_args: &raw_args,
        force_on_mismatch,
        include_pre,
        assume_yes: matches.get_flag("yes"),
    })
    .await
}

/// Configuration for the `sub_run_inner` function
pub(crate) struct RunConfig<'a> {
    pub(crate) gdvm: &'a Gdvm,
    pub(crate) version_input: Option<&'a String>,
    pub(crate) variant: Option<String>,
    pub(crate) console: bool,
    pub(crate) raw_args: &'a Vec<String>,
    pub(crate) force_on_mismatch: bool,
    pub(crate) include_pre: bool,
    pub(crate) assume_yes: bool,
}

/// Run the Godot executable
pub(crate) async fn sub_run_inner(config: RunConfig<'_>) -> Result<()> {
    let RunConfig {
        gdvm,
        version_input,
        variant,
        console,
        raw_args,
        force_on_mismatch,
        include_pre,
        assume_yes,
    } = config;

    // Try to see if a path was given in raw_args. First, by checking if the --path flag was given
    // and then by checking if the first argument is a path. Prefer the --path flag if both are
    // given.
    let possible_paths = collect_possible_paths(raw_args);

    let (explicit_version, resolved_variant, resolved_registry) = if let Some(v) = version_input {
        let spec = VersionSpec::parse(v)?;
        let var = spec.variant.or(variant);
        let reg = spec.registry;
        let ver = match spec.target {
            VersionTarget::Pattern(gv) => Some(gv),
            VersionTarget::Keyword(ref kw) => Some(keyword_to_version_filter(kw)),
        };
        (ver, var, reg)
    } else {
        (None, variant, None)
    };

    let resolver = RunVersionResolver::new(gdvm);
    let request = RunResolutionRequest {
        explicit: explicit_version,
        variant: resolved_variant,
        registry: resolved_registry,
        include_pre,
        possible_paths: &possible_paths,
        force_on_mismatch,
        install_if_missing: true,
    };

    let trust_registry = resolver.select(&request).await?.and_then(|s| s.registry);
    ensure_registry_trusted(gdvm, trust_registry.as_deref(), assume_yes).await?;

    let resolved = resolver.resolve(request).await?;

    let display = version::display_version(
        &resolved.version.to_display_str(),
        &resolved.variant,
        resolved.registry.as_deref(),
    );

    eprintln_i18n!("running-version", version = &display);

    gdvm.launcher().run(
        &resolved.version,
        &resolved.variant,
        resolved.registry.as_deref(),
        console,
        raw_args,
    )?;

    Ok(())
}
