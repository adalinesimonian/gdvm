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

use anyhow::Result;
use clap::ArgMatches;
use gdvm::i18n::current_locale;
use icu::locale::{Locale, LocaleDirectionality};
use serde::Serialize;
use unicode_width::UnicodeWidthStr;

/// The output format of a command.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    /// Text for people.
    Text,
    /// Machine-readable JSON.
    Json,
}

impl OutputFormat {
    /// Read the `--format` flag from parsed arguments.
    pub(crate) fn from_matches(matches: &ArgMatches) -> Self {
        match matches.get_one::<String>("format").map(String::as_str) {
            Some("json") => OutputFormat::Json,
            _ => OutputFormat::Text,
        }
    }
}

/// Print a value as JSON.
pub(crate) fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// A version entry as outputted in JSON.
#[derive(Serialize)]
pub(crate) struct VersionEntry {
    pub(crate) version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) variant: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) registry: Option<String>,
}

/// Format a byte count into a user-friendly string.
pub(crate) fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Format `(label, value)` tuples as a table.
pub(crate) fn format_label_value_table(rows: &[(String, String)]) -> String {
    let label_width = rows
        .iter()
        .map(|(label, _)| label.width())
        .max()
        .unwrap_or(0);

    let right_to_left = current_locale()
        .parse::<Locale>()
        .map(|locale| LocaleDirectionality::new_common().is_right_to_left(&locale.id))
        .unwrap_or(false);

    rows.iter()
        .map(|(label, value)| {
            let padding = " ".repeat(label_width.saturating_sub(label.width()));
            if right_to_left {
                format!("{padding}{label} {value}")
            } else {
                format!("{label}{padding} {value}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_entry_omits_absent_fields() {
        let entry = VersionEntry {
            version: "4.3-stable".into(),
            variant: None,
            registry: None,
        };
        assert_eq!(
            serde_json::to_string(&entry).unwrap(),
            r#"{"version":"4.3-stable"}"#
        );

        let entry = VersionEntry {
            version: "4.3-stable".into(),
            variant: Some("csharp".into()),
            registry: Some("official".into()),
        };
        assert_eq!(
            serde_json::to_string(&entry).unwrap(),
            r#"{"version":"4.3-stable","variant":"csharp","registry":"official"}"#
        );
    }

    #[test]
    fn bytes_format_scales_units() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KiB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
    }
}
