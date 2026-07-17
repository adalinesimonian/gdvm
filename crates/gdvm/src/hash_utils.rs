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

use std::io::Read;
use std::path::Path;

use anyhow::Result;
use digest_io::IoWrapper;
use sha2::{Digest, Sha256, Sha512};

use crate::terr;

/// Hex encode a byte slice.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug, Clone, Copy)]
pub enum ShaType {
    Sha256,
    Sha512,
}

impl ShaType {
    /// Detect the SHA type based on the hash length.
    pub fn from_hash_length(hash: &str) -> Option<Self> {
        match hash.len() {
            64 => Some(ShaType::Sha256),
            128 => Some(ShaType::Sha512),
            _ => None,
        }
    }

    /// Detect the SHA type based on the hash length, returning an error if the
    /// length is invalid.
    pub fn from_expected(expected: &str) -> Result<Self> {
        Self::from_hash_length(expected)
            .ok_or_else(|| terr!("error-invalid-sha-length", length = expected.len()))
    }
}

/// Hash the contents of a reader with the given SHA type.
pub fn hash_reader<R: Read>(sha_type: ShaType, reader: &mut R) -> Result<String> {
    Ok(match sha_type {
        ShaType::Sha256 => {
            let mut hasher = IoWrapper(Sha256::new());
            std::io::copy(reader, &mut hasher)?;
            to_hex(&hasher.0.finalize())
        }
        ShaType::Sha512 => {
            let mut hasher = IoWrapper(Sha512::new());
            std::io::copy(reader, &mut hasher)?;
            to_hex(&hasher.0.finalize())
        }
    })
}

pub(crate) fn checksum_mismatch_error(display_path: &Path) -> anyhow::Error {
    terr!(
        "error-checksum-mismatch",
        file = display_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    )
}
