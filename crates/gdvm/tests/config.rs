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

use gdvm::config::{ConfigFile, ConfigFileState, ConfigKey};
use serial_test::serial;

mod common;
use common::TestHome;

#[test]
#[serial]
fn test_load_save_roundtrip() {
    let home = TestHome::new();

    ConfigFile::modify(|file| file.set_value(ConfigKey::PruneMaxAgeDays, "7")).unwrap();

    let loaded = ConfigFile::load().unwrap().into_config();
    assert_eq!(loaded.prune.max_age_days, Some(7));

    ConfigFile::modify(|file| file.set_value(ConfigKey::PruneMaxAgeDays, "14")).unwrap();

    let loaded2 = ConfigFile::load().unwrap().into_config();
    assert_eq!(loaded2.prune.max_age_days, Some(14));

    drop(home);
}

#[test]
#[serial]
fn test_legacy_config_option_is_ignored_on_load() {
    let home = TestHome::new();

    let config_dir = home.gdvm_dir();
    std::fs::write(
        config_dir.join("config.toml"),
        "github_token = \"secret\"\n[prune]\nmax-age-days = 9\n",
    )
    .unwrap();

    let loaded = ConfigFile::load().unwrap().into_config();
    assert_eq!(loaded.prune.max_age_days, Some(9));
}

#[test]
#[serial]
fn test_malformed_config_is_not_overwritten() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");
    let original = "this is not valid toml =====\n[prune]\nmax-age-days = 7\n";

    std::fs::write(&path, original).unwrap();

    let file = ConfigFile::load().unwrap();

    assert_eq!(file.state(), ConfigFileState::Malformed);
    assert!(file.save().is_err());
    assert!(ConfigFile::modify(|file| file.set_value(ConfigKey::PruneMaxAgeDays, "3")).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
}

#[test]
#[serial]
fn test_unusable_value_is_reported_and_left_alone() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    std::fs::write(
        &path,
        "trusted-registries = [\"https://example.com/reg\"]\n[prune]\nmax-age-days = \"soon\"\n",
    )
    .unwrap();

    let file = ConfigFile::load().unwrap();

    assert_eq!(file.state(), ConfigFileState::Usable);
    assert_eq!(file.problems().len(), 1);
    assert_eq!(file.problems()[0].key, "prune.max-age-days");
    assert!(file.config().prune.max_age_days.is_none());
    assert_eq!(
        file.config().trusted_registries,
        vec!["https://example.com/reg"]
    );

    ConfigFile::modify(|file| file.add_registry("mybuilds", "https://example.com/godot")).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(contents.contains("max-age-days = \"soon\""));
    assert!(contents.contains("mybuilds"));
}

#[test]
#[serial]
fn test_setting_a_key_replaces_an_unusable_value() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    std::fs::write(&path, "[prune]\nmax-age-days = \"soon\"\n").unwrap();

    ConfigFile::modify(|file| file.set_value(ConfigKey::PruneMaxAgeDays, "5")).unwrap();
    assert_eq!(
        ConfigFile::load().unwrap().config().prune.max_age_days,
        Some(5)
    );

    std::fs::write(&path, "[prune]\nmax-age-days = \"soon\"\n").unwrap();
    ConfigFile::modify(|file| {
        file.unset_value(ConfigKey::PruneMaxAgeDays);
        Ok(())
    })
    .unwrap();
    assert!(!std::fs::read_to_string(&path).unwrap().contains("soon"));
}

#[test]
#[serial]
fn test_unrecognized_keys_survive_a_save() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    std::fs::write(&path, "some_future_key = 42\n").unwrap();

    ConfigFile::modify(|file| file.set_value(ConfigKey::PruneMaxAgeDays, "5")).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(contents.contains("some_future_key = 42"));
    assert!(contents.contains("max-age-days = 5"));
}

#[test]
#[serial]
fn test_unreadable_config_is_not_overwritten() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    std::fs::create_dir_all(&path).unwrap();

    let file = ConfigFile::load().unwrap();

    assert_eq!(file.state(), ConfigFileState::Unreadable);
    assert!(file.save().is_err());
    assert!(path.is_dir());
}

#[test]
#[serial]
fn test_comments_and_layout_survive_a_save() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    std::fs::write(
        &path,
        "# My employer's registry.\n[registries.work]\nurl = \"https://example.com/godot\"\n\n\
         [prune]\n# Keep old builds around for a year.\nmax-age-days = 365\n",
    )
    .unwrap();

    ConfigFile::modify(|file| file.set_value(ConfigKey::PruneMaxAgeDays, "30")).unwrap();
    ConfigFile::modify(|file| file.add_registry("other", "https://example.org/reg")).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(contents.contains("# Keep old builds around for a year."));
    assert!(contents.contains("# My employer's registry."));
    assert!(contents.contains("max-age-days = 30"));
    assert!(contents.contains("[registries.work]"));
    assert!(contents.contains("[registries.other]"));
}

#[test]
#[serial]
fn test_removing_a_registry_leaves_the_others_alone() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    std::fs::write(
        &path,
        "# top\ntrusted-registries = []\n\n[registries.work]\nurl = \"https://example.com/godot\"\n\n\
         [registries.other]\nurl = \"https://example.org/reg\"\n\n[prune]\nmax-age-days = 365\n",
    )
    .unwrap();

    ConfigFile::modify(|file| file.remove_registry("work")).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(contents.contains("# top"));
    assert!(!contents.contains("[registries.work]"));
    assert!(contents.contains("[registries.other]"));

    ConfigFile::modify(|file| file.remove_registry("other")).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(!contents.contains("[registries"));
    assert!(contents.contains("trusted-registries = []"));
    assert!(contents.contains("max-age-days = 365"));
}

#[test]
#[serial]
fn test_writing_to_a_file_that_does_not_exist_yet() {
    let home = TestHome::new();
    let path = home.path().join(".gdvm").join("config.toml");

    assert!(!path.exists());

    ConfigFile::modify(|file| {
        file.set_value(ConfigKey::PruneMaxAgeDays, "7")?;
        file.add_registry("work", "https://example.com/godot")?;
        file.trust_registry("https://example.com/godot");
        Ok(())
    })
    .unwrap();

    let loaded = ConfigFile::load().unwrap().into_config();

    assert_eq!(loaded.prune.max_age_days, Some(7));
    assert_eq!(
        loaded.registry_url("work"),
        Some("https://example.com/godot")
    );
    assert!(loaded.is_registry_trusted("https://example.com/godot"));
    assert!(path.exists());
}

#[test]
#[serial]
fn test_loading_from_an_explicit_path() {
    let home = TestHome::new();
    let elsewhere = home.path().join("elsewhere.toml");

    std::fs::write(&elsewhere, "[prune]\nmax-age-days = 3\n").unwrap();

    let mut file = ConfigFile::load_from(&elsewhere).unwrap();

    assert_eq!(file.config().prune.max_age_days, Some(3));

    file.set_value(ConfigKey::PruneMaxAgeDays, "4").unwrap();
    file.save().unwrap();

    assert!(
        std::fs::read_to_string(&elsewhere)
            .unwrap()
            .contains("max-age-days = 4")
    );
    assert!(!home.path().join(".gdvm").join("config.toml").exists());
}
