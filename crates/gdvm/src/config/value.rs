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

use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use strum::VariantNames;

use crate::terr;

/// Trait for enums that can be used as config values.
pub(super) trait ConfigEnum: std::str::FromStr + AsRef<str> + VariantNames {}

/// Where a value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValueSource {
    /// The value was provided by the user on the CLI.
    Cli,
    /// The value was read from the config file.
    File,
}

pub(super) trait ConfigValue:
    Sized + Clone + std::fmt::Debug + Serialize + DeserializeOwned
{
    /// Parse a value from the CLI.
    fn parse_config_value(key: &str, value: &str) -> Result<Self>;

    /// Check a value for validity and optionally normalize it.
    fn check_config_value(self, key: &str, source: ValueSource) -> Result<Self> {
        let _ = (key, source);
        Ok(self)
    }

    /// Format the value for display.
    fn to_config_string(&self) -> String;
}

impl ConfigValue for u64 {
    fn parse_config_value(key: &str, value: &str) -> Result<Self> {
        value
            .parse()
            .map_err(|_| terr!("error-config-invalid-number", key = key, value = value).into())
    }

    fn to_config_string(&self) -> String {
        self.to_string()
    }
}

impl<T> ConfigValue for T
where
    T: ConfigEnum + Clone + std::fmt::Debug + Serialize + DeserializeOwned,
{
    fn parse_config_value(key: &str, value: &str) -> Result<Self> {
        value.parse().ok().ok_or_else(|| {
            terr!(
                "error-config-invalid-enum",
                key = key,
                value = value,
                expected = Self::VARIANTS.join(", ")
            )
            .into()
        })
    }

    fn to_config_string(&self) -> String {
        self.as_ref().to_string()
    }
}

/// A value that shouldn't be written to the config as it's empty.
pub(super) trait IsEmpty {
    fn is_empty_value(&self) -> bool;
}

pub(super) fn is_empty<T: IsEmpty>(value: &T) -> bool {
    value.is_empty_value()
}

impl<T> IsEmpty for Option<T> {
    fn is_empty_value(&self) -> bool {
        self.is_none()
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty_value(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V> IsEmpty for HashMap<K, V> {
    fn is_empty_value(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigKey;

    #[test]
    fn test_numbers_parse_and_render() {
        let key = ConfigKey::PruneMaxAgeDays.as_str();

        assert_eq!(u64::parse_config_value(key, "7").unwrap(), 7);
        assert_eq!(7u64.to_config_string(), "7");

        assert!(u64::parse_config_value(key, "soon").is_err());
        assert!(u64::parse_config_value(key, "-1").is_err());
        assert!(u64::parse_config_value(key, "").is_err());
    }

    #[test]
    fn test_values_are_taken_as_given_by_default() {
        let key = ConfigKey::PruneMaxAgeDays.as_str();

        assert_eq!(7u64.check_config_value(key, ValueSource::Cli).unwrap(), 7);
        assert_eq!(7u64.check_config_value(key, ValueSource::File).unwrap(), 7);
    }
}
