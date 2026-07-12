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

use gdvm::app::{Gdvm, PruneOptions};
use gdvm::usage_tracker::{ArchiveUsage, InstallUsage, LinkRecord, UsageState, UsageTracker};
use gdvm::version::{Variant, VersionQuery};
use serial_test::serial;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const DAY: u64 = 24 * 60 * 60;

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
        fs::write(
            gdvm_dir.join("cache.json"),
            format!(
                r#"{{"gdvm":{{"last_update_check":{},"new_version":null,"new_major_version":null}},"registries":{{}}}}"#,
                now_secs()
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

    fn installs(&self) -> PathBuf {
        self.path().join(".gdvm").join("installs")
    }

    fn cache(&self) -> PathBuf {
        self.path().join(".gdvm").join("cache")
    }

    fn usage_path(&self) -> PathBuf {
        self.path().join(".gdvm").join("usage.json")
    }

    fn locks_path(&self) -> PathBuf {
        self.path().join(".gdvm").join("locks")
    }

    /// Create an install at `key` with a single regular file inside. Returns
    /// the path to the install directory.
    fn make_install(&self, key: &str) -> PathBuf {
        let dir = self.installs().join(key);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Godot"), b"fake-binary").unwrap();
        dir
    }

    /// Create a cached archive file with the given name and contents.
    fn make_cache_file(&self, name: &str, contents: &[u8]) -> PathBuf {
        let dir = self.cache();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    fn write_usage(&self, state: &UsageState) {
        UsageTracker::new(self.usage_path(), self.locks_path())
            .save(state)
            .unwrap();
    }

    fn read_usage(&self) -> UsageState {
        UsageTracker::new(self.usage_path(), self.locks_path())
            .load()
            .unwrap()
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

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn determinate(install_str: &str) -> gdvm::version::ResolvedVersion {
    VersionQuery::from_install_str(install_str)
        .unwrap()
        .to_resolved()
}

async fn gdvm() -> Gdvm {
    Gdvm::new().await.unwrap()
}

/// Create a symlink at `link` pointing at `target`.
fn make_symlink(target: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(target, link).unwrap();
}

fn state_with(
    installs: &[(&str, u64)],
    archives: &[(&str, u64)],
    links: &[(&str, &str, u64)],
) -> UsageState {
    UsageState {
        installs: installs
            .iter()
            .map(|(k, t)| (k.to_string(), InstallUsage { last_used: *t }))
            .collect::<HashMap<_, _>>(),
        archives: archives
            .iter()
            .map(|(k, t)| (k.to_string(), ArchiveUsage { last_used: *t }))
            .collect::<HashMap<_, _>>(),
        links: links
            .iter()
            .map(|(path, key, t)| {
                (
                    path.to_string(),
                    LinkRecord {
                        install_key: key.to_string(),
                        last_used: *t,
                    },
                )
            })
            .collect::<HashMap<_, _>>(),
        ..UsageState::default()
    }
}

#[tokio::test]
#[serial]
async fn prune_removes_stale_keeps_recent() {
    let env = TestHome::new();

    let stale_key = "store/default/4.3-stable";
    let fresh_key = "store/default/4.4-stable";
    let stale = env.make_install(stale_key);
    let fresh = env.make_install(fresh_key);
    env.make_cache_file("stalearchive.zip", b"old");
    env.make_cache_file("fresharchive.zip", b"new");

    let now = now_secs();
    env.write_usage(&state_with(
        &[(stale_key, now - 40 * DAY), (fresh_key, now - DAY)],
        &[
            ("stalearchive.zip", now - 40 * DAY),
            ("fresharchive.zip", now - DAY),
        ],
        &[],
    ));

    let mgr = gdvm().await;
    let report = mgr
        .pruner()
        .prune(30 * DAY, PruneOptions::default())
        .unwrap();

    assert!(!stale.exists(), "stale install should be removed");
    assert!(fresh.exists(), "fresh install should be kept");
    assert!(
        !env.cache().join("stalearchive.zip").exists(),
        "stale archive should be removed"
    );
    assert!(
        env.cache().join("fresharchive.zip").exists(),
        "fresh archive should be kept"
    );

    assert_eq!(report.installs.len(), 1);
    assert_eq!(report.archives.len(), 1);
    assert!(!report.dry_run);

    let usage = env.read_usage();
    assert!(!usage.installs.contains_key(stale_key));
    assert!(usage.installs.contains_key(fresh_key));
    assert!(!usage.archives.contains_key("stalearchive.zip"));
    assert!(usage.archives.contains_key("fresharchive.zip"));
}

#[tokio::test]
#[serial]
async fn prune_keeps_freshly_created_untracked_install() {
    let env = TestHome::new();

    let key = "store/default/4.3-stable";
    let dir = env.make_install(key);
    env.write_usage(&UsageState::default());

    let mgr = gdvm().await;
    let report = mgr
        .pruner()
        .prune(30 * DAY, PruneOptions::default())
        .unwrap();

    assert!(dir.exists(), "recently created install should be kept");
    assert!(report.is_empty());
}

#[tokio::test]
#[serial]
async fn prune_all_preserves_link_referenced_install() {
    let env = TestHome::new();

    let linked_key = "store/default/4.3-stable";
    let other_key = "store/default/4.4-stable";
    let linked = env.make_install(linked_key);
    let other = env.make_install(other_key);

    let link = env.path().join("godot-link");
    make_symlink(&linked.join("Godot"), &link);
    let link_str = link.to_string_lossy().to_string();

    let now = now_secs();
    env.write_usage(&state_with(
        &[(linked_key, now - DAY), (other_key, now - DAY)],
        &[],
        &[(&link_str, linked_key, now - DAY)],
    ));

    let mgr = gdvm().await;
    let report = mgr
        .pruner()
        .prune(
            30 * DAY,
            PruneOptions {
                all: true,
                force: false,
                dry_run: false,
            },
        )
        .unwrap();

    assert!(linked.exists(), "link-referenced install must be preserved");
    assert!(!other.exists(), "unreferenced install must be removed");
    assert_eq!(report.preserved_by_link, 1);
}

#[tokio::test]
#[serial]
async fn prune_force_removes_link_referenced_install() {
    let env = TestHome::new();

    let key = "store/default/4.3-stable";
    let dir = env.make_install(key);
    let link = env.path().join("godot-link");
    make_symlink(&dir.join("Godot"), &link);
    let link_str = link.to_string_lossy().to_string();

    let now = now_secs();
    env.write_usage(&state_with(
        &[(key, now - 40 * DAY)],
        &[],
        &[(&link_str, key, now - 40 * DAY)],
    ));

    let mgr = gdvm().await;

    let report = mgr
        .pruner()
        .prune(30 * DAY, PruneOptions::default())
        .unwrap();
    assert!(dir.exists(), "link should protect the install by default");
    assert_eq!(report.preserved_by_link, 1);

    let report = mgr
        .pruner()
        .prune(
            30 * DAY,
            PruneOptions {
                all: false,
                force: true,
                dry_run: false,
            },
        )
        .unwrap();
    assert!(
        !dir.exists(),
        "force must ignore links and remove the install"
    );
    assert_eq!(report.installs.len(), 1);

    let usage = env.read_usage();
    assert!(
        usage.links.is_empty(),
        "dangling link record must be pruned"
    );
    assert!(!usage.installs.contains_key(key));
}

#[tokio::test]
#[serial]
async fn prune_dry_run_changes_nothing() {
    let env = TestHome::new();

    let key = "store/default/4.3-stable";
    let dir = env.make_install(key);
    env.make_cache_file("stalearchive.zip", b"old");

    let now = now_secs();
    env.write_usage(&state_with(
        &[(key, now - 40 * DAY)],
        &[("stalearchive.zip", now - 40 * DAY)],
        &[],
    ));

    let mgr = gdvm().await;
    let report = mgr
        .pruner()
        .prune(
            30 * DAY,
            PruneOptions {
                all: false,
                force: false,
                dry_run: true,
            },
        )
        .unwrap();

    assert!(report.dry_run);
    assert!(!report.is_empty(), "dry run should still report candidates");
    assert!(dir.exists(), "dry run must not delete installs");
    assert!(
        env.cache().join("stalearchive.zip").exists(),
        "dry run must not delete archives"
    );

    let usage = env.read_usage();
    assert!(usage.installs.contains_key(key));
    assert!(usage.archives.contains_key("stalearchive.zip"));
}

#[tokio::test]
#[serial]
async fn prune_all_force_removes_everything() {
    let env = TestHome::new();

    let key = "store/default/4.3-stable";
    let dir = env.make_install(key);
    let link = env.path().join("godot-link");
    make_symlink(&dir.join("Godot"), &link);
    let link_str = link.to_string_lossy().to_string();
    env.make_cache_file("archive.zip", b"data");

    let now = now_secs();
    env.write_usage(&state_with(
        &[(key, now)],
        &[("archive.zip", now)],
        &[(&link_str, key, now)],
    ));

    let mgr = gdvm().await;
    let report = mgr
        .pruner()
        .prune(
            30 * DAY,
            PruneOptions {
                all: true,
                force: true,
                dry_run: false,
            },
        )
        .unwrap();

    assert!(!dir.exists(), "all+force must remove even linked installs");
    assert!(!env.cache().join("archive.zip").exists());
    assert_eq!(report.preserved_by_link, 0);

    let usage = env.read_usage();
    assert!(usage.installs.is_empty());
    assert!(usage.archives.is_empty());
    assert!(usage.links.is_empty());
}

#[tokio::test]
#[serial]
async fn prune_never_removes_the_default_install() {
    let env = TestHome::new();

    let mgr = gdvm().await;

    let default_gv = determinate("4.3-stable");
    let other_gv = determinate("4.4-stable");
    let default_key = mgr
        .library()
        .install_key(&default_gv, &Variant::default(), None)
        .unwrap();
    let other_key = mgr
        .library()
        .install_key(&other_gv, &Variant::default(), None)
        .unwrap();

    let default_dir = env.make_install(&default_key);
    let other_dir = env.make_install(&other_key);

    mgr.defaults()
        .set_default(&default_gv, &Variant::default(), None)
        .unwrap();

    env.make_cache_file("stalearchive.zip", b"old");

    let now = now_secs();
    env.write_usage(&state_with(
        &[
            (default_key.as_str(), now - 400 * DAY),
            (other_key.as_str(), now - 400 * DAY),
        ],
        &[("stalearchive.zip", now - 400 * DAY)],
        &[],
    ));

    let report = mgr
        .pruner()
        .prune(
            0,
            PruneOptions {
                all: true,
                force: true,
                dry_run: false,
            },
        )
        .unwrap();

    assert!(
        default_dir.exists(),
        "the default install must never be pruned, even with --all --force"
    );
    assert!(
        !other_dir.exists(),
        "the non-default install should be removed"
    );
    assert!(
        !env.cache().join("stalearchive.zip").exists(),
        "an unreferenced archive should still be pruned even when a default exists"
    );
    assert!(
        !report.installs.iter().any(|i| i.label.contains("4.3")),
        "default 4.3 must not appear in the removed list"
    );
    let usage = env.read_usage();
    assert!(
        usage.installs.contains_key(&default_key),
        "default install usage record must be retained"
    );
}
