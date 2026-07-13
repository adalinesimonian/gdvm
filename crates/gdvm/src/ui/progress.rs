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

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use super::theme::theme;
use super::{multi, status_line, with_suspended};
use crate::{fs_utils, t};

const TICK_INTERVAL: Duration = Duration::from_millis(80);
const BAR_WIDTH: usize = 24;
const BAR_CHARS: &str = "█▉▊▋▌▍▎▏ ";
const SPINNER_CHARS: &[&str] = &[
    "○", "◎", "◉", "●", "●", "●", "●", "◉", "◎", "○", "○", "○", "○", "○", " ",
];

/// Get whether `TERM` is set to `dumb` or empty.
fn is_dumb_terminal() -> bool {
    match std::env::var("TERM") {
        Ok(term) => term.trim().is_empty() || term.eq_ignore_ascii_case("dumb"),
        Err(_) => true,
    }
}

/// Get whether an env var should be treated as `true`.
fn env_is_truthy(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| {
        let value = value.trim();
        !value.is_empty()
            && !value.eq_ignore_ascii_case("0")
            && !value.eq_ignore_ascii_case("false")
    })
}

/// Get any override for fancy progress printing from the env.
fn progress_override() -> Option<bool> {
    let value = std::env::var("GDVM_PROGRESS").ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "on" | "always" | "1" | "true" | "yes" => Some(true),
        "off" | "never" | "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

/// Get whether stderr should be treated as an interactive terminal.
fn stderr_is_interactive() -> bool {
    if let Some(forced) = progress_override() {
        return forced;
    }
    if is_dumb_terminal() || env_is_truthy("CI") {
        return false;
    }
    console::user_attended_stderr()
}

/// Format the downloaded byte count.
fn format_size(bytes: u64) -> String {
    let (value, unit) = fs_utils::byte_display_args(bytes);
    t!("size-display", value = value, unit = unit)
}

fn format_fraction(done: u64, total: u64) -> String {
    t!(
        "progress-fraction",
        done = format_size(done),
        total = format_size(total)
    )
}

fn format_rate(bytes_per_sec: u64) -> String {
    let (value, unit) = fs_utils::byte_display_args(bytes_per_sec);
    t!("progress-rate", value = value, unit = unit)
}

/// Format the ETA.
fn format_eta(eta: Duration) -> String {
    let total_secs = eta.as_secs();

    let time = if total_secs >= 3600 {
        t!(
            "progress-eta",
            magnitude = "hours",
            hours = (total_secs / 3600) as i64,
            mins = ((total_secs % 3600) / 60) as i64
        )
    } else if total_secs >= 60 {
        t!(
            "progress-eta",
            magnitude = "minutes",
            mins = (total_secs / 60) as i64,
            secs = (total_secs % 60) as i64
        )
    } else {
        t!(
            "progress-eta",
            magnitude = "seconds",
            secs = total_secs as i64
        )
    };

    t!("progress-eta-remaining", time = time)
}

/// Register template keys for the byte gauge.
fn with_byte_keys(style: ProgressStyle) -> ProgressStyle {
    style
        .with_key(
            "local_bytes",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                let _ = write!(w, "{}", format_size(state.pos()));
            },
        )
        .with_key(
            "local_fraction",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                let _ = write!(
                    w,
                    "{}",
                    format_fraction(state.pos(), state.len().unwrap_or(0))
                );
            },
        )
        .with_key(
            "local_rate",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                let _ = write!(w, "{}", format_rate(state.per_sec() as u64));
            },
        )
        .with_key(
            "local_eta",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                let _ = write!(w, "{}", format_eta(state.eta()));
            },
        )
}

/// A single task in a phase.
pub struct Task {
    /// The progress bar to use for this task.
    bar: ProgressBar,
    /// Whether interactive display is active.
    interactive: bool,
}

impl Task {
    /// Print a status line and start a progress indicator for a task.
    fn start_progress(prefix: String, style: ProgressStyle, total: Option<u64>) -> Self {
        if !stderr_is_interactive() {
            with_suspended(|| eprintln!("{prefix}"));
            return Self {
                bar: ProgressBar::hidden(),
                interactive: false,
            };
        }

        let bar = match total {
            Some(total) => ProgressBar::new(total),
            None => ProgressBar::new_spinner(),
        };
        let bar = multi().add(bar);

        bar.set_style(style);
        bar.set_prefix(prefix);
        bar.enable_steady_tick(TICK_INTERVAL);

        Self {
            bar,
            interactive: true,
        }
    }

    /// Set the absolute position of a byte gauge.
    pub fn set_position(&self, position: u64) {
        self.bar.set_position(position);
    }

    /// Set the total length of a byte gauge.
    pub fn set_length(&self, length: u64) {
        self.bar.set_length(length);
    }

    /// Advance a byte gauge by `delta` bytes.
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Get whether the progress indicator is in interactive mode.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Print a line above the indicator.
    pub fn print_above(&self, message: impl AsRef<str>) {
        let message = message.as_ref().to_string();
        with_suspended(move || eprintln!("{message}"));
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if self.interactive {
            self.bar.finish_and_clear();
            multi().remove(&self.bar);
        }
    }
}

/// Start a task for a phase with a total size that may or may not be known,
/// such as a download.
pub fn transfer(label: impl AsRef<str>, subject: impl AsRef<str>, total: Option<u64>) -> Task {
    let prefix = status_line(&theme().step, label.as_ref(), subject.as_ref());
    let style = match total {
        Some(_) => ProgressStyle::with_template(&format!(
            "{{prefix}}  ▕{{bar:{BAR_WIDTH}.magenta}}▏ {{local_fraction}} │ {{local_rate}} │ {{local_eta}}"
        ))
        .expect("valid template")
        .progress_chars(BAR_CHARS),
        None => {
            ProgressStyle::with_template("{prefix}  {spinner:.magenta} {local_bytes} │ {local_rate}")
                .expect("valid template")
                .tick_strings(SPINNER_CHARS)
        }
    };
    Task::start_progress(prefix, with_byte_keys(style), total)
}

/// Start a task for a phase with a known total size, e.g. extracting ZIPs.
pub fn gauge(label: impl AsRef<str>, subject: impl AsRef<str>, total: u64) -> Task {
    transfer(label, subject, Some(total))
}

/// Start an indeterminate task when the total size is not known.
pub fn activity(label: impl AsRef<str>, subject: impl AsRef<str>) -> Task {
    let prefix = status_line(&theme().step, label.as_ref(), subject.as_ref());
    let style = ProgressStyle::with_template("{prefix}  {spinner:.magenta}")
        .expect("valid template")
        .tick_strings(SPINNER_CHARS);
    Task::start_progress(prefix, style, None)
}
