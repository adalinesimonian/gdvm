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

use std::sync::OnceLock;

use console::Style;

/// CLI output styles.
pub(super) struct Theme {
    /// Milestone header.
    pub milestone: Style,
    /// Individual step.
    pub step: Style,
    /// Warning label.
    pub warn_label: Style,
    /// Error label.
    pub error_label: Style,
    /// Note label.
    pub note_label: Style,
    /// Tips.
    pub tip: Style,
}

impl Theme {
    fn new() -> Self {
        Self {
            milestone: Style::new().magenta().bold().for_stderr(),
            step: Style::new().cyan().bold().for_stderr(),
            warn_label: Style::new().yellow().bold().for_stderr(),
            error_label: Style::new().red().bold().for_stderr(),
            note_label: Style::new().cyan().bold().for_stderr(),
            tip: Style::new().magenta().bold().for_stderr(),
        }
    }
}

/// Get the shared theme.
pub(super) fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(Theme::new)
}
