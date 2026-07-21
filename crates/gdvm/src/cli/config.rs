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

use std::io::Write;

use gdvm::config::{self, ConfigOps};
use gdvm::{println_i18n, t, terr};

/// Handle the 'config' subcommand
pub(crate) fn sub_config(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let config = config::Config::load()?;
    match matches.subcommand() {
        Some(("get", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            let value = config.get_value(key);

            if super::format::OutputFormat::is_json(sub_m) {
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
            } else if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                return Err(terr!("error-non-interactive-value", key = key.as_str()).into());
            } else {
                // Build the prompt message from the Fluent bundle.
                let prompt = t!("config-set-prompt", key = key.as_str());
                eprint!("{prompt} ");
                if config.is_sensitive_key(key) {
                    // Mask input for sensitive values.
                    match rpassword::prompt_password("") {
                        Ok(input) => input,
                        Err(err) => {
                            return Err(terr!("error-reading-input").with_source(err).into());
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
                gdvm::ui::warn(t!("warning-setting-sensitive"));
            }
            if !config::KNOWN_KEYS.contains(&key.as_str()) {
                return Err(terr!("error-unknown-config-key").into());
            }
            config::Config::modify(|config| config.set_value(key, &value))?;
            println_i18n!("config-set-success");
        }
        Some(("unset", sub_m)) => {
            let key = sub_m.get_one::<String>("key").unwrap();
            config::Config::modify(|config| config.unset_value(key))?;
            println_i18n!("config-unset-success", key = key);
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
                            (None, _, _) => t!("config-key-not-set-value").to_string(),
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
        _ => return Err(terr!("error-invalid-config-subcommand").into()),
    }
    Ok(())
}
