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

use anyhow::Result;
use fluent_bundle::{FluentResource, FluentValue, concurrent::FluentBundle};
use std::env;
use std::sync::OnceLock;
use unic_langid::langid;

// Include the Fluent translation files as static strings
static EN_US_FTL: &str = include_str!("../i18n/en-US.ftl");
static FR_FR_FTL: &str = include_str!("../i18n/fr-FR.ftl");
static HY_AM_FTL: &str = include_str!("../i18n/hy-AM.ftl");
static NB_NO_FTL: &str = include_str!("../i18n/nb-NO.ftl");
static NN_NO_FTL: &str = include_str!("../i18n/nn-NO.ftl");
static RU_RU_FTL: &str = include_str!("../i18n/ru-RU.ftl");

use std::collections::HashMap;

static I18N: OnceLock<I18n> = OnceLock::new();

pub struct I18n {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl I18n {
    /// Create a new I18n instance and load the translation resources
    pub fn new() -> Result<Self> {
        let resources = [
            (langid!("en-US"), EN_US_FTL),
            (langid!("fr-FR"), FR_FR_FTL),
            (langid!("hy-AM"), HY_AM_FTL),
            (langid!("nb-NO"), NB_NO_FTL),
            (langid!("nn-NO"), NN_NO_FTL),
            (langid!("ru-RU"), RU_RU_FTL),
        ];
        let mut bundles = HashMap::new();

        for (locale, ftl_code) in resources.iter() {
            let mut bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
            let res = FluentResource::try_new(ftl_code.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse resource for {locale}: {e:?}"))?;
            bundle
                .add_resource(res)
                .map_err(|e| anyhow::anyhow!("Failed to add resource for {locale}: {e:?}"))?;
            bundles.insert(locale.to_string(), bundle);
        }

        Ok(Self { bundles })
    }

    /// Initialize the global i18n instance.
    pub fn init() -> Result<&'static Self> {
        if let Some(i18n) = I18N.get() {
            return Ok(i18n);
        }
        let i18n = Self::new()?;
        Ok(I18N.get_or_init(|| i18n))
    }

    /// Get the global i18n instance.
    pub fn get() -> &'static Self {
        I18N.get_or_init(|| Self::new().expect("Failed to load translation resources"))
    }

    /// Translate a key without arguments
    pub fn t(&self, key: &str) -> String {
        let locale = env::var("LANG")
            .unwrap_or_else(|_| "en-US".to_string())
            .split('.')
            .next()
            .unwrap_or("en-US")
            .replace("_", "-");

        let fallback_bundle = if let Some(fallback_bundle) = self.bundles.get("en-US") {
            fallback_bundle
        } else {
            panic!("Fallback locale en-US not found");
        };

        let bundle = if let Some(bundle) = self.bundles.get(&locale) {
            bundle
        } else {
            fallback_bundle
        };

        let msg = if let Some(msg) = bundle
            .get_message(key)
            .or_else(|| fallback_bundle.get_message(key))
        {
            msg
        } else {
            return key.to_string();
        };

        let pattern = match msg.value() {
            Some(pattern) => pattern,
            None => {
                return key.to_string();
            }
        };

        let mut errors = vec![];
        let value = bundle.format_pattern(pattern, None, &mut errors);

        value.to_string()
    }

    /// Translate a key with arguments
    pub fn t_args(&self, key: &str, args: &[(&str, FluentValue)]) -> String {
        let locale = env::var("LANG")
            .unwrap_or_else(|_| "en-US".to_string())
            .split('.')
            .next()
            .unwrap_or("en-US")
            .replace("_", "-");

        let fallback_bundle = if let Some(fallback_bundle) = self.bundles.get("en-US") {
            fallback_bundle
        } else {
            panic!("Fallback locale en-US not found");
        };

        let bundle = self.bundles.get(&locale).unwrap_or(fallback_bundle);

        let msg = if let Some(msg) = bundle
            .get_message(key)
            .or_else(|| fallback_bundle.get_message(key))
        {
            msg
        } else {
            return key.to_string();
        };

        let pattern = match msg.value() {
            Some(pattern) => pattern,
            None => {
                return key.to_string();
            }
        };

        let mut errors = vec![];
        let mut fluent_args = fluent_bundle::FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, v.clone());
        }
        let value = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);

        value.to_string()
    }
}

#[macro_export]
macro_rules! i18n_generic {
    ($noargs_func:ident, $args_func:ident, $key:expr, $( $arg_key:ident = $arg_val:expr ),* $(,)?) => {
        $crate::i18n::I18n::get().$args_func(
            $key,
            &[
                $( ( stringify!($arg_key), fluent_bundle::FluentValue::from($arg_val) ) ),*
            ]
        )
    };
    ($noargs_func:ident, $args_func:ident, $key:expr) => {
        $crate::i18n::I18n::get().$noargs_func($key)
    };
}

#[macro_export]
macro_rules! t {
    ($key:expr $(, $( $arg_key:ident = $arg_val:expr ),* )? $(,)?) => {
        $crate::i18n_generic!(
            t,
            t_args,
            $key
            $(, $( $arg_key = $arg_val ),* )?
        )
    };
}

#[macro_export]
macro_rules! i18n_print {
    ($print_fn:ident, $key:expr $(, $( $arg_key:ident = $arg_val:expr ),* )? $(,)?) => {
        $print_fn!(
            "{}",
            $crate::t!(
                $key
                $(, $( $arg_key = $arg_val ),* )?
            )
        )
    };
}

#[macro_export]
macro_rules! println_i18n {
    ($key:expr $(, $( $arg_key:ident = $arg_val:expr ),* )? $(,)?) => {
        $crate::i18n_print!(
            println,
            $key
            $(, $( $arg_key = $arg_val ),* )?
        )
    };
}

#[macro_export]
macro_rules! eprintln_i18n {
    ($key:expr $(, $( $arg_key:ident = $arg_val:expr ),* )? $(,)?) => {
        $crate::i18n_print!(
            eprintln,
            $key
            $(, $( $arg_key = $arg_val ),* )?
        )
    };
}

/// Prints to stdout if the result of an operation is true, otherwise prints to stderr
#[macro_export]
macro_rules! xprintln_i18n {
    ($result:expr, $success_key:expr, $failure_key:expr) => {
        if $result {
            $crate::println_i18n!($success_key);
        } else {
            $crate::eprintln_i18n!($failure_key);
        }
    };
}
