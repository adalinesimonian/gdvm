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

pub mod progress;
mod theme;

use std::sync::OnceLock;

use console::Style;
use indicatif::MultiProgress;
use theme::theme;

use crate::t;

include!(concat!(env!("OUT_DIR"), "/status_label_widths.rs"));

/// Get the width of the longest status label for the current locale.
fn status_label_width() -> usize {
    let locale = crate::i18n::current_locale();
    STATUS_LABEL_WIDTHS
        .iter()
        .find(|(maybe, _)| *maybe == locale)
        .map_or(STATUS_LABEL_WIDTH_FALLBACK, |(_, width)| *width)
}

struct Console {
    multi: MultiProgress,
}

/// Get the global console.
fn console() -> &'static Console {
    static CONSOLE: OnceLock<Console> = OnceLock::new();
    CONSOLE.get_or_init(|| Console {
        multi: MultiProgress::new(),
    })
}

/// Get the global `MultiProgress`.
pub(crate) fn multi() -> &'static MultiProgress {
    &console().multi
}

/// Run `func` with any progress indicator suspended so that the text that prints
/// above it doesn't compete with its updating.
pub(crate) fn with_suspended<R>(func: impl FnOnce() -> R) -> R {
    console().multi.suspend(func)
}

/// Format a status line.
pub(crate) fn status_line(style: &Style, label: &str, subject: &str) -> String {
    let pad = status_label_width().saturating_sub(console::measure_text_width(label)) + 2;
    let label = style.apply_to(format!("{}{label}", " ".repeat(pad)));
    let subject = subject.trim_end();
    if subject.is_empty() {
        label.to_string()
    } else {
        format!("{label} {subject}")
    }
}

/// Format a milestone header.
pub fn milestone(label: impl AsRef<str>, subject: impl AsRef<str>) {
    let line = status_line(&theme().milestone, label.as_ref(), subject.as_ref());
    with_suspended(move || eprintln!("{line}"));
}

/// Format a step.
pub fn step(label: impl AsRef<str>, subject: impl AsRef<str>) {
    let line = status_line(&theme().step, label.as_ref(), subject.as_ref());
    with_suspended(move || eprintln!("{line}"));
}

/// Print a warning.
pub fn warn(message: impl AsRef<str>) {
    let theme = theme();
    let label = theme.warn_label.apply_to(t!("label-warning"));
    let body = message.as_ref();
    with_suspended(move || eprintln!("{label} {body}"));
}

/// Print an error.
pub fn error(message: impl AsRef<str>) {
    let theme = theme();
    let label = theme.error_label.apply_to(t!("label-error"));
    let body = message.as_ref();
    with_suspended(move || eprintln!("{label} {body}"));
}

/// Report an error and its causes.
pub fn report_error(err: &anyhow::Error) {
    let theme = theme();
    let code = err
        .chain()
        .find_map(|cause| cause.downcast_ref::<crate::error::CodedError>())
        .and_then(|coded| coded.code());
    let label = match code {
        Some(code) => t!("label-error-coded", code = code),
        None => t!("label-error-coded", code = "GDVM0000"),
    };
    let label = theme.error_label.apply_to(label);
    let body = err.to_string();

    with_suspended(move || eprintln!("{label} {body}"));

    for cause in err.chain().skip(1) {
        let label = t!("label-caused-by");
        let body = cause.to_string();

        with_suspended(move || eprintln!("  {label} {body}"));
    }
}

/// Print a note.
pub fn note(message: impl AsRef<str>) {
    let label = theme().note_label.apply_to(t!("label-note"));
    let message = message.as_ref().to_string();
    with_suspended(move || eprintln!("{label} {message}"));
}

/// Print a tip.
pub fn tip(message: impl AsRef<str>) {
    let styled = theme().tip.apply_to(message.as_ref().to_string());
    with_suspended(move || eprintln!("{styled}"));
}
