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

use gdvm::config::{self, ConfigKey};
use gdvm::{println_i18n, t, terr};

/// Parse a config key argument from the command line.
fn config_key(sub_m: &clap::ArgMatches) -> anyhow::Result<ConfigKey> {
    sub_m.get_one::<String>("key").unwrap().parse()
}

/// Format a config value for display.
fn display_value(key: ConfigKey, value: Option<String>, show_sensitive: bool) -> String {
    match value {
        Some(_) if key.is_sensitive() && !show_sensitive => "********".to_string(),
        Some(value) => value,
        None => t!("config-key-not-set-value").to_string(),
    }
}

/// Handle the 'config' subcommand
pub(crate) fn sub_config(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let config = config::ConfigFile::load()?.into_config();
    match matches.subcommand() {
        Some(("get", sub_m)) => {
            let key = config_key(sub_m)?;
            let value = config.get_value(key);

            if super::format::OutputFormat::is_json(sub_m) {
                #[derive(serde::Serialize)]
                struct ConfigValue<'a> {
                    key: &'a str,
                    value: Option<String>,
                }

                return super::format::print_json(&ConfigValue {
                    key: key.as_str(),
                    value,
                });
            }

            if let Some(value) = value {
                println!("{value}");
            } else {
                println!("{}", t!("config-key-not-set"));
            }
        }
        Some(("set", sub_m)) => {
            let key = config_key(sub_m)?;
            // If the value argument is not provided, prompt the user.
            let value: String = if let Some(v) = sub_m.get_one::<String>("value") {
                v.clone()
            } else if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                return Err(terr!("error-non-interactive-value", key = key.as_str()).into());
            } else {
                // Build the prompt message from the Fluent bundle.
                let prompt = t!("config-set-prompt", key = key.as_str());
                eprint!("{prompt} ");
                if key.is_sensitive() {
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

            if key.is_sensitive() {
                gdvm::ui::warn(t!("warning-setting-sensitive"));
            }
            config::ConfigFile::modify(|file| file.set_value(key, &value))?;
            println_i18n!("config-set-success");
        }
        Some(("unset", sub_m)) => {
            let key = config_key(sub_m)?;
            config::ConfigFile::modify(|file| {
                file.unset_value(key);
                Ok(())
            })?;
            println_i18n!("config-unset-success", key = key.as_str());
        }
        Some(("list", sub_m)) => {
            let show_sensitive = sub_m.get_flag("show-sensitive");

            let entries: Vec<(ConfigKey, Option<String>)> = if sub_m.get_flag("available") {
                // Every known key, whether set or not.
                ConfigKey::ALL
                    .iter()
                    .map(|&key| (key, config.get_value(key)))
                    .collect()
            } else {
                // Only the keys that are set.
                config
                    .list_set_keys()
                    .into_iter()
                    .map(|(key, value)| (key, Some(value)))
                    .collect()
            };

            for (key, value) in entries {
                println!("{key} = {}", display_value(key, value, show_sensitive));
            }
        }
        _ => return Err(terr!("error-invalid-config-subcommand").into()),
    }
    Ok(())
}
