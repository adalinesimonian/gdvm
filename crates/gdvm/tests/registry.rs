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

#![cfg(feature = "integration-tests")]

use gdvm::app::{Gdvm, InstallOutcome};
use gdvm::config::Config;
use gdvm::registry::{self, publish};
use gdvm::version::{Variant, VersionQuery};
use serial_test::serial;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct TestHome {
    home: TempDir,
    prev_home: Option<std::ffi::OsString>,
}

impl TestHome {
    fn new() -> Self {
        let home = TempDir::new().unwrap();
        let prev_home = std::env::var_os("GDVM_TEST_HOME");

        let gdvm_dir = home.path().join(".gdvm");
        fs::create_dir_all(&gdvm_dir).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        fs::write(
            gdvm_dir.join("cache.json"),
            format!(
                r#"{{"gdvm":{{"last_update_check":{now},"new_version":null,"new_major_version":null}},"registries":{{}}}}"#
            ),
        )
        .unwrap();

        unsafe {
            std::env::set_var("GDVM_TEST_HOME", home.path());
        }
        Self { home, prev_home }
    }

    fn path(&self) -> &Path {
        self.home.path()
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        unsafe {
            match &self.prev_home {
                Some(v) => std::env::set_var("GDVM_TEST_HOME", v),
                None => std::env::remove_var("GDVM_TEST_HOME"),
            }
        }
    }
}

/// The `os-arch` platform key for the current host, so the published build matches.
fn host_platform() -> String {
    let host = gdvm::host::detect_host().unwrap();
    format!(
        "{}-{}",
        registry::registry_os_key(host),
        registry::registry_arch_key(host)
    )
}

fn make_zip(path: &Path, entry: &str, contents: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file(entry, zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(contents).unwrap();
    zip.finish().unwrap();
}

/// Build a local registry containing a single stable build for the host
/// platform. Returns the registry directory and its platform key.
fn publish_registry() -> (PathBuf, String) {
    let reg = TempDir::new().unwrap().keep().join("reg");
    publish::init(&reg, Some("local")).unwrap();

    let platform = host_platform();
    let archive_src = reg.parent().unwrap().join("godot.zip");
    make_zip(&archive_src, "Godot.test", b"a real-enough godot archive");

    publish::add_build(
        &reg,
        &publish::AddBuild {
            version: "4.4-stable".to_string(),
            variant: None,
            platform: platform.clone(),
            file: Some(archive_src),
            store: true,
            url: None,
            sha512: None,
            size: None,
        },
    )
    .unwrap();

    (reg, platform)
}

/// Build a registry and add it to the config as `localreg`.
fn publish_local_registry(env: &TestHome) -> (PathBuf, PathBuf, String) {
    let (reg, platform) = publish_registry();

    let mut config = Config::load().unwrap();
    config
        .add_registry("localreg", &format!("file://{}", reg.display()))
        .unwrap();
    config.save().unwrap();

    let stored = reg.join(format!("binaries/4.4-stable/{platform}.zip"));
    let _ = env;
    (reg, stored, platform)
}

/// Restores the working directory when dropped.
struct CwdGuard(Option<PathBuf>);

impl CwdGuard {
    fn enter(dir: &Path) -> Self {
        let prev = std::env::current_dir().ok();
        std::env::set_current_dir(dir).unwrap();
        CwdGuard(prev)
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.0.take() {
            let _ = std::env::set_current_dir(prev);
        }
    }
}

#[tokio::test]
#[serial]
async fn install_from_file_registry_extracts_build() {
    let env = TestHome::new();
    let (reg, _stored, _platform) = publish_local_registry(&env);

    let gdvm = Gdvm::new().await.unwrap();
    let gv = VersionQuery::from_install_str("4.4-stable")
        .unwrap()
        .to_resolved();

    let outcome = gdvm
        .installer()
        .install(&gv, &Variant::default(), Some("localreg"), false, false)
        .await
        .expect("install should succeed from a file: registry");
    assert!(matches!(outcome, InstallOutcome::Installed));

    let store = registry::store_dir_name(&format!("file://{}", reg.display()));
    let extracted = env.path().join(format!(
        ".gdvm/installs/{store}/default/4.4-stable/Godot.test"
    ));
    assert!(
        extracted.is_file(),
        "expected extracted file at {}",
        extracted.display()
    );
}

#[tokio::test]
#[serial]
async fn install_fails_closed_on_sha512_mismatch() {
    let env = TestHome::new();
    let (_reg, stored, _platform) = publish_local_registry(&env);

    let gdvm = Gdvm::new().await.unwrap();
    let gv = VersionQuery::from_install_str("4.4-stable")
        .unwrap()
        .to_resolved();

    gdvm.installer()
        .install(&gv, &Variant::default(), Some("localreg"), false, false)
        .await
        .expect("initial install should succeed");

    fs::write(&stored, b"tampered contents that do not match the hash").unwrap();

    let result = gdvm
        .installer()
        .install(&gv, &Variant::default(), Some("localreg"), true, true)
        .await;
    assert!(
        result.is_err(),
        "install must fail when the artifact does not match its declared sha512"
    );
}

#[tokio::test]
#[serial]
async fn project_gdvm_toml_registry_is_honored_over_machine() {
    let env = TestHome::new();
    let (reg, _platform) = publish_registry();

    let mut config = Config::load().unwrap();
    config
        .add_registry("proj", "file:///gdvm/does-not-exist")
        .unwrap();
    config.save().unwrap();

    let project_url = format!("file://{}", reg.to_string_lossy().replace('\\', "/"));
    let project = TempDir::new().unwrap();
    fs::write(
        project.path().join("gdvm.toml"),
        format!(
            "[godot]\nversion = \"proj/4.4-stable\"\n\n[registries.proj]\nurl = \"{project_url}\"\n",
        ),
    )
    .unwrap();

    let installed = {
        let _cwd = CwdGuard::enter(project.path());
        let gdvm = Gdvm::new().await.unwrap();

        assert_eq!(
            gdvm.catalogs().registry_base_url("proj").unwrap(),
            project_url
        );

        let gv = VersionQuery::from_install_str("4.4-stable")
            .unwrap()
            .to_resolved();
        let outcome = gdvm
            .installer()
            .install(&gv, &Variant::default(), Some("proj"), false, false)
            .await
            .expect("install from project-defined registry should succeed");
        matches!(outcome, InstallOutcome::Installed)
    };

    assert!(installed);
    let store = registry::store_dir_name(&project_url);
    assert!(
        env.path()
            .join(format!(".gdvm/installs/{store}/default/4.4-stable"))
            .is_dir(),
        "build should be installed under the store path"
    );
}
