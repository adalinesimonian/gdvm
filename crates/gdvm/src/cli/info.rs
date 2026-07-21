// SPDX-FileCopyrightText: Copyright (C) 2026 Adaline Simonian
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
use gdvm::{fs_utils, t, t_attr};

use super::VersionRequest;
use super::format::{OutputFormat, byte_display_args, format_label_value_table, print_json};

/// Information for a specific installed version of Godot.
#[derive(serde::Serialize)]
struct VersionInfo {
    version: String,
    variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    registry: Option<String>,
    install_path: String,
    executable: String,
    size_bytes: u64,
    is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_used: Option<u64>,
}

/// Handle the 'info' subcommand.
pub(crate) async fn sub_info(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let include_pre = matches.get_flag("include-pre");

    let request = VersionRequest::from_matches(matches)?;
    let resolved = request
        .resolve_selection(gdvm, include_pre, false, false)
        .await?;

    let library = gdvm.library();
    let registry = resolved.registry.as_deref();
    let (install_key, install_path) =
        library.install_dir(&resolved.version, &resolved.variant, registry)?;
    let executable =
        library.get_executable_path(&resolved.version, &resolved.variant, registry, false)?;

    let is_default = gdvm.defaults().get_default()?.is_some_and(|default| {
        default.version == resolved.version
            && default.variant == resolved.variant
            && default.registry.as_deref() == registry
    });

    let info = VersionInfo {
        version: resolved.version.to_display_str(),
        variant: resolved.variant.as_str().to_string(),
        registry: registry.map(str::to_string),
        install_path: install_path.display().to_string(),
        executable: executable.display().to_string(),
        size_bytes: fs_utils::dir_size(&install_path),
        is_default,
        last_used: library.last_used(&install_key)?,
    };

    if OutputFormat::is_json(matches) {
        return print_json(&info);
    }

    let mut rows: Vec<(String, String)> = vec![
        (
            t_attr!("info-version", "label"),
            t!("info-version", version = info.version.as_str()),
        ),
        (
            t_attr!("info-variant", "label"),
            t!("info-variant", variant = info.variant.as_str()),
        ),
    ];

    if let Some(registry) = &info.registry {
        rows.push((
            t_attr!("info-registry", "label"),
            t!("info-registry", registry = registry.as_str()),
        ));
    }

    rows.push((
        t_attr!("info-install-path", "label"),
        t!("info-install-path", path = info.install_path.as_str()),
    ));
    rows.push((
        t_attr!("info-executable", "label"),
        t!("info-executable", path = info.executable.as_str()),
    ));
    rows.push((t_attr!("info-size", "label"), {
        let (value, unit) = byte_display_args(info.size_bytes);
        t!("info-size", value = value, unit = unit)
    }));
    rows.push((
        t_attr!("info-default", "label"),
        t!("info-default", value = i64::from(info.is_default)),
    ));

    if let Some(last_used) = info.last_used {
        rows.push((
            t_attr!("info-last-used", "label"),
            t!(
                "info-last-used",
                timestamp = gdvm::date_utils::format_unix_timestamp_local(last_used)
            ),
        ));
    }

    println!("{}", format_label_value_table(&rows));

    Ok(())
}
