use anyhow::Result;
use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::env;
use terminal_size::terminal_size;
use textwrap::{wrap, Options, WordSeparator};
use unic_langid::langid;

// Include the Fluent translation files as static strings
static EN_US_FTL: &str = include_str!("../i18n/en-US.ftl");
static HY_AM_FTL: &str = include_str!("../i18n/hy-AM.ftl");
static NB_NO_FTL: &str = include_str!("../i18n/nb-NO.ftl");
static NN_NO_FTL: &str = include_str!("../i18n/nn-NO.ftl");
static RU_RU_FTL: &str = include_str!("../i18n/ru-RU.ftl");

use std::collections::HashMap;

pub struct I18n {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    /// The maximum length of a line when wrapping text. If 0, wrapping is only limited by the
    /// terminal width.
    max_length: usize,
}

impl I18n {
    /// Create a new I18n instance and load the translation resources
    pub fn new(max_length: usize) -> Result<Self> {
        let resources = [
            (langid!("en-US"), EN_US_FTL),
            (langid!("hy-AM"), HY_AM_FTL),
            (langid!("nb-NO"), NB_NO_FTL),
            (langid!("nn-NO"), NN_NO_FTL),
            (langid!("ru-RU"), RU_RU_FTL),
        ];
        let mut bundles = HashMap::new();

        for (locale, ftl_code) in resources.iter() {
            let mut bundle = FluentBundle::new(vec![locale.clone()]);
            let res = FluentResource::try_new(ftl_code.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse resource for {}: {:?}", locale, e))?;
            bundle
                .add_resource(res)
                .map_err(|e| anyhow::anyhow!("Failed to add resource for {}: {:?}", locale, e))?;
            bundles.insert(locale.to_string(), bundle);
        }

        Ok(Self {
            bundles,
            max_length,
        })
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

    /// Wraps a string to the terminal width, taking into account the maximum length. Uses Unicode
    /// properties for multi-lingual support. If the terminal width cannot be determined, a width of
    /// 80 is used. If the maximum length is 0, wrapping is only limited by the terminal width.
    pub fn wrap(&self, text: &str) -> String {
        let (terminal_width, _) =
            terminal_size().unwrap_or((terminal_size::Width(80), terminal_size::Height(24)));

        let width = if self.max_length > 0 {
            std::cmp::min(self.max_length, terminal_width.0 as usize)
        } else {
            terminal_width.0 as usize
        };

        let options = Options::new(width)
            .break_words(false)
            .word_separator(WordSeparator::UnicodeBreakProperties);

        wrap(text, &options).join("\n")
    }

    /// Translate a key without arguments and wrap the result
    pub fn t_w(&self, key: &str) -> String {
        self.wrap(self.t(key).as_str())
    }

    /// Translate a key with arguments and wrap the result
    pub fn t_args_w(&self, key: &str, args: &[(&str, FluentValue)]) -> String {
        self.wrap(self.t_args(key, args).as_str())
    }
}

#[macro_export]
macro_rules! println_i18n {
    // With arguments
    ($i18n:expr, $key:expr, [$( ($arg_key:expr, $arg_val:expr) ),*]) => {
        println!(
            "{}",
            $i18n.t_args_w(
                $key,
                &[
                    $( ($arg_key, fluent_bundle::FluentValue::from($arg_val)) ),*
                ]
            ).as_str()
        )
    };
    // Without arguments
    ($i18n:expr, $key:expr) => {
        println!("{}", $i18n.t_w($key).as_str())
    };
}
