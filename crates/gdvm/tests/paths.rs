// SPDX-FileCopyrightText: Copyright (C) 2026 UtileRain
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

use gdvm::{
    config::{Config, ConfigOps, get_absolute_path_to_directory},
    i18n::I18n,
    paths::GdvmPaths,
};
use serial_test::serial;
use std::path::Path;
use tempfile::tempdir;
mod common;

#[test]
fn test_get_absolute_path_normalizes_relative_paths() {
    let path = get_absolute_path_to_directory("subdir/../installs").unwrap();

    assert!(path.is_absolute());
    assert!(
        !path
            .components()
            .any(|component| component == std::path::Component::ParentDir)
    );
    assert_eq!(path, std::env::current_dir().unwrap().join("installs"));
}

#[test]
fn test_get_absolute_path_rejects_empty_strings() {
    assert!(get_absolute_path_to_directory("").is_err());
    assert!(get_absolute_path_to_directory("   ").is_err());
}

#[test]
fn test_get_absolute_path_rejects_existing_files() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("not_a_dir.txt");
    std::fs::write(&file_path, "data").unwrap();

    assert!(get_absolute_path_to_directory(file_path.to_string_lossy().as_ref()).is_err());
}

#[test]
#[serial]
fn test_gdvm_paths_uses_normalized_absolute_paths() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new().unwrap();

    common::with_test_home(dir.path(), || {
        let mut cfg = Config::default();
        cfg.set_value("install.path", "./custom-installs").unwrap();
        cfg.set_value("cache.path", "./custom-cache").unwrap();
        cfg.save(&i18n).unwrap();

        let paths = GdvmPaths::new(&i18n).unwrap();

        assert!(paths.installs().is_absolute());
        assert!(paths.cache_dir().is_absolute());
        assert!(
            !paths
                .installs()
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        );
        assert!(
            !paths
                .cache_dir()
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        );
    });
}
