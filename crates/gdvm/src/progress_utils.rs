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

use std::time::Duration;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{fs_utils, t};

/// Format the downloaded byte count.
pub(crate) fn format_progress_size(bytes: u64) -> String {
    let (value, unit) = fs_utils::byte_display_args(bytes);
    t!("size-display", value = value, unit = unit)
}

/// Format the ETA.
pub(crate) fn format_progress_eta(eta: Duration) -> String {
    let total_secs = eta.as_secs();

    if total_secs >= 3600 {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;

        t!(
            "progress-eta",
            magnitude = "hours",
            hours = hours as i64,
            mins = mins as i64
        )
    } else if total_secs >= 60 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;

        t!(
            "progress-eta",
            magnitude = "minutes",
            mins = mins as i64,
            secs = secs as i64
        )
    } else {
        t!(
            "progress-eta",
            magnitude = "seconds",
            secs = total_secs as i64
        )
    }
}

/// Create a spinner displaying the given message.
pub fn spinner(message: String) -> Result<ProgressBar> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(message);
    Ok(pb)
}
