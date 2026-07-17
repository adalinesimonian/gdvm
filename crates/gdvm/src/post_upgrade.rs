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

use std::path::Path;

use anyhow::Result;
use semver::Version;

use crate::locks::{Lock, Resource};
use crate::{migrations, terr};

/// Name of the file that stores the gdvm version that last ran.
const VERSION_FILE: &str = "gdvm_version";

/// When a post-upgrade action runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    /// Runs before data migrations.
    BeforeMigrations,
    /// Runs after data migrations.
    AfterMigrations,
}

/// An action that runs after gdvm is upgraded.
struct PostUpgradeAction {
    /// ID.
    id: &'static str,
    /// When the action runs.
    stage: Stage,
    /// The action to perform.
    run: fn(&Path) -> Result<()>,
}

const ACTIONS: &[PostUpgradeAction] = &[PostUpgradeAction {
    id: "reinstall-shims",
    stage: Stage::BeforeMigrations,
    run: crate::shims::ensure,
}];

/// Run post-upgrade actions and migrations.
pub fn run(base_path: &Path) -> Result<()> {
    let current = current_version();

    if read_version(base_path)?.as_ref() == Some(&current) {
        return migrations::run_migrations(base_path);
    }

    let _lock = Lock::acquire(&base_path.join("locks"), Resource::PostUpgrade)?;

    if read_version(base_path)?.as_ref() == Some(&current) {
        return migrations::run_migrations(base_path);
    }

    run_stage(base_path, Stage::BeforeMigrations)?;
    migrations::run_migrations(base_path)?;
    run_stage(base_path, Stage::AfterMigrations)?;

    write_version(base_path, &current)?;

    Ok(())
}

/// Get the version of the running gdvm build.
fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("crate version is valid semver")
}

/// Run actions for the given stage.
fn run_stage(base_path: &Path, stage: Stage) -> Result<()> {
    for action in ACTIONS {
        if action.stage == stage {
            (action.run)(base_path).map_err(|err| {
                terr!("error-post-upgrade-action-failed", id = action.id)
                    .with_string_source(err.to_string())
            })?;
        }
    }

    Ok(())
}

/// Get the stored gdvm version.
fn read_version(base_path: &Path) -> Result<Option<Version>> {
    Ok(
        crate::fs_utils::read_marker_line(&base_path.join(VERSION_FILE))?
            .and_then(|line| Version::parse(&line).ok()),
    )
}

/// Store `version` as the gdvm version that last ran.
fn write_version(base_path: &Path, version: &Version) -> Result<()> {
    crate::fs_utils::write_marker_line(&base_path.join(VERSION_FILE), &version.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn shim_path(base: &Path) -> std::path::PathBuf {
        let name = if cfg!(target_os = "windows") {
            "godot.exe"
        } else {
            "godot"
        };
        base.join("bin").join(name)
    }

    fn skip_migrations(base: &Path) {
        fs::write(base.join("data_version"), "1000\n").unwrap();
    }

    #[test]
    fn upgrade_reinstalls_shims() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        skip_migrations(base);
        write_version(base, &Version::parse("0.0.1").unwrap()).unwrap();

        let shim = shim_path(base);
        fs::create_dir_all(shim.parent().unwrap()).unwrap();
        fs::write(&shim, b"stale-shim").unwrap();

        run(base).unwrap();

        assert_ne!(
            fs::read(&shim).unwrap(),
            b"stale-shim",
            "an upgrade must rewrite the shim"
        );

        let recorded = read_version(base).unwrap().unwrap();
        assert_eq!(recorded, current_version());
    }

    #[test]
    fn same_version_leaves_shims_untouched() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        skip_migrations(base);
        write_version(base, &current_version()).unwrap();

        let shim = shim_path(base);
        fs::create_dir_all(shim.parent().unwrap()).unwrap();
        fs::write(&shim, b"stale-shim").unwrap();

        run(base).unwrap();

        assert_eq!(
            fs::read(&shim).unwrap(),
            b"stale-shim",
            "no upgrade means the shim must be left alone"
        );
    }

    #[test]
    fn fresh_install_installs_shims() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        skip_migrations(base);

        let shim = shim_path(base);
        fs::create_dir_all(shim.parent().unwrap()).unwrap();
        fs::write(&shim, b"stale-shim").unwrap();

        run(base).unwrap();

        assert_ne!(
            fs::read(&shim).unwrap(),
            b"stale-shim",
            "a fresh install must install the shim"
        );

        let recorded = read_version(base).unwrap().unwrap();
        assert_eq!(recorded, current_version());
    }

    #[test]
    fn downgrade_reinstalls_shims() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        skip_migrations(base);
        write_version(base, &Version::parse("9999.0.0").unwrap()).unwrap();

        let shim = shim_path(base);
        fs::create_dir_all(shim.parent().unwrap()).unwrap();
        fs::write(&shim, b"stale-shim").unwrap();

        run(base).unwrap();

        assert_ne!(
            fs::read(&shim).unwrap(),
            b"stale-shim",
            "a downgrade must reinstall the running build's shim"
        );

        let recorded = read_version(base).unwrap().unwrap();
        assert_eq!(recorded, current_version());
    }
}
