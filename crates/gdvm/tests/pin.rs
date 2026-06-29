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
use gdvm::i18n::I18n;
use gdvm::version_utils::{GodotVersion, GodotVersionDeterminate, Variant};
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

struct PinTestEnv {
    home: TempDir,
    project: TempDir,
    prev_home: Option<std::ffi::OsString>,
    prev_cwd: Option<std::path::PathBuf>,
}

impl PinTestEnv {
    fn new() -> Self {
        let home = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        let prev_home = std::env::var_os("GDVM_TEST_HOME");
        let prev_cwd = std::env::current_dir().ok();

        // Save an update check timestamp to avoid triggering update checks.
        let gdvm_dir = home.path().join(".gdvm");
        fs::create_dir_all(&gdvm_dir).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        fs::write(
            gdvm_dir.join("cache.json"),
            format!(
                r#"{{"gdvm":{{"last_update_check":{now},"new_version":null,"new_major_version":null}},"godot_registry":{{"last_fetched":0,"releases":[]}}}}"#
            ),
        )
        .unwrap();

        unsafe {
            std::env::set_var("GDVM_TEST_HOME", home.path());
        }
        std::env::set_current_dir(project.path()).unwrap();

        Self {
            home,
            project,
            prev_home,
            prev_cwd,
        }
    }

    /// The project directory that is also the cwd.
    fn project_dir(&self) -> &Path {
        self.project.path()
    }
}

impl Drop for PinTestEnv {
    fn drop(&mut self) {
        // Move the cwd out of the temp project dir before it is deleted so the
        // delete works on Windows.
        if let Some(cwd) = &self.prev_cwd {
            let _ = std::env::set_current_dir(cwd);
        }

        unsafe {
            match &self.prev_home {
                Some(v) => std::env::set_var("GDVM_TEST_HOME", v),
                None => std::env::remove_var("GDVM_TEST_HOME"),
            }
        }

        // Own for the lifetime of the test.
        let _ = &self.home;
        let _ = &self.project;
    }
}

fn determinate(install_str: &str) -> GodotVersionDeterminate {
    GodotVersion::from_install_str(install_str)
        .unwrap()
        .to_determinate()
}

async fn manager(i18n: &I18n) -> GodotManager<'_> {
    GodotManager::new(i18n).await.unwrap()
}

#[tokio::test]
#[serial]
async fn pin_writes_gdvm_toml_and_legacy_gdvmrc() {
    let env = PinTestEnv::new();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    mgr.pin_version(&determinate("4.3-stable"), &Variant::default(), None, false)
        .unwrap();

    let toml = fs::read_to_string(env.project_dir().join("gdvm.toml")).unwrap();
    assert!(
        toml.contains("version = \"default:4.3-stable\""),
        "gdvm.toml should use the new specifier format, got: {toml}"
    );

    let rc = fs::read_to_string(env.project_dir().join(".gdvmrc")).unwrap();
    assert_eq!(rc, "4.3.0-stable");
}

#[tokio::test]
#[serial]
async fn pin_csharp_writes_variant_formats() {
    let env = PinTestEnv::new();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    mgr.pin_version(
        &determinate("4.3-stable"),
        &Variant::from_option(Some("csharp")),
        None,
        false,
    )
    .unwrap();

    let toml = fs::read_to_string(env.project_dir().join("gdvm.toml")).unwrap();
    assert!(
        toml.contains("version = \"csharp:4.3-stable\""),
        "gdvm.toml should carry the csharp variant prefix, got: {toml}"
    );

    let rc = fs::read_to_string(env.project_dir().join(".gdvmrc")).unwrap();
    assert_eq!(rc, "4.3.0-stable-csharp");
}

#[tokio::test]
#[serial]
async fn pin_no_legacy_skips_gdvmrc() {
    let env = PinTestEnv::new();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    mgr.pin_version(&determinate("4.3-stable"), &Variant::default(), None, true)
        .unwrap();

    assert!(
        env.project_dir().join("gdvm.toml").is_file(),
        "gdvm.toml should always be written"
    );
    assert!(
        !env.project_dir().join(".gdvmrc").exists(),
        ".gdvmrc must not be written when gdvm_toml_only is set"
    );
}

#[tokio::test]
#[serial]
async fn get_pinned_prefers_gdvm_toml_over_gdvmrc() {
    let env = PinTestEnv::new();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    fs::write(
        env.project_dir().join("gdvm.toml"),
        "[godot]\nversion = \"4.4-stable\"\n",
    )
    .unwrap();
    fs::write(env.project_dir().join(".gdvmrc"), "4.3.0-stable").unwrap();

    let pinned = mgr.get_pinned_version().expect("a pinned version");
    let gv = pinned.version;
    let variant = pinned.variant;
    assert_eq!(gv.major, Some(4));
    assert_eq!(gv.minor, Some(4), "gdvm.toml (4.4) must win over .gdvmrc");
    assert!(variant.is_none());
}

#[tokio::test]
#[serial]
async fn get_pinned_falls_back_to_legacy_gdvmrc() {
    let env = PinTestEnv::new();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    fs::write(env.project_dir().join(".gdvmrc"), "4.3.0-stable-csharp").unwrap();

    let pinned = mgr.get_pinned_version().expect("a pinned version");
    let gv = pinned.version;
    let variant = pinned.variant;
    assert_eq!(gv.major, Some(4));
    assert_eq!(gv.minor, Some(3));
    assert_eq!(variant.as_deref(), Some("csharp"));
}

#[tokio::test]
#[serial]
async fn get_pinned_walks_up_parent_directories() {
    let env = PinTestEnv::new();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    fs::write(
        env.project_dir().join("gdvm.toml"),
        "[godot]\nversion = \"4.3-stable\"\n",
    )
    .unwrap();

    let nested = env.project_dir().join("a").join("b");
    fs::create_dir_all(&nested).unwrap();
    std::env::set_current_dir(&nested).unwrap();

    let gv = mgr.get_pinned_version().expect("a pinned version").version;
    assert_eq!(gv.major, Some(4));
    assert_eq!(gv.minor, Some(3));
}

#[tokio::test]
#[serial]
async fn pin_then_get_roundtrips_variant() {
    let env = PinTestEnv::new();
    let _ = env.project_dir();
    let i18n = I18n::new().unwrap();
    let mgr = manager(&i18n).await;

    mgr.pin_version(
        &determinate("4.3-stable"),
        &Variant::from_option(Some("csharp")),
        None,
        false,
    )
    .unwrap();

    let pinned = mgr.get_pinned_version().expect("a pinned version");
    let gv = pinned.version;
    let variant = pinned.variant;
    assert_eq!(gv.major, Some(4));
    assert_eq!(gv.minor, Some(3));
    assert_eq!(variant.as_deref(), Some("csharp"));
}
