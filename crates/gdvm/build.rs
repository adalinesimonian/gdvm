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

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let shim_name = if target.contains("windows") {
        "shim.exe"
    } else {
        "shim"
    };

    println!("cargo:rerun-if-env-changed=GDVM_PREBUILT_SHIM");
    if let Ok(prebuilt) = env::var("GDVM_PREBUILT_SHIM") {
        fs::copy(&prebuilt, out_dir.join(shim_name)).expect("failed to copy prebuilt shim binary");
        println!("cargo:rerun-if-changed={prebuilt}");
        return;
    }

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(&target)
        // Build shim as standalone package to avoid workspace lock.
        .arg("--manifest-path")
        .arg(
            workspace_root
                .join("crates")
                .join("shim")
                .join("Cargo.toml"),
        )
        .arg("--target-dir")
        .arg(workspace_root.join("intermediate"))
        .status()
        .expect("failed to build shim");

    assert!(status.success());

    let shim_source = workspace_root
        .join("intermediate")
        .join(&target)
        .join("release")
        .join(shim_name);
    let shim_dest = out_dir.join(shim_name);
    fs::copy(&shim_source, &shim_dest).expect("failed to copy shim binary");

    println!("cargo:rerun-if-changed=../shim/src/main.rs");
    println!("cargo:rerun-if-changed=../shim/Cargo.toml");
}
