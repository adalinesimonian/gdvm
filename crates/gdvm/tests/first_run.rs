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

#![cfg(feature = "integration-tests")]

use std::path::PathBuf;
use std::process::Command;

use serial_test::serial;

mod common;
use common::TestHome;

fn shim_paths(home: &TestHome) -> Vec<PathBuf> {
    let bin = home.gdvm_dir().join("bin");
    if cfg!(target_os = "windows") {
        vec![bin.join("godot.exe"), bin.join("godot_console.exe")]
    } else {
        vec![bin.join("godot")]
    }
}

fn run_gdvm(home: &TestHome, arg: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gdvm"))
        .arg(arg)
        .env("GDVM_TEST_HOME", home.path())
        .output()
        .expect("gdvm runs")
}

#[test]
#[serial]
fn version_and_help_are_side_effect_free() {
    let home = TestHome::new();

    for arg in ["--version", "--help"] {
        let output = run_gdvm(&home, arg);
        assert!(output.status.success(), "{arg}: {output:?}");
    }

    for shim in shim_paths(&home) {
        assert!(
            !shim.exists(),
            "{shim:?} must not be written before the command line parses"
        );
    }
    assert!(
        !home.gdvm_dir().join("gdvm_version").exists(),
        "no post-upgrade state may be written by --version/--help"
    );
}

#[test]
#[serial]
fn a_real_command_materializes_the_shims() {
    let home = TestHome::new();

    let output = run_gdvm(&home, "diagnose");
    assert!(output.status.success(), "{output:?}");

    for shim in shim_paths(&home) {
        assert!(
            shim.is_file(),
            "shim must exist after a real command: {shim:?}"
        );

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&shim).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0, "shim must be executable: {shim:?}");
        }
    }
    assert!(home.gdvm_dir().join("gdvm_version").exists());
}

#[test]
#[serial]
fn a_second_run_is_idempotent() {
    let home = TestHome::new();

    assert!(run_gdvm(&home, "diagnose").status.success());
    let first: Vec<_> = shim_paths(&home)
        .iter()
        .map(|p| std::fs::metadata(p).unwrap().modified().unwrap())
        .collect();

    assert!(run_gdvm(&home, "diagnose").status.success());
    let second: Vec<_> = shim_paths(&home)
        .iter()
        .map(|p| std::fs::metadata(p).unwrap().modified().unwrap())
        .collect();

    assert_eq!(
        first, second,
        "an unchanged shim must not be rewritten on every run"
    );
}
