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
use serde::{Deserialize, Serialize};

use super::RegistryConfig;
use super::value::{ConfigValue, IsEmpty, ValueSource, is_empty};
use crate::terr;

/// Schema for the config file. `tables` are settings the user can change with
/// the config command. `internal` are keys gdvm uses for its own purposes but
/// are not surfaced to the user.
///
/// Keys in `tables` are grouped by the table they live in:
///
/// ```text
/// "<table>" => <field>: <StructName> {
///     <Variant> = "<key>" => <field>: <type>, sensitive = <bool>;
/// }
/// ```
///
/// Keys can also define a default value, as well as a `check` function, which
/// can perform additional validation or normalization on the value:
///
/// ```text
/// <Variant> = "<key>" => <field>: <type>,
///     sensitive = <bool>,
///     default = <expr>,
///     check = <fn>;
/// ```
///
/// Use `try_default` instead if getting the default can error:
///
/// ```text
/// <Variant> = "<key>" => <field>: <type>,
///     sensitive = <bool>,
///     try_default = <expr>;
/// ```
///
/// Keys in `internal` are simple pairs:
///
/// ```text
/// <Variant> = "<key>" => <field>: <type>;
/// ```
macro_rules! config_schema {
    (
        tables {
            $(
                $(#[$tmeta:meta])*
                $tkey:literal => $tfield:ident: $ttype:ident {
                    $(
                        $(#[$smeta:meta])*
                        $svariant:ident = $skey:literal => $sfield:ident: $sty:ty,
                            sensitive = $ssensitive:literal
                            $(, default = $sdefault:expr)?
                            $(, try_default = $stry:expr)?
                            $(, check = $scheck:path)?;
                    )*
                }
            )*
        }
        internal {
            $(
                $(#[$imeta:meta])*
                $ivariant:ident = $ikey:literal => $ifield:ident: $ity:ty;
            )*
        }
    ) => {
        $(
            $(#[$tmeta])*
            #[derive(Debug, Default, Clone, Serialize, Deserialize)]
            pub struct $ttype {
                $(
                    $(#[$smeta])*
                    #[serde(default, rename = $skey, skip_serializing_if = "is_empty")]
                    pub $sfield: Option<$sty>,
                )*
            }

            impl IsEmpty for $ttype {
                fn is_empty_value(&self) -> bool {
                    $(self.$sfield.is_none() &&)* true
                }
            }

            impl $ttype {
                $($(
                    #[doc = concat!("The configured `", $skey, "`, or its default when unset.")]
                    pub fn $sfield(&self) -> $sty {
                        match &self.$sfield {
                            Some(value) => value.clone(),
                            None => $sdefault,
                        }
                    }
                )?)*
                $($(
                    #[doc = concat!("The configured `", $skey, "`, or its default when unset.")]
                    pub fn $sfield(&self) -> Result<$sty> {
                        match &self.$sfield {
                            Some(value) => Ok(value.clone()),
                            None => $stry,
                        }
                    }
                )?)*
            }
        )*

        /// Machine-level config.
        #[derive(Debug, Default, Clone, Serialize, Deserialize)]
        pub struct Config {
            $(
                $(#[$tmeta])*
                #[serde(default, rename = $tkey, skip_serializing_if = "is_empty")]
                pub $tfield: $ttype,
            )*
            $(
                $(#[$imeta])*
                #[serde(default, rename = $ikey, skip_serializing_if = "is_empty")]
                pub $ifield: $ity,
            )*
        }

        /// A setting the user can set with `gdvm config`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ConfigKey {
            $($(
                $(#[$smeta])*
                $svariant,
            )*)*
        }

        impl ConfigKey {
            /// Every known configuration key.
            pub const ALL: &'static [ConfigKey] = &[$($(ConfigKey::$svariant,)*)*];

            /// The dotted name of the key.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $($(ConfigKey::$svariant => concat!($tkey, ".", $skey),)*)*
                }
            }

            /// Get whether the key's value should be masked when displayed.
            pub const fn is_sensitive(self) -> bool {
                match self {
                    $($(ConfigKey::$svariant => $ssensitive,)*)*
                }
            }
        }

        /// A key gdvm owns in the config file.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(super) enum ManagedKey {
            /// A setting the user can change.
            Setting(ConfigKey),
            $($(#[$imeta])* $ivariant,)*
        }

        impl ManagedKey {
            /// All known keys, both settings and internal.
            pub(super) const ALL: &'static [ManagedKey] = &[
                $($(ManagedKey::Setting(ConfigKey::$svariant),)*)*
                $(ManagedKey::$ivariant,)*
            ];

            /// The dotted name of the key.
            pub(super) const fn as_str(self) -> &'static str {
                match self {
                    ManagedKey::Setting(key) => key.as_str(),
                    $(ManagedKey::$ivariant => $ikey,)*
                }
            }
        }

        impl Config {
            /// Retrieve the value of a configuration key, if set.
            pub fn get_value(&self, key: ConfigKey) -> Option<String> {
                match key {
                    $($(ConfigKey::$svariant =>
                        self.$tfield.$sfield.as_ref().map(ConfigValue::to_config_string),)*)*
                }
            }

            /// Set a configuration key to a value as the user typed it.
            pub fn set_value(&mut self, key: ConfigKey, value: &str) -> Result<()> {
                match key {
                    $($(ConfigKey::$svariant => {
                        let value = <$sty as ConfigValue>::parse_config_value(key.as_str(), value)?
                            .check_config_value(key.as_str(), ValueSource::Cli)?;
                        $(let value = $scheck(key, value, ValueSource::Cli)?;)?
                        self.$tfield.$sfield = Some(value);
                    })*)*
                }
                Ok(())
            }

            /// Unset (remove) a configuration key's value.
            pub fn unset_value(&mut self, key: ConfigKey) {
                match key {
                    $($(ConfigKey::$svariant => self.$tfield.$sfield = None,)*)*
                }
            }

            /// List all set configuration keys as (key, value) pairs.
            pub fn list_set_keys(&self) -> Vec<(ConfigKey, String)> {
                let mut entries = Vec::new();
                $($(
                    if let Some(value) = self.$tfield.$sfield.as_ref() {
                        entries.push((
                            ConfigKey::$svariant,
                            ConfigValue::to_config_string(value),
                        ));
                    }
                )*)*
                entries
            }

            /// Read a managed key's value out of the config file. If the value
            /// cannot be used, a description of the problem is returned.
            pub(super) fn apply_managed_key<'de, D: serde::Deserializer<'de>>(
                &mut self,
                key: ManagedKey,
                de: D,
            ) -> std::result::Result<(), String> {
                match key {
                    $($(ManagedKey::Setting(ConfigKey::$svariant) => {
                        let setting = ConfigKey::$svariant;

                        <$sty as Deserialize>::deserialize(de)
                            .map_err(|e| e.to_string())
                            .and_then(|value| {
                                value
                                    .check_config_value(setting.as_str(), ValueSource::File)
                                    .map_err(|e| e.to_string())
                            })
                            $(.and_then(|value| {
                                $scheck(setting, value, ValueSource::File)
                                    .map_err(|e| e.to_string())
                            }))?
                            .map(|value| self.$tfield.$sfield = Some(value))
                    })*)*
                    $(ManagedKey::$ivariant => <$ity as Deserialize>::deserialize(de)
                        .map(|value| self.$ifield = value)
                        .map_err(|e| e.to_string()),)*
                }
            }
        }
    };
}

/// The default maximum age, in days, before an unused asset becomes eligible
/// for pruning, unless `prune.max-age-days` is configured.
pub const DEFAULT_PRUNE_MAX_AGE_DAYS: u64 = 30;

config_schema! {
    tables {
        /// Settings for `gdvm prune`.
        "prune" => prune: PruneConfig {
            /// Maximum age, in days, before an unused asset becomes eligible
            /// for pruning. When unset, `DEFAULT_PRUNE_MAX_AGE_DAYS` is used.
            PruneMaxAgeDays = "max-age-days" => max_age_days: u64, sensitive = false,
                default = DEFAULT_PRUNE_MAX_AGE_DAYS;
        }
    }

    internal {
        /// Registry configs keyed by alias.
        Registries = "registries" => registries: HashMap<String, RegistryConfig>;

        /// Base URLs of unofficial registries the user has confirmed they
        /// trust.
        TrustedRegistries = "trusted-registries" => trusted_registries: Vec<String>;
    }
}

impl std::str::FromStr for ConfigKey {
    type Err = anyhow::Error;

    fn from_str(key: &str) -> Result<Self> {
        ConfigKey::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == key)
            .ok_or_else(|| terr!("error-config-unknown-key", key = key).into())
    }
}

impl std::fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_unset_list() {
        let mut cfg = Config::default();

        assert!(cfg.get_value(ConfigKey::PruneMaxAgeDays).is_none());
        assert!(cfg.set_value(ConfigKey::PruneMaxAgeDays, "5").is_ok());
        assert_eq!(
            cfg.get_value(ConfigKey::PruneMaxAgeDays),
            Some("5".to_string())
        );
        assert!(!ConfigKey::PruneMaxAgeDays.is_sensitive());

        let listed = cfg.list_set_keys();

        assert_eq!(listed, vec![(ConfigKey::PruneMaxAgeDays, "5".to_string())]);

        cfg.unset_value(ConfigKey::PruneMaxAgeDays);

        assert!(cfg.get_value(ConfigKey::PruneMaxAgeDays).is_none());
        assert!(cfg.list_set_keys().is_empty());
    }

    #[test]
    fn test_config_key_parsing_roundtrip() {
        for &key in ConfigKey::ALL {
            assert_eq!(key.as_str().parse::<ConfigKey>().unwrap(), key);
            assert_eq!(key.to_string(), key.as_str());
        }

        assert_eq!(
            "prune.max-age-days".parse::<ConfigKey>().unwrap(),
            ConfigKey::PruneMaxAgeDays
        );
        assert!("prune.max_age_days".parse::<ConfigKey>().is_err());
        assert!("unknown".parse::<ConfigKey>().is_err());
        assert!("github.token".parse::<ConfigKey>().is_err());
    }

    #[test]
    fn test_managed_keys_cover_every_setting() {
        for &key in ConfigKey::ALL {
            assert!(ManagedKey::ALL.contains(&ManagedKey::Setting(key)));
            assert_eq!(ManagedKey::Setting(key).as_str(), key.as_str());
        }

        assert!(ManagedKey::ALL.contains(&ManagedKey::Registries));
        assert!(ManagedKey::ALL.contains(&ManagedKey::TrustedRegistries));
    }

    #[test]
    fn test_empty_config_serializes_to_nothing() {
        assert_eq!(toml::to_string(&Config::default()).unwrap(), "");
    }

    #[test]
    fn test_settings_toml_roundtrip() {
        let mut cfg = Config::default();
        cfg.set_value(ConfigKey::PruneMaxAgeDays, "14").unwrap();

        let parsed: Config = toml::from_str(&toml::to_string(&cfg).unwrap()).unwrap();

        assert_eq!(parsed.prune.max_age_days, Some(14));
    }

    #[test]
    fn test_a_key_falls_back_to_its_default() {
        let mut cfg = Config::default();

        assert_eq!(cfg.prune.max_age_days(), DEFAULT_PRUNE_MAX_AGE_DAYS);

        cfg.set_value(ConfigKey::PruneMaxAgeDays, "7").unwrap();
        assert_eq!(cfg.prune.max_age_days(), 7);

        assert!(cfg.set_value(ConfigKey::PruneMaxAgeDays, "soon").is_err());
        assert_eq!(cfg.prune.max_age_days(), 7);

        cfg.unset_value(ConfigKey::PruneMaxAgeDays);
        assert_eq!(cfg.prune.max_age_days(), DEFAULT_PRUNE_MAX_AGE_DAYS);
    }

    #[test]
    fn test_values_from_the_file_go_through_the_value_checks() {
        use serde::de::IntoDeserializer;
        use serde::de::value::Error;

        let key = ManagedKey::Setting(ConfigKey::PruneMaxAgeDays);
        let mut cfg = Config::default();

        let value: <u64 as IntoDeserializer<Error>>::Deserializer = 7u64.into_deserializer();
        cfg.apply_managed_key(key, value).unwrap();
        assert_eq!(cfg.prune.max_age_days, Some(7));

        let value: <&str as IntoDeserializer<Error>>::Deserializer = "soon".into_deserializer();
        assert!(cfg.apply_managed_key(key, value).is_err());
    }
}

#[cfg(test)]
mod schema_tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn even_only(key: ConfigKey, value: u64, source: ValueSource) -> Result<u64> {
        match (source, value % 2) {
            (_, 0) => Ok(value),
            (ValueSource::Cli, _) => Ok(value + 1),
            (ValueSource::File, _) => Err(anyhow::anyhow!("{} must be even", key.as_str())),
        }
    }

    static ENVIRONMENT: AtomicU64 = AtomicU64::new(0);

    fn from_the_environment() -> Result<u64> {
        match ENVIRONMENT.load(Ordering::Relaxed) {
            0 => Err(anyhow::anyhow!("nothing to get the default from")),
            found => Ok(found),
        }
    }

    config_schema! {
        tables {
            "example" => example: ExampleConfig {
                Plain = "plain" => plain: u64, sensitive = false;
                Secret = "secret" => secret: u64, sensitive = true;
                Default = "default" => default: u64, sensitive = false, default = 12;
                TryDefault = "try_default" => try_default: u64, sensitive = false,
                    try_default = from_the_environment();
                Checked = "checked" => checked: u64, sensitive = false, check = even_only;
            }
        }

        internal {
            Extras = "extras" => extras: Vec<String>;
        }
    }

    fn from_file(cfg: &mut Config, key: ConfigKey, value: u64) -> std::result::Result<(), String> {
        use serde::de::IntoDeserializer;
        use serde::de::value::Error;

        let value: <u64 as IntoDeserializer<Error>>::Deserializer = value.into_deserializer();

        cfg.apply_managed_key(ManagedKey::Setting(key), value)
    }

    #[test]
    fn test_keys_carry_their_table_and_their_flags() {
        assert_eq!(ConfigKey::Plain.as_str(), "example.plain");
        assert_eq!(ConfigKey::ALL.len(), 5);
        assert!(!ConfigKey::Plain.is_sensitive());
        assert!(ConfigKey::Secret.is_sensitive());

        assert_eq!(ManagedKey::Extras.as_str(), "extras");
        assert_eq!(ManagedKey::ALL.len(), 6);
    }

    #[test]
    fn test_a_default_can_be_worked_out_at_the_time_it_is_asked_for() {
        let mut cfg = Config::default();

        ENVIRONMENT.store(0, Ordering::Relaxed);
        assert!(cfg.example.try_default().is_err());

        ENVIRONMENT.store(7, Ordering::Relaxed);
        assert_eq!(cfg.example.try_default().unwrap(), 7);

        ENVIRONMENT.store(9, Ordering::Relaxed);
        assert_eq!(cfg.example.try_default().unwrap(), 9);

        cfg.set_value(ConfigKey::TryDefault, "5").unwrap();
        ENVIRONMENT.store(0, Ordering::Relaxed);
        assert_eq!(cfg.example.try_default().unwrap(), 5);
    }

    #[test]
    fn test_a_key_with_a_default_reads_as_set() {
        let mut cfg = Config::default();

        assert_eq!(cfg.example.default(), 12);
        assert!(cfg.get_value(ConfigKey::Default).is_none());

        cfg.set_value(ConfigKey::Default, "5").unwrap();
        assert_eq!(cfg.example.default(), 5);
    }

    #[test]
    fn test_a_keys_own_rule_gets_the_last_word() {
        let mut cfg = Config::default();

        cfg.set_value(ConfigKey::Checked, "3").unwrap();
        assert_eq!(cfg.get_value(ConfigKey::Checked), Some("4".to_string()));

        from_file(&mut cfg, ConfigKey::Checked, 8).unwrap();
        assert_eq!(cfg.example.checked, Some(8));

        let problem = from_file(&mut cfg, ConfigKey::Checked, 9).unwrap_err();

        assert!(problem.contains("example.checked"));
        assert_eq!(cfg.example.checked, Some(8));
    }

    #[test]
    fn test_keys_without_a_rule_are_left_as_they_are() {
        let mut cfg = Config::default();

        cfg.set_value(ConfigKey::Plain, "3").unwrap();
        from_file(&mut cfg, ConfigKey::Secret, 9).unwrap();

        assert_eq!(
            cfg.list_set_keys(),
            vec![
                (ConfigKey::Plain, "3".to_string()),
                (ConfigKey::Secret, "9".to_string()),
            ]
        );

        cfg.unset_value(ConfigKey::Plain);
        cfg.unset_value(ConfigKey::Secret);

        assert!(cfg.list_set_keys().is_empty());
        assert_eq!(toml::to_string(&cfg).unwrap(), "");
    }

    #[test]
    fn test_the_table_maps_to_toml() {
        let mut cfg = Config::default();

        cfg.set_value(ConfigKey::Plain, "3").unwrap();
        cfg.extras.push("kept".to_string());

        let text = toml::to_string(&cfg).unwrap();

        assert!(text.contains("[example]"));
        assert!(text.contains("plain = 3"));
        assert!(text.contains(r#"extras = ["kept"]"#));

        let parsed: Config = toml::from_str(&text).unwrap();

        assert_eq!(parsed.example.plain, Some(3));
        assert_eq!(parsed.extras, vec!["kept".to_string()]);
    }
}
