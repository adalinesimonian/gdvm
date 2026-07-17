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

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::ArgMatches;
use gdvm::app::Gdvm;
use gdvm::config::{self};
use gdvm::{println_i18n, registry, t, terr};

/// Download `url` to a unique temporary file and return its path.
async fn download_to_temp(url: &str) -> Result<PathBuf> {
    let tmp = tempfile::Builder::new()
        .prefix("gdvm-add-build-")
        .suffix(".tmp")
        .tempfile()?;
    let mut file = tokio::fs::File::from_std(tmp.as_file().try_clone()?);
    gdvm::download_utils::download_to_file(url, &mut file, url).await?;
    drop(file);
    let (_file, path) = tmp.keep()?;
    Ok(path)
}

/// Error if an explicit override disagrees with the values measured from the
/// downloaded or local artifact.
fn verify_overrides(
    sha512: &Option<String>,
    size: Option<u64>,
    actual_sha: &str,
    actual_size: u64,
) -> Result<()> {
    if let Some(s) = sha512
        && !s.eq_ignore_ascii_case(actual_sha)
    {
        return Err(terr!(
            "registry-build-sha-mismatch",
            expected = s.clone(),
            actual = actual_sha.to_string()
        ));
    }
    if let Some(s) = size
        && s != actual_size
    {
        return Err(terr!(
            "registry-build-size-mismatch",
            expected = s.to_string(),
            actual = actual_size.to_string()
        ));
    }
    Ok(())
}

/// Resolve the SHA-512 and size to record for an `add-build`.
async fn resolve_build_integrity(
    store: bool,
    file: Option<&Path>,
    url: Option<&str>,
    sha512: Option<String>,
    size: Option<u64>,
) -> Result<(Option<String>, Option<u64>)> {
    if store {
        if sha512.is_some() || size.is_some() {
            gdvm::ui::warn(t!("registry-build-warn-explicit-store"));
        }
        return Ok((sha512, size));
    }

    let Some(url) = url else {
        return Ok((sha512, size));
    };

    if let Some(file) = file {
        gdvm::ui::warn(t!("registry-build-warn-local-hash", url = url));
        let (computed_sha, computed_size) = registry::publish::hash_file(file)?;
        verify_overrides(&sha512, size, &computed_sha, computed_size)?;
        return Ok((
            Some(sha512.unwrap_or(computed_sha)),
            Some(size.unwrap_or(computed_size)),
        ));
    }

    if let (Some(sha512), Some(size)) = (sha512.clone(), size) {
        gdvm::ui::warn(t!("registry-build-warn-unverified"));
        return Ok((Some(sha512), Some(size)));
    }

    let tmp = download_to_temp(url).await?;
    let resolved = (|| {
        let (computed_sha, computed_size) = registry::publish::hash_file(&tmp)?;
        verify_overrides(&sha512, size, &computed_sha, computed_size)?;
        Ok((Some(computed_sha), Some(computed_size)))
    })();
    let _ = fs::remove_file(&tmp);
    resolved
}

/// Handle the 'registry' subcommand
pub(crate) async fn sub_registry(gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("add", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            let url = sub_m.get_one::<String>("url").unwrap();
            config::Config::modify(|config| config.add_registry(name, url))?;
            println_i18n!(
                "registry-added",
                registry = name.as_str(),
                url = url.as_str()
            );
        }
        Some(("remove", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            config::Config::modify(|config| config.remove_registry(name))?;
            println_i18n!("registry-removed", registry = name.as_str());
        }
        Some(("list", sub_m)) => {
            let registries = gdvm.catalogs().registry_list();

            if super::format::OutputFormat::from_matches(sub_m) == super::format::OutputFormat::Json
            {
                #[derive(serde::Serialize)]
                struct RegistryEntry {
                    name: String,
                    url: String,
                    official: bool,
                }
                let entries: Vec<RegistryEntry> = registries
                    .into_iter()
                    .map(|info| RegistryEntry {
                        name: info.name,
                        url: info.url,
                        official: info.is_official,
                    })
                    .collect();
                return super::format::print_json(&entries);
            }

            println_i18n!("registry-list-header");
            for info in registries {
                let mut tags = Vec::new();
                if info.is_official {
                    tags.push(t!("registry-tag-official"));
                }
                let suffix = if tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", tags.join(", "))
                };
                println!("- {} ({}){suffix}", info.name, info.url);
            }
        }
        Some(("refresh", sub_m)) => {
            match sub_m.get_one::<String>("name") {
                Some(name) => gdvm.catalogs().refresh_registry_cache(Some(name)).await?,
                None => gdvm.catalogs().refresh_all_registry_caches().await?,
            }
            gdvm::ui::milestone(gdvm::t!("status-refreshed"), gdvm::t!("subject-cache"));
        }
        Some(("init", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let name = sub_m.get_one::<String>("name").map(|s| s.as_str());
            let name = registry::publish::init(&dir, name)?;
            println_i18n!(
                "registry-init-success",
                name = name,
                path = dir.display().to_string()
            );
        }
        Some(("add-build", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let file = sub_m.get_one::<String>("file").map(PathBuf::from);
            let store = sub_m.get_flag("store");
            let url = sub_m.get_one::<String>("url").cloned();
            let sha512 = sub_m.get_one::<String>("sha512").cloned();
            let size = sub_m.get_one::<u64>("size").copied();

            let (sha512, size) =
                resolve_build_integrity(store, file.as_deref(), url.as_deref(), sha512, size)
                    .await?;
            let args = registry::publish::AddBuild {
                version: sub_m.get_one::<String>("version").unwrap().clone(),
                variant: sub_m.get_one::<String>("variant").cloned(),
                platform: sub_m.get_one::<String>("platform").unwrap().clone(),
                file,
                store,
                url,
                sha512,
                size,
            };
            let version = args.version.clone();
            let platform = args.platform.clone();
            registry::publish::add_build(&dir, &args)?;
            println_i18n!(
                "registry-build-added",
                version = version,
                platform = platform
            );
        }
        Some(("remove-build", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let args = registry::publish::RemoveBuild {
                version: sub_m.get_one::<String>("version").unwrap().clone(),
                variant: sub_m.get_one::<String>("variant").cloned(),
                platform: sub_m.get_one::<String>("platform").cloned(),
            };
            let version = args.version.clone();
            registry::publish::remove_build(&dir, &args)?;
            println_i18n!("registry-build-removed", version = version);
        }
        Some(("validate", sub_m)) => {
            let dir = PathBuf::from(sub_m.get_one::<String>("dir").unwrap());
            let report = registry::publish::validate(&dir)?;
            if report.is_valid() {
                println_i18n!("registry-validate-ok", count = report.checked);
            } else {
                let mut message = t!("registry-validate-failed").to_string();
                for error in &report.errors {
                    message.push_str(&format!("\n  - {error}"));
                }
                return Err(anyhow::Error::new(gdvm::error::CodedError::new(
                    "registry-validate-failed",
                    message,
                )));
            }
        }
        _ => return Err(terr!("error-invalid-registry-subcommand")),
    }
    Ok(())
}
