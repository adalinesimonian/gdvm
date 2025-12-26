use anyhow::{Result, anyhow};

use crate::{i18n::I18n, t_w};

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
    pub fn gdvm_target_triple(self, i18n: &I18n) -> Result<&'static str> {
        match (self.os, self.arch) {
            (HostOs::Windows, HostArch::Aarch64) => Ok("aarch64-pc-windows-msvc"),
            (HostOs::Windows, HostArch::X86_64) => Ok("x86_64-pc-windows-msvc"),
            (HostOs::Windows, HostArch::X86) => Ok("i686-pc-windows-msvc"),
            (HostOs::Linux, HostArch::Aarch64) => Ok("aarch64-unknown-linux-gnu"),
            (HostOs::Linux, HostArch::X86_64) => Ok("x86_64-unknown-linux-gnu"),
            (HostOs::Linux, HostArch::X86) => Ok("i686-unknown-linux-gnu"),
            (HostOs::Macos, HostArch::Aarch64) => Ok("aarch64-apple-darwin"),
            (HostOs::Macos, HostArch::X86_64) => Ok("x86_64-apple-darwin"),
            _ => Err(anyhow!(t_w!(i18n, "unsupported-architecture"))),
        }
    }
}

pub fn detect_host(i18n: &I18n) -> Result<HostPlatform> {
    let os = if cfg!(target_os = "windows") {
        HostOs::Windows
    } else if cfg!(target_os = "macos") {
        HostOs::Macos
    } else if cfg!(target_os = "linux") {
        HostOs::Linux
    } else {
        return Err(anyhow!(t_w!(i18n, "unsupported-platform")));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        HostArch::X86_64
    } else if cfg!(target_arch = "x86") {
        HostArch::X86
    } else if cfg!(target_arch = "aarch64") {
        HostArch::Aarch64
    } else {
        return Err(anyhow!(t_w!(i18n, "unsupported-architecture")));
    };

    Ok(HostPlatform { os, arch })
}
