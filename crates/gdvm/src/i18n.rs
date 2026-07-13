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

use std::env;
use std::sync::OnceLock;

use anyhow::Result;
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
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

/// Get the current locale. Falls back to `en-US` if `LANG` is not set.
pub fn current_locale() -> String {
    env::var("LANG")
        .unwrap_or_else(|_| "en-US".to_string())
        .split('.')
        .next()
        .unwrap_or("en-US")
        .replace("_", "-")
}

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
            bundle.set_formatter(Some(format_value));
            bundle
                .add_function("NUMBER", number_function)
                .map_err(|e| anyhow::anyhow!("Failed to add NUMBER for {locale}: {e:?}"))?;

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

    /// Translate a key or one of its attributes.
    pub fn translate_message(
        &self,
        key: &str,
        attr: Option<&str>,
        args: &[(&str, FluentValue)],
    ) -> String {
        self.translate_message_in(&current_locale(), key, attr, args)
    }

    /// Translate a key or one of its attributes for a specific locale.
    pub fn translate_message_in(
        &self,
        locale: &str,
        key: &str,
        attr: Option<&str>,
        args: &[(&str, FluentValue)],
    ) -> String {
        let locale = locale.to_string();

        let fallback_bundle = if let Some(fallback_bundle) = self.bundles.get("en-US") {
            fallback_bundle
        } else {
            panic!("Fallback locale en-US not found");
        };

        let bundle = self.bundles.get(&locale).unwrap_or(fallback_bundle);

        let mut fluent_args = fluent_bundle::FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, v.clone());
        }

        let format_from = |locale: &str, bundle: &FluentBundle<FluentResource>| -> Option<String> {
            let msg = bundle.get_message(key)?;
            let pattern = match attr {
                Some(attr) => msg.get_attribute(attr)?.value(),
                None => msg.value()?,
            };
            let mut errors = vec![];
            FORMATTING_LOCALE.with(|cell| *cell.borrow_mut() = locale.to_string());
            Some(
                bundle
                    .format_pattern(pattern, Some(&fluent_args), &mut errors)
                    .to_string(),
            )
        };

        format_from(&locale, bundle)
            .or_else(|| format_from("en-US", fallback_bundle))
            .unwrap_or_else(|| match attr {
                Some(attr) => format!("{key}.{attr}"),
                None => key.to_string(),
            })
    }
}

thread_local! {
    /// The locale of the bundle currently being formatted.
    static FORMATTING_LOCALE: std::cell::RefCell<String> =
        const { std::cell::RefCell::new(String::new()) };
}

/// Custom value formatter for bundles.
fn format_value(
    value: &FluentValue,
    _memoizer: &intl_memoizer::concurrent::IntlLangMemoizer,
) -> Option<String> {
    match value {
        FluentValue::Number(n) => {
            Some(FORMATTING_LOCALE.with(|cell| format_number(n, &cell.borrow())))
        }
        _ => None,
    }
}

/// Defines the NUMBER function for Fluent.
fn number_function<'a>(positional: &[FluentValue<'a>], named: &FluentArgs) -> FluentValue<'a> {
    let Some(FluentValue::Number(number)) = positional.first() else {
        return FluentValue::Error;
    };

    let mut number = number.clone();
    for (name, value) in named.iter() {
        let FluentValue::Number(digits) = value else {
            continue;
        };
        let digits = digits.value as usize;
        match name {
            "minimumFractionDigits" => number.options.minimum_fraction_digits = Some(digits),
            "maximumFractionDigits" => number.options.maximum_fraction_digits = Some(digits),
            _ => {}
        }
    }

    FluentValue::Number(number)
}

/// Format a number for a locale using ICU.
fn format_number(number: &FluentNumber, locale: &str) -> String {
    use fixed_decimal::{Decimal, FloatPrecision};
    use icu::decimal::DecimalFormatter;

    let locale: icu::locale::Locale = locale.parse().unwrap_or(icu::locale::Locale::UNKNOWN);

    let Ok(formatter) = DecimalFormatter::try_new(locale.into(), Default::default()) else {
        return number.value.to_string();
    };

    let precision = match number.options.maximum_fraction_digits {
        Some(digits) => FloatPrecision::Magnitude(-(digits as i16)),
        None => FloatPrecision::RoundTrip,
    };

    let Ok(mut decimal) = Decimal::try_from_f64(number.value, precision) else {
        return number.value.to_string();
    };

    if let Some(digits) = number.options.minimum_fraction_digits {
        decimal.pad_end(-(digits as i16));
    }

    formatter.format_to_string(&decimal)
}

#[macro_export]
macro_rules! t {
    ($key:expr $(, $( $arg_key:ident = $arg_val:expr ),* )? $(,)?) => {
        $crate::i18n::I18n::get().translate_message(
            $key,
            None,
            &[
                $( $( ( stringify!($arg_key), fluent_bundle::FluentValue::from($arg_val) ) ),* )?
            ]
        )
    };
}

#[macro_export]
macro_rules! t_attr {
    ($key:expr, $attr:expr $(, $( $arg_key:ident = $arg_val:expr ),* )? $(,)?) => {
        $crate::i18n::I18n::get().translate_message(
            $key,
            Some($attr),
            &[
                $( $( ( stringify!($arg_key), fluent_bundle::FluentValue::from($arg_val) ) ),* )?
            ]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn translate(locale: &str, key: &str, args: &[(&str, FluentValue)]) -> String {
        I18n::get().translate_message_in(locale, key, None, args)
    }

    #[test]
    fn terms_render_with_case_variants() {
        let hy = translate("hy-AM", "help-upgrade", &[]);
        assert!(hy.contains("gdvm-ը"), "{hy}");

        let hy = translate(
            "hy-AM",
            "pinned-success",
            &[("version", FluentValue::from("4.3"))],
        );
        assert!(hy.contains("gdvm.toml-ում"), "{hy}");

        let en = translate("en-US", "help-upgrade", &[]);
        assert!(en.contains("gdvm"), "{en}");
    }

    #[test]
    fn plural_categories_select_correctly() {
        let one = translate(
            "ru-RU",
            "registry-validate-ok",
            &[("count", FluentValue::from(1))],
        );
        assert!(one.contains("артефакт)"), "{one}");
        let few = translate(
            "ru-RU",
            "registry-validate-ok",
            &[("count", FluentValue::from(3))],
        );
        assert!(few.contains("артефакта"), "{few}");
        let many = translate(
            "ru-RU",
            "registry-validate-ok",
            &[("count", FluentValue::from(7))],
        );
        assert!(many.contains("артефактов"), "{many}");

        let one = translate(
            "en-US",
            "prune-preserved-by-link",
            &[("count", FluentValue::from(1))],
        );
        assert!(one.contains("install still"), "{one}");
        let other = translate(
            "en-US",
            "prune-preserved-by-link",
            &[("count", FluentValue::from(2))],
        );
        assert!(other.contains("installs still"), "{other}");
    }

    #[test]
    fn numbers_format_for_the_locale() {
        let en = translate(
            "en-US",
            "info-size",
            &[
                ("value", FluentValue::from(106.7)),
                ("unit", FluentValue::from("mib")),
            ],
        );
        assert!(en.contains("106.7"), "{en}");

        let no = translate(
            "nn-NO",
            "info-size",
            &[
                ("value", FluentValue::from(106.7)),
                ("unit", FluentValue::from("mib")),
            ],
        );
        assert!(no.contains("106,7"), "{no}");
    }
}
