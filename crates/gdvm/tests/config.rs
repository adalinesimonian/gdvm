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

use gdvm::{config::Config, i18n::I18n};
use serial_test::serial;
use tempfile::tempdir;

fn with_test_home<F, R>(path: &std::path::Path, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = std::env::var("GDVM_TEST_HOME").ok();

    unsafe {
        std::env::set_var("GDVM_TEST_HOME", path);
    }

    let result = f();

    if let Some(val) = previous {
        unsafe {
            std::env::set_var("GDVM_TEST_HOME", val);
        }
    } else {
        unsafe {
            std::env::remove_var("GDVM_TEST_HOME");
        }
    }

    result
}

#[test]
#[serial]
fn test_load_save_roundtrip() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new(80).unwrap();

    with_test_home(dir.path(), || {
        let cfg = Config {
            github_token: Some("token1".into()),
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded = with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded.github_token, Some("token1".to_string()));

    with_test_home(dir.path(), || {
        let cfg = Config {
            github_token: Some("token2".into()),
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded2 = with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded2.github_token, Some("token2".to_string()));
}
