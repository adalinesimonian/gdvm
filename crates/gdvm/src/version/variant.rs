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

use std::fmt;

/// A resolved Godot build variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variant(String);

impl Variant {
    /// The default variant name.
    pub const DEFAULT: &str = "default";

    /// Normalize optional user input into a variant. `None` and `"default"`
    /// both resolve to the default variant. Any other name is treated as the
    /// variant name.
    pub fn from_option(input: Option<&str>) -> Self {
        match input {
            Some(v) if v != Self::DEFAULT => Self(v.to_string()),
            _ => Self(Self::DEFAULT.to_string()),
        }
    }

    /// The variant name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True when this is the default variant.
    pub fn is_default(&self) -> bool {
        self.0 == Self::DEFAULT
    }
}

impl Default for Variant {
    fn default() -> Self {
        Self::from_option(None)
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
