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

use icu::datetime::{DateTimeFormatter, fieldsets};
use icu::locale::Locale;
use icu::time::DateTime as IcuDateTime;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use jiff_icu::ConvertFrom as _;

use crate::i18n::current_locale;

pub fn now_iso8601() -> String {
    jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%S.%3fZ")
        .to_string()
}

/// Format a Unix timestamp for the current locale. Falls back to ISO format.
pub fn format_unix_timestamp_local(secs: u64) -> String {
    let secs = i64::try_from(secs).unwrap_or(i64::MAX);

    let zoned = match Timestamp::from_second(secs) {
        Ok(timestamp) => timestamp.to_zoned(TimeZone::system()),
        Err(_) => return secs.to_string(),
    };

    let icu_datetime = IcuDateTime::convert_from(zoned.datetime());

    let locale = current_locale()
        .parse::<Locale>()
        .unwrap_or(Locale::UNKNOWN);

    match DateTimeFormatter::try_new(locale.into(), fieldsets::YMDT::medium()) {
        Ok(formatter) => formatter.format(&icu_datetime).to_string(),
        Err(_) => zoned.strftime("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// The current time as whole seconds since the Unix epoch.
pub fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The modification time of a filesystem entry as whole seconds since the Unix
/// epoch, if it can be determined.
pub fn modified_unix_secs(path: &std::path::Path) -> Option<u64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// The number of whole seconds elapsed between an earlier Unix timestamp and
/// `now`.
pub fn age_secs(now: u64, earlier: u64) -> u64 {
    now.saturating_sub(earlier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_secs_computes_elapsed_seconds() {
        assert_eq!(age_secs(100, 40), 60);
        assert_eq!(age_secs(100, 100), 0);
    }

    #[test]
    fn age_secs_saturates_on_future_dated_timestamps() {
        assert_eq!(age_secs(100, 200), 0);
        assert_eq!(age_secs(0, u64::MAX), 0);
    }
}
