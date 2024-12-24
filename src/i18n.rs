use anyhow::Result;
use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::env;
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
}

impl I18n {
    /// Create a new I18n instance and load the translation resources
    pub fn new() -> Result<Self> {
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

        Ok(Self { bundles })
    }

    /// Translate a key without arguments
    pub fn t(&self, key: &str) -> String {
        let locale = env::var("LANG")
            .unwrap_or_else(|_| "en-US".to_string())
            .split('.')
            .next()
            .unwrap_or("en-US")
            .replace("_", "-");
        let bundle = self.bundles.get(&locale).unwrap_or_else(|| {
            self.bundles
                .get("en-US")
                .expect("Fallback locale not found")
        });

        let msg = bundle.get_message(key).unwrap_or_else(|| {
            self.bundles
                .get("en-US")
                .expect("Fallback locale not found")
                .get_message(key)
                .expect("Message not found in fallback locale")
        });

        let pattern = msg.value().expect("Message has no value.");
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
        let bundle = self.bundles.get(&locale).unwrap_or_else(|| {
            self.bundles
                .get("en-US")
                .expect("Fallback locale not found")
        });

        let msg = bundle.get_message(key).unwrap_or_else(|| {
            self.bundles
                .get("en-US")
                .expect("Fallback locale not found")
                .get_message(key)
                .expect("Message not found in fallback locale")
        });

        let pattern = msg.value().expect("Message has no value.");
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
macro_rules! println_i18n {
    // With arguments
    ($i18n:expr, $key:expr, [$( ($arg_key:expr, $arg_val:expr) ),*]) => {
        println!(
            "{}",
            $i18n.t_args(
                $key,
                &[
                    $( ($arg_key, FluentValue::from($arg_val)) ),*
                ]
            )
        );
    };
    // Without arguments
    ($i18n:expr, $key:expr) => {
        println!("{}", $i18n.t($key));
    };
}
