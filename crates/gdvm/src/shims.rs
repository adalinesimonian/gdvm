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

use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;

use crate::eprintln_i18n;

#[cfg(target_os = "windows")]
const GDVM_SHIM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shim.exe"));
#[cfg(not(target_os = "windows"))]
const GDVM_SHIM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shim"));

/// Write the shims to disk.
pub fn ensure(base_path: &Path) -> Result<()> {
    let bin_path = base_path.join("bin");

    fs::create_dir_all(&bin_path)?;

    let targets: &[&str] = if cfg!(target_os = "windows") {
        &["godot.exe", "godot_console.exe"]
    } else {
        &["godot"]
    };

    for exe in targets {
        let exe_path = bin_path.join(exe);
        if let Err(err) = write_bytes_if_different(GDVM_SHIM, &exe_path, Some(0o755)) {
            eprintln_i18n!(
                "error-ensure-godot-binaries-failed",
                error = &err.to_string(),
                path = &exe_path.to_string_lossy().to_string(),
            );
            return Err(err);
        }
    }

    Ok(())
}

/// Write `bytes` to `dest` only if the current contents at `dest` are different.
fn write_bytes_if_different(bytes: &[u8], dest: &Path, perm: Option<u32>) -> Result<()> {
    #[cfg(not(target_family = "unix"))]
    let _ = perm;

    let write_bytes = || -> Result<()> {
        fs::write(dest, bytes)?;

        #[cfg(target_family = "unix")]
        if let Some(mode) = perm {
            fs::set_permissions(dest, fs::Permissions::from_mode(mode))?;
        }

        Ok(())
    };

    if let Ok(metadata) = fs::metadata(dest) {
        if metadata.len() != bytes.len() as u64 {
            write_bytes()?;
        }
    } else {
        write_bytes()?;
    }

    let file = fs::File::open(dest)?;
    let mut reader = std::io::BufReader::new(file);
    let mut idx = 0;
    let mut buffer = [0u8; 8192];

    loop {
        let read = std::io::Read::read(&mut reader, &mut buffer)?;

        if read == 0 {
            break;
        }

        if buffer[..read] != bytes[idx..idx + read] {
            write_bytes()?;
            return Ok(());
        }

        idx += read;
    }

    Ok(())
}
