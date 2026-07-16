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

use anyhow::Result;

use crate::terr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOs {
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostArch {
    X86_64,
    X86,
    Aarch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostPlatform {
    pub os: HostOs,
    pub arch: HostArch,
}

impl HostPlatform {
    pub fn gdvm_target_triple(self) -> Result<&'static str> {
        match (self.os, self.arch) {
            (HostOs::Windows, HostArch::Aarch64) => Ok("aarch64-pc-windows-msvc"),
            (HostOs::Windows, HostArch::X86_64) => Ok("x86_64-pc-windows-msvc"),
            (HostOs::Windows, HostArch::X86) => Ok("i686-pc-windows-msvc"),
            (HostOs::Linux, HostArch::Aarch64) => Ok("aarch64-unknown-linux-gnu"),
            (HostOs::Linux, HostArch::X86_64) => Ok("x86_64-unknown-linux-gnu"),
            (HostOs::Linux, HostArch::X86) => Ok("i686-unknown-linux-gnu"),
            (HostOs::Macos, HostArch::Aarch64) => Ok("aarch64-apple-darwin"),
            (HostOs::Macos, HostArch::X86_64) => Ok("x86_64-apple-darwin"),
            _ => Err(terr!("unsupported-architecture")),
        }
    }
}

pub fn detect_host() -> Result<HostPlatform> {
    let os = if cfg!(target_os = "windows") {
        HostOs::Windows
    } else if cfg!(target_os = "macos") {
        HostOs::Macos
    } else if cfg!(target_os = "linux") {
        HostOs::Linux
    } else {
        return Err(terr!("unsupported-platform"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        HostArch::X86_64
    } else if cfg!(target_arch = "x86") {
        HostArch::X86
    } else if cfg!(target_arch = "aarch64") {
        HostArch::Aarch64
    } else {
        return Err(terr!("unsupported-architecture"));
    };

    Ok(HostPlatform { os, arch })
}
