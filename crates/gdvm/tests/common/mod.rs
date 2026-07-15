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