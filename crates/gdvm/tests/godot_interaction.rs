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

mod common;

use gdvm::app::find_godot_executable;
use gdvm::zip_utils::extract_zip;
use tempfile::tempdir;

#[test]
fn test_extract_zip_basic() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test.zip");
    common::make_zip(&zip_path, "folder/file.txt", b"hello");

    let out_dir = dir.path().join("out");
    extract_zip(&zip_path, &out_dir).unwrap();
    let extracted = std::fs::read_to_string(out_dir.join("folder/file.txt")).unwrap();
    assert_eq!(extracted, "hello");
}

#[cfg(target_family = "unix")]
#[test]
fn test_extract_zip_strips_special_permission_bits() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    use zip::write::SimpleFileOptions;

    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test.zip");
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().unix_permissions(0o6755);
        zip.start_file("folder/evil", options).unwrap();
        zip.write_all(b"#!/bin/sh\n").unwrap();
        zip.finish().unwrap();
    }

    let out_dir = dir.path().join("out");
    extract_zip(&zip_path, &out_dir).unwrap();

    let mode = std::fs::metadata(out_dir.join("folder/evil"))
        .unwrap()
        .permissions()
        .mode();

    // Setuid, setgid, and sticky bits must not be applied, only the other
    // permission bits.
    assert_eq!(mode & 0o7000, 0);
    assert_eq!(mode & 0o777, 0o755);
}

#[test]
fn test_find_godot_executable() {
    let dir = tempdir().unwrap();
    #[cfg(target_os = "windows")]
    let exe_path = dir.path().join("Godot_v4.0.exe");
    #[cfg(target_os = "linux")]
    let exe_path = dir.path().join("Godot_v4.0");
    #[cfg(target_os = "macos")]
    std::fs::create_dir_all(dir.path().join("Godot.app/Contents/MacOS")).unwrap();
    #[cfg(target_os = "macos")]
    let exe_path = dir.path().join("Godot.app/Contents/MacOS/Godot");
    std::fs::write(&exe_path, "").unwrap();
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&exe_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&exe_path, perms).unwrap();
    }
    let found = find_godot_executable(dir.path(), false).unwrap();
    assert!(found.is_some());
}
