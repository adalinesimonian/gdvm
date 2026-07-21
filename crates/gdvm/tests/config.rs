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

use gdvm::{config::Config, config::ConfigOps, i18n::I18n};
use serial_test::serial;

mod common;
use common::TestHome;

#[test]
#[serial]
fn test_load_save_roundtrip() {
    let home = TestHome::new();

    let cfg = Config {
        prune_max_age_days: Some(7),
        ..Default::default()
    };
    cfg.save().unwrap();

    let loaded = Config::load().unwrap();
    assert_eq!(loaded.prune_max_age_days, Some(7));

    let cfg = Config {
        prune_max_age_days: Some(14),
        ..Default::default()
    };
    cfg.save().unwrap();

    let loaded2 = Config::load().unwrap();
    assert_eq!(loaded2.prune_max_age_days, Some(14));

    drop(home);
}

#[test]
#[serial]
fn test_legacy_config_option_is_ignored_on_load() {
    let home = TestHome::new();

    let config_dir = home.gdvm_dir();
    std::fs::write(
        config_dir.join("config.toml"),
        "github_token = \"secret\"\nprune_max_age_days = 9\n",
    )
    .unwrap();

    let loaded = Config::load().unwrap();
    assert_eq!(loaded.prune_max_age_days, Some(9));
}

#[test]
#[serial]
fn test_change_install_path_config() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new().unwrap();

    common::with_test_home(dir.path(), || {
        let cfg = Config {
            install_path: Some(dir.path().join("test_installs")),
            ..Default::default()
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded = common::with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded.install_path, Some(dir.path().join("test_installs")));
}

#[test]
#[serial]
fn test_change_cache_path_config() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new().unwrap();

    common::with_test_home(dir.path(), || {
        let cfg = Config {
            cache_path: Some(dir.path().join("test_cache")),
            ..Default::default()
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded = common::with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded.cache_path, Some(dir.path().join("test_cache")));
}

#[test]
#[serial]
fn test_config_rejects_reserved_install_path() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new().unwrap();

    common::with_test_home(dir.path(), || {
        let mut cfg = Config::default();
        let reserved = dir.path().join(".gdvm").join("bin");

        assert!(
            cfg.set_value("install.path", reserved.to_string_lossy().as_ref())
                .is_err()
        );
        cfg.save(&i18n).unwrap();
    });
}

#[test]
#[serial]
fn test_config_rejects_overlapping_install_and_cache_paths() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new().unwrap();

    common::with_test_home(dir.path(), || {
        let mut cfg = Config::default();
        let installs = dir.path().join("custom-installs");
        let cache = installs.join("cache");

        cfg.set_value("install.path", installs.to_string_lossy().as_ref())
            .unwrap();
        assert!(
            cfg.set_value("cache.path", cache.to_string_lossy().as_ref())
                .is_err()
        );
        cfg.save(&i18n).unwrap();
    });
}
