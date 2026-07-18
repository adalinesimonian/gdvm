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

use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use gdvm::app::Gdvm;
use gdvm::{t, terr};

use super::format::OutputFormat;

/// The outcome of a check.
#[derive(serde::Serialize)]
struct Check {
    id: &'static str,
    status: &'static str,
    detail: String,
}

pub(crate) async fn sub_diagnose(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    let mut checks: Vec<Check> = Vec::new();
    let mut problems = 0usize;

    let base = gdvm.base_path().to_path_buf();
    record(
        &mut checks,
        "base-dir",
        "ok",
        t!("diagnose-base-dir", path = base.display().to_string()),
    );

    for shim in expected_shims(&base) {
        let name = shim
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if is_executable(&shim) {
            record(
                &mut checks,
                "shim",
                "ok",
                t!("diagnose-shim-ok", name = name.as_str()),
            );
        } else {
            problems += 1;
            record(
                &mut checks,
                "shim",
                "problem",
                t!("diagnose-shim-missing", name = name.as_str()),
            );
        }
    }

    let bin_dir = base.join("bin");
    let on_path = std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|entry| entry == bin_dir));
    record(
        &mut checks,
        "path",
        if on_path { "ok" } else { "warning" },
        if on_path {
            t!("diagnose-path-ok")
        } else {
            t!(
                "diagnose-path-missing",
                path = bin_dir.display().to_string()
            )
        },
    );

    for installed in gdvm.library().list_installed()? {
        let healthy = gdvm
            .library()
            .get_executable_path(
                &installed.version,
                &installed.variant,
                installed.registry.as_deref(),
                false,
            )
            .is_ok();
        if healthy {
            record(
                &mut checks,
                "install",
                "ok",
                t!("diagnose-install-ok", version = installed.display()),
            );
        } else {
            problems += 1;
            record(
                &mut checks,
                "install",
                "problem",
                t!("diagnose-install-broken", version = installed.display()),
            );
        }
    }

    let partials = gdvm.partial_download_count();
    if partials > 0 {
        record(
            &mut checks,
            "partial-downloads",
            "warning",
            t!("diagnose-partial-downloads", count = partials),
        );
    }

    if OutputFormat::is_json(matches) {
        return super::format::print_json(&checks);
    }

    for check in &checks {
        match check.status {
            "ok" => gdvm::ui::step(t!("status-ok"), &check.detail),
            "warning" => gdvm::ui::warn(&check.detail),
            _ => gdvm::ui::error(&check.detail),
        }
    }

    if problems == 0 {
        gdvm::ui::milestone(t!("status-healthy"), t!("diagnose-healthy"));
        Ok(())
    } else {
        Err(terr!("error-diagnose-problems", count = problems).into())
    }
}

/// The shim paths this platform expects.
fn expected_shims(base: &std::path::Path) -> Vec<PathBuf> {
    let bin = base.join("bin");
    if cfg!(target_os = "windows") {
        vec![bin.join("godot.exe"), bin.join("godot_console.exe")]
    } else {
        vec![bin.join("godot")]
    }
}

/// Whether a file exists and is executable.
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(target_family = "unix"))]
    {
        path.is_file()
    }
}

fn record(checks: &mut Vec<Check>, id: &'static str, status: &'static str, detail: String) {
    checks.push(Check { id, status, detail });
}
