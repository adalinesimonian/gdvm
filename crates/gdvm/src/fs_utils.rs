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

use anyhow::{Result, anyhow};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Write data to a file atomically by writing to a temp file in the same
/// directory and then renaming it to the target path.
pub fn atomic_write(path: &Path, data: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("Invalid path: {}", path.display()))?;

    fs::create_dir_all(parent)?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(data.as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_parent_and_overwrites() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("nested").join("cache.json");

        atomic_write(&path, "first")?;
        assert_eq!(fs::read_to_string(&path)?, "first");

        atomic_write(&path, "second")?;
        assert_eq!(fs::read_to_string(&path)?, "second");

        Ok(())
    }
}
