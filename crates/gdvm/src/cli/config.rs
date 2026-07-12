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

use gdvm::config::{self, ConfigOps};
use gdvm::{eprintln_i18n, println_i18n, t};

use std::io::Write;

/// Handle the 'config' subcommand
pub(crate) fn sub_config(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let config = config::Config::load()?;
    match matches.subcommand() {
        Some(("get", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            let value = config.get_value(key);

            if super::format::OutputFormat::from_matches(sub_m) == super::format::OutputFormat::Json
            {
                #[derive(serde::Serialize)]
                struct ConfigValue<'a> {
                    key: &'a str,
                    value: Option<String>,
                }

                return super::format::print_json(&ConfigValue { key, value });
            }

            if let Some(value) = value {
                println!("{value}");
            } else {
                println!("{}", t!("config-key-not-set"));
            }
        }
        Some(("set", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            // If the value argument is not provided, prompt the user.
            let value: String = if let Some(v) = sub_m.get_one::<String>("value") {
                v.clone()
            } else {
                // Build the prompt message from the Fluent bundle.
                let prompt = t!("config-set-prompt", key = key.as_str());
                eprint!("{prompt} ");
                if config.is_sensitive_key(key) {
                    // Mask input for sensitive values.
                    match rpassword::prompt_password("") {
                        Ok(input) => input,
                        Err(err) => {
                            eprintln!("{}: {}", t!("error-reading-input"), err);
                            return Ok(());
                        }
                    }
                } else {
                    // For non-sensitive values, read normally.
                    std::io::stdout().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            if config.is_sensitive_key(key) {
                eprintln_i18n!("warning-setting-sensitive");
            }
            if !config::KNOWN_KEYS.contains(&key.as_str()) {
                eprintln_i18n!("error-unknown-config-key");
            } else {
                match config::Config::modify(|config| Ok(config.set_value(key, &value)))? {
                    Ok(()) => println_i18n!("config-set-success"),
                    Err(_) => eprintln_i18n!("error-invalid-config-value", key = key),
                }
            }
        }
        Some(("unset", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            match config::Config::modify(|config| Ok(config.unset_value(key)))? {
                Ok(()) => println_i18n!("config-unset-success", key = key),
                Err(_) => eprintln_i18n!("error-unknown-config-key"),
            }
        }
        Some(("list", sub_m)) => {
            let show_sensitive = sub_m.get_flag("show-sensitive");
            let available = sub_m.get_flag("available");
            if available {
                // List all known keys whether set or not.
                for key in config::KNOWN_KEYS {
                    let value_opt = config.get_value(key);
                    let display_value =
                        match (value_opt, config.is_sensitive_key(key), show_sensitive) {
                            (Some(_), true, false) => "********".to_string(),
                            (Some(val), _, _) => val,
                            (None, _, _) => "<not set>".to_string(),
                        };
                    println!("{key} = {display_value}");
                }
            } else {
                // List only keys that are set.
                for (key, value, sensitive) in config.list_set_keys() {
                    let display_value = if sensitive && !show_sensitive {
                        "********".to_string()
                    } else {
                        value
                    };
                    println!("{key} = {display_value}");
                }
            }
        }
        _ => eprintln!("{}", t!("error-invalid-config-subcommand")),
    }
    Ok(())
}
