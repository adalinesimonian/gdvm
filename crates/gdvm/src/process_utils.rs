// SPDX-FileCopyrightText: Copyright (C) 2026 Adaline Simonian
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

use std::io;
use std::process::{Command, ExitStatus, Stdio};

/// Get the exit code of a child process.
pub fn child_exit_code(status: ExitStatus) -> i32 {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::process::ExitStatusExt;

        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }

    status.code().unwrap_or(1)
}

/// Spawn `command` detached from the current process.
pub fn spawn_detached(command: &mut Command) -> io::Result<()> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_family = "windows")]
    {
        use std::os::windows::process::CommandExt;

        use winapi::um::winbase::DETACHED_PROCESS;

        command.creation_flags(DETACHED_PROCESS);
    }

    command.spawn()?;

    Ok(())
}
