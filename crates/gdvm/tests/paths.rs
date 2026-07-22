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

use gdvm::config::{Config, ConfigOps};
use gdvm::paths::GdvmPaths;
use serial_test::serial;

mod common;
use common::TestHome;

#[test]
#[serial]
fn test_gdvm_paths_uses_normalized_absolute_paths() {
    let _home = TestHome::new();

    let mut cfg = Config::default();
    cfg.set_value("install.path", "./custom-installs").unwrap();
    cfg.set_value("cache.path", "./custom-cache").unwrap();
    cfg.save().unwrap();

    let paths = GdvmPaths::new(&cfg).unwrap();

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
}
