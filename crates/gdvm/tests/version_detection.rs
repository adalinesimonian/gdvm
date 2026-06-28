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

use gdvm::{
    i18n::I18n,
    project_version_detector::{detect_godot_version_in_path, find_project_file},
};
use tempfile::tempdir;

#[test]
fn test_find_project_file() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let proj = dir.path().join("project.godot");
    std::fs::write(&proj, "").unwrap();
    let found = find_project_file(&sub).unwrap();
    assert_eq!(found, proj);
}

#[test]
fn test_detect_godot_version_in_path() {
    let dir = tempdir().unwrap();
    let proj = dir.path().join("project.godot");
    std::fs::write(
        &proj,
        r#"
[application]
config/features=PackedStringArray("4.2")

[other]
foo=bar
"#,
    )
    .unwrap();
    let i18n = I18n::new().unwrap();
    let (gv, _variant) = detect_godot_version_in_path(&i18n, &proj).unwrap();
    assert_eq!(gv.major, Some(4));
    assert_eq!(gv.minor, Some(2));
}
