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

use crate::i18n::I18n;
use crate::{terr, ui};

type MigrationFn = fn(&Path) -> Result<()>;

struct Migration {
    version: u32,
    run: MigrationFn,
}

macro_rules! define_migrations {
    ( $( $ver:literal => |$path:ident, $i18n:ident| $body:block ),* $(,)? ) => {
        const MIGRATIONS: &[Migration] = &[
            $( Migration { version: $ver, run: |$path| -> Result<()> {
                #[allow(unused_variables)]
                let $i18n = I18n::get();
                $body
            } }, )*
        ];
    };
}

define_migrations! {
    1 => |_base_path, i18n| {
        // Shims are now installed by post_upgrade before migrations run.
        Ok(())
    },
    2 => |base_path, i18n| {
        // Migrate install directories from flat layout to variant subfolder
        // layout, so every build lives under a variant dir:
        //
        //   4.4.1-stable-csharp/  moves to  csharp/4.4.1-stable/
        //   4.4.1-stable/         moves to  default/4.4.1-stable/
        //
        // Create a symlink from the old path to the new path for backward
        // compatibility with existing links. Silently warn on symlink failure.
        let installs = base_path.join("installs");

        if !installs.is_dir() {
            return Ok(());
        }

        let entries: Vec<PathBuf> = fs::read_dir(&installs)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Only process real directories, not symlinks from a previous migration.
                e.metadata().map(|m| m.is_dir()).unwrap_or(false)
                    && !e.file_type().map(|t| t.is_symlink()).unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        for old_path in entries {
            let dir_name = match old_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let new_path = if let Some(base) = dir_name.strip_suffix("-csharp") {
                installs.join("csharp").join(base)
            } else if crate::version::VersionQuery::from_install_str(&dir_name).is_ok() {
                installs
                    .join(crate::version::Variant::DEFAULT)
                    .join(&dir_name)
            } else {
                // Not a recognized install, ignore.
                continue;
            };

            if new_path.exists() {
                continue;
            }

            if let Some(parent) = new_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Don't error if the user ran gdvm twice and the other instance
            // already got rid of the path.
            match fs::rename(&old_path, &new_path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            }

            if let Err(err) = create_symlink(&new_path, &old_path) {
                ui::report_error(&terr!(
                    "error-link-symlink",
                    target = &old_path.to_string_lossy().to_string(),
                    link = &new_path.to_string_lossy().to_string(),
                ).with_source(err).into());
            }
        }

        Ok(())
    },
    3 => |base_path, i18n| {
        // Migrate install stores so that they're keyed by registry URL.
        let installs = base_path.join("installs");
        if !installs.is_dir() {
            return Ok(());
        }

        let dst_store = installs.join(crate::registry::official_store_dir_name());

        let tops: Vec<PathBuf> = fs::read_dir(&installs)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Ignore symlinks from v2 migration.
                e.metadata().map(|m| m.is_dir()).unwrap_or(false)
                    && !e.file_type().map(|t| t.is_symlink()).unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        let mut migrated_any = false;
        for top in tops {
            if top.join(crate::registry_store::STORE_META_FILE).is_file() {
                continue;
            }
            if !dir_holds_versions(&top) {
                continue;
            }
            let Some(variant) = top.file_name().map(|n| n.to_os_string()) else {
                continue;
            };
            let dst_variant = dst_store.join(&variant);
            move_versions_into(&top, &dst_variant)?;
            finish_legacy_top(&top, &dst_variant);
            migrated_any = true;
        }

        if migrated_any {
            crate::registry_store::upsert(
                &dst_store,
                crate::registry::OFFICIAL_BASE_URL,
                None,
                None,
            )?;
        }

        Ok(())
    },
    4 => |base_path, _i18n| {
        // Remove the legacy `github_token` value from config.toml to avoid
        // secrets sticking around.
        edit_config(base_path, |document| {
            Ok(document.remove("github_token").is_some())
        })
    },
    5 => |base_path, _i18n| {
        // Move old keys into their new namespaces.
        edit_config(base_path, |document| {
            let mut changed = false;
            for (old_key, new_key) in [
                ("prune_max_age_days", "prune.max-age-days"),
                ("trusted_registries", "trusted-registries"),
            ] {
                if document_has(document, new_key) {
                    changed |= document.remove(old_key).is_some();
                    continue;
                }

                let Some((key, item)) = document.as_table_mut().remove_entry(old_key) else {
                    continue;
                };

                move_into(document, new_key, key, item);

                changed = true;
            }
            Ok(changed)
        })
    }
}

fn edit_config(
    base_path: &Path,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<bool>,
) -> Result<()> {
    let config_path = base_path.join("config.toml");
    if !config_path.is_file() {
        return Ok(());
    }

    let contents = fs::read_to_string(&config_path)?;
    let Ok(mut document) = contents.parse::<toml_edit::DocumentMut>() else {
        return Ok(());
    };

    if edit(&mut document)? {
        crate::fs_utils::atomic_write(&config_path, &document.to_string())?;
    }

    Ok(())
}

fn document_has(document: &toml_edit::DocumentMut, path: &str) -> bool {
    let mut segments = path.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    let Some(mut item) = document.get(first) else {
        return false;
    };
    for segment in segments {
        match item.get(segment) {
            Some(next) => item = next,
            None => return false,
        }
    }
    true
}

fn move_into(
    document: &mut toml_edit::DocumentMut,
    path: &str,
    old_key: toml_edit::Key,
    item: toml_edit::Item,
) {
    let mut segments: Vec<&str> = path.split('.').collect();
    let Some(leaf) = segments.pop() else {
        return;
    };

    let prefix = old_key.leaf_decor().prefix().cloned();
    let mut table = document.as_table_mut();

    for segment in &segments {
        let is_new = !table.contains_key(segment);
        let entry = table[*segment].or_insert(toml_edit::table());
        let Some(next) = entry.as_table_mut() else {
            return;
        };

        if is_new {
            next.set_dotted(true);
        }

        table = next;
    }

    table[leaf] = item;

    if let Some(prefix) = prefix
        && let Some(mut key) = table.key_mut(leaf)
    {
        key.leaf_decor_mut().set_prefix(prefix);
    }
}

pub fn run_migrations(base_path: &Path) -> Result<()> {
    fs::create_dir_all(base_path)?;

    let version_file = base_path.join("data_version");

    let latest = MIGRATIONS.iter().map(|m| m.version).max().unwrap_or(0);
    if read_data_version(&version_file)? >= latest {
        return Ok(());
    }

    // If there was a lock for migrations held by another process, wait for it
    // to finish and then re-read the version file to see if we still need to
    // run migrations.
    let _lock =
        crate::locks::Lock::acquire(&base_path.join("locks"), crate::locks::Resource::Migrations)?;

    let original_version = read_data_version(&version_file)?;
    let mut current = original_version;

    for mig in MIGRATIONS {
        if current < mig.version {
            (mig.run)(base_path)?;
            current = mig.version;
        }
    }

    if original_version == current {
        return Ok(());
    }

    write_data_version(&version_file, current)?;

    Ok(())
}

fn read_data_version(path: &Path) -> Result<u32> {
    Ok(crate::fs_utils::read_marker_line(path)?
        .and_then(|line| line.parse::<u32>().ok())
        .unwrap_or(0))
}

fn write_data_version(path: &Path, version: u32) -> Result<()> {
    crate::fs_utils::write_marker_line(path, &version.to_string())
}

#[cfg(target_family = "unix")]
fn create_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(target_family = "windows")]
fn create_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

/// True when a directory's immediate children include version directories.
fn dir_holds_versions(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            && crate::version::VersionQuery::from_install_str(&e.file_name().to_string_lossy())
                .is_ok()
    })
}

/// Move every `{version}` directory from `src` into `dst`.
fn move_versions_into(src: &Path, dst: &Path) -> Result<()> {
    let Ok(entries) = fs::read_dir(src) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        let target = dst.join(entry.file_name());
        if target.exists() {
            continue;
        }
        fs::create_dir_all(dst)?;
        match fs::rename(entry.path(), &target) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Remove a legacy top directory and replace it with a symlink to its new
/// location.
fn finish_legacy_top(old: &Path, new_target: &Path) {
    let _ = fs::remove_dir(old);
    if old.exists() {
        return;
    }
    if let Err(err) = create_symlink(new_target, old) {
        ui::report_error(
            &terr!(
                "error-link-symlink",
                target = &old.to_string_lossy().to_string(),
                link = &new_target.to_string_lossy().to_string(),
            )
            .with_source(err)
            .into(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::run_migrations;

    fn make_version(base: &Path, rel: &str) {
        let dir = base.join("installs").join(rel);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Godot_v_bin"), b"bin").unwrap();
    }

    #[test]
    fn migration_v3_rekeys_official_installs_by_url() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        make_version(base, "default/4.4-stable");
        make_version(base, "csharp/4.3-stable");

        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(
            base.join("installs/default/4.4-stable"),
            base.join("installs/4.4-stable"),
        )
        .unwrap();

        fs::write(base.join("data_version"), "2\n").unwrap();

        run_migrations(base).unwrap();

        let store = base
            .join("installs")
            .join(crate::registry::official_store_dir_name());

        assert!(store.join("default/4.4-stable/Godot_v_bin").is_file());
        assert!(store.join("csharp/4.3-stable/Godot_v_bin").is_file());

        let meta = crate::registry_store::read(&store).unwrap().unwrap();
        assert_eq!(
            crate::registry::normalize_url(&meta.url),
            crate::registry::normalize_url(crate::registry::OFFICIAL_BASE_URL)
        );

        #[cfg(target_family = "unix")]
        {
            let old_default = base.join("installs/default");
            assert!(
                fs::symlink_metadata(&old_default)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert!(old_default.join("4.4-stable/Godot_v_bin").is_file());
        }

        let dv = fs::read_to_string(base.join("data_version")).unwrap();
        assert!(
            dv.lines()
                .filter_map(|l| l.trim().parse::<u32>().ok())
                .any(|v| v >= 3)
        );
    }

    #[test]
    fn migration_v3_does_not_renest_an_existing_store() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        make_version(base, "default/4.4-stable");
        fs::write(base.join("data_version"), "2\n").unwrap();
        run_migrations(base).unwrap();

        fs::write(base.join("data_version"), "2\n").unwrap();
        run_migrations(base).unwrap();

        let official = crate::registry::official_store_dir_name();
        let store = base.join("installs").join(&official);
        assert!(store.join("default/4.4-stable/Godot_v_bin").is_file());
        assert!(!store.join(&official).exists());
    }

    #[test]
    fn migration_v4_scrubs_legacy_github_token() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        fs::write(
            base.join("config.toml"),
            "github_token = \"super-secret\"\nprune_max_age_days = 12\n",
        )
        .unwrap();
        fs::write(base.join("data_version"), "3\n").unwrap();

        run_migrations(base).unwrap();

        let contents = fs::read_to_string(base.join("config.toml")).unwrap();
        assert!(
            !contents.contains("github_token"),
            "github_token must be removed, got: {contents}"
        );
        assert!(
            !contents.contains("super-secret"),
            "token value must be removed, got: {contents}"
        );
        assert!(contents.contains("max-age-days = 12"));

        let dv = fs::read_to_string(base.join("data_version")).unwrap();
        assert!(dv.lines().any(|l| l.trim() == "5"));
    }

    #[test]
    fn migration_v4_no_config_is_ok() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        fs::write(base.join("data_version"), "3\n").unwrap();

        run_migrations(base).unwrap();

        assert!(!base.join("config.toml").exists());
        let dv = fs::read_to_string(base.join("data_version")).unwrap();
        assert!(dv.lines().any(|l| l.trim() == "5"));
    }

    #[test]
    fn migration_v5_moves_keys_under_their_namespace() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        fs::write(
            base.join("config.toml"),
            "# Blah blah blah blah.\nprune_max_age_days = 365\ntrusted_registries = [\"https://example.com/reg\"]\n\n\
             # Work registry.\n[registries.work]\nurl = \"https://example.com/godot\"\n",
        )
        .unwrap();
        fs::write(base.join("data_version"), "4\n").unwrap();

        run_migrations(base).unwrap();

        let contents = fs::read_to_string(base.join("config.toml")).unwrap();

        assert!(!contents.contains("prune_max_age_days"));
        assert!(!contents.contains("trusted_registries"));
        assert!(contents.contains("prune.max-age-days = 365"));
        assert!(contents.contains("trusted-registries = [\"https://example.com/reg\"]"));
        assert!(contents.contains("# Blah blah blah blah."));
        assert!(contents.contains("# Work registry."));
        assert!(contents.contains("[registries.work]"));
    }

    #[test]
    fn migration_v5_leaves_an_already_migrated_config_alone() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let original = "[prune]\nmax-age-days = 30\n";

        fs::write(base.join("config.toml"), original).unwrap();
        fs::write(base.join("data_version"), "4\n").unwrap();

        run_migrations(base).unwrap();

        assert_eq!(
            fs::read_to_string(base.join("config.toml")).unwrap(),
            original
        );
    }

    #[test]
    fn migration_v5_prefers_the_new_key_when_both_are_present() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        fs::write(
            base.join("config.toml"),
            "prune_max_age_days = 365\n\n[prune]\nmax-age-days = 30\n",
        )
        .unwrap();
        fs::write(base.join("data_version"), "4\n").unwrap();

        run_migrations(base).unwrap();

        let contents = fs::read_to_string(base.join("config.toml")).unwrap();

        assert!(!contents.contains("prune_max_age_days"));
        assert!(contents.contains("max-age-days = 30"));
        assert!(!contents.contains("365"));
    }

    #[test]
    fn migration_v5_leaves_a_malformed_config_alone() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let original = "not toml =====\nprune_max_age_days = 365\n";

        fs::write(base.join("config.toml"), original).unwrap();
        fs::write(base.join("data_version"), "4\n").unwrap();

        run_migrations(base).unwrap();

        assert_eq!(
            fs::read_to_string(base.join("config.toml")).unwrap(),
            original
        );
    }
}
