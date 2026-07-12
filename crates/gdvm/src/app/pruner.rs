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

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use super::*;
use crate::artifact_cache::ArtifactCache;
use crate::date_utils::{modified_unix_secs, now_unix_secs};
use crate::fs_utils::dir_size;
use crate::paths::GdvmPaths;
use crate::usage_tracker::{UsageState, UsageTracker};
use crate::version::{Variant, VersionQuery};

/// Options controlling how `Gdvm::prune` behaves.
#[derive(Debug, Clone, Copy, Default)]
pub struct PruneOptions {
    /// Remove all installs and cached archives regardless of age. Installs that
    /// still have an active link are preserved unless `force` is also set.
    pub all: bool,
    /// Ignore links entirely, allowing linked installs to be removed.
    pub force: bool,
    /// Report what would be removed without deleting anything.
    pub dry_run: bool,
}

/// A single asset removed by prune.
#[derive(Debug, Clone, Serialize)]
pub struct PrunedItem {
    /// A user-friendly label for the asset.
    pub label: String,
    /// Approximate bytes freed by removing the asset.
    pub freed_bytes: u64,
}

/// The outcome of a prune operation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PruneReport {
    /// Installs that were removed.
    pub installs: Vec<PrunedItem>,
    /// Cached archives that were removed.
    pub archives: Vec<PrunedItem>,
    /// Number of installs preserved because they still have an active link.
    pub preserved_by_link: usize,
    /// Total approximate bytes freed.
    pub freed_bytes: u64,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

impl PruneReport {
    /// True when nothing was removed.
    pub fn is_empty(&self) -> bool {
        self.installs.is_empty() && self.archives.is_empty()
    }
}

#[derive(Clone, Copy)]
pub struct Pruner<'a> {
    pub(super) paths: &'a GdvmPaths,
    pub(super) artifact_cache: &'a ArtifactCache,
    pub(super) usage_tracker: &'a UsageTracker,
    pub(super) catalogs: Catalogs<'a>,
}

impl<'a> Pruner<'a> {
    fn defaults(&self) -> Defaults<'a> {
        Defaults {
            paths: self.paths,
            usage_tracker: self.usage_tracker,
            catalogs: self.catalogs,
        }
    }

    fn library(&self) -> Library<'a> {
        Library {
            paths: self.paths,
            usage_tracker: self.usage_tracker,
            catalogs: self.catalogs,
        }
    }

    /// Remove installs and cached archives that are no longer needed.
    pub fn prune(&self, max_age_secs: u64, opts: PruneOptions) -> Result<PruneReport> {
        let now = now_unix_secs();
        let state = self.usage_tracker.load()?;

        let default_install_key: Option<String> = self
            .defaults()
            .get_default()
            .ok()
            .flatten()
            .and_then(|def| {
                self.library()
                    .install_key(&def.version, &def.variant, def.registry.as_deref())
                    .ok()
            });

        let protected: HashSet<String> = if opts.force {
            HashSet::new()
        } else {
            self.live_link_install_keys(&state)
        };

        let mut report = PruneReport {
            dry_run: opts.dry_run,
            ..Default::default()
        };

        for (key, path) in self.collect_prunable_installs() {
            if default_install_key.as_deref() == Some(key.as_str()) {
                continue;
            }

            if !opts.force && protected.contains(&key) {
                report.preserved_by_link += 1;
                continue;
            }

            let should_remove = if opts.all {
                true
            } else {
                let last_used = self.effective_install_last_used(&key, &path, &state);
                crate::date_utils::age_secs(now, last_used) >= max_age_secs
            };

            if !should_remove {
                continue;
            }

            if !opts.dry_run {
                let Some(_lock) = crate::locks::Lock::try_acquire(
                    &self.paths.locks(),
                    crate::locks::Resource::Install(&key),
                )?
                else {
                    eprintln_i18n!("prune-skipped-in-use", item = self.install_label(&key));
                    continue;
                };

                let freed = dir_size(&path);
                if let Err(error) = fs::remove_dir_all(&path) {
                    eprintln_i18n!(
                        "prune-skipped-error",
                        item = self.install_label(&key),
                        error = error.to_string()
                    );
                    continue;
                }
                report.freed_bytes += freed;
                report.installs.push(PrunedItem {
                    label: self.install_label(&key),
                    freed_bytes: freed,
                });
            } else {
                let freed = dir_size(&path);
                report.freed_bytes += freed;
                report.installs.push(PrunedItem {
                    label: self.install_label(&key),
                    freed_bytes: freed,
                });
            }
        }

        for path in self.collect_cached_archives() {
            let should_remove = if opts.all {
                true
            } else {
                let last_used = self.effective_archive_last_used(&path, &state);
                crate::date_utils::age_secs(now, last_used) >= max_age_secs
            };

            if !should_remove {
                continue;
            }

            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let freed = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if !opts.dry_run {
                let Some(_lock) = crate::locks::Lock::try_acquire(
                    &self.paths.locks(),
                    crate::locks::Resource::Archive(&label),
                )?
                else {
                    eprintln_i18n!("prune-skipped-in-use", item = label.as_str());
                    continue;
                };

                if let Err(error) = fs::remove_file(&path) {
                    eprintln_i18n!(
                        "prune-skipped-error",
                        item = label.as_str(),
                        error = error.to_string()
                    );
                    continue;
                }
            }
            report.freed_bytes += freed;
            report.archives.push(PrunedItem {
                label,
                freed_bytes: freed,
            });
        }

        if opts.dry_run {
            return Ok(report);
        }

        let installs_dir = self.paths.installs().to_path_buf();
        let cache_dir = self.artifact_cache.dir().to_path_buf();
        self.usage_tracker.update(|state| {
            state.installs.retain(|k, _| installs_dir.join(k).exists());
            state
                .archives
                .retain(|name, _| cache_dir.join(name).exists());
            state
                .links
                .retain(|path_str, rec| self.link_is_live(Path::new(path_str), &rec.install_key));
        })?;

        Ok(report)
    }

    /// The set of install keys that still have at least one symlink.
    fn live_link_install_keys(&self, state: &UsageState) -> HashSet<String> {
        let mut protected = HashSet::new();
        for (path_str, rec) in &state.links {
            if self.link_is_live(Path::new(path_str), &rec.install_key) {
                protected.insert(rec.install_key.clone());
            }
        }
        protected
    }

    /// True when `link_path` is a symlink that resolves into the install
    /// directory identified by `install_key`.
    fn link_is_live(&self, link_path: &Path, install_key: &str) -> bool {
        let Ok(meta) = fs::symlink_metadata(link_path) else {
            return false;
        };
        if !meta.file_type().is_symlink() {
            return false;
        }
        let install_dir = self.paths.installs().join(install_key);
        let (Ok(target), Ok(install_canon)) =
            (fs::canonicalize(link_path), fs::canonicalize(&install_dir))
        else {
            return false;
        };
        target.starts_with(install_canon)
    }

    /// Enumerate every install directory as `(install_key, path)`.
    fn collect_prunable_installs(&self) -> Vec<(String, PathBuf)> {
        let installs = self.paths.installs();
        let mut out = Vec::new();
        let Ok(tops) = fs::read_dir(installs) else {
            return out;
        };
        for top in tops.flatten() {
            if !top.file_type().is_ok_and(|ft| ft.is_dir()) {
                continue;
            }
            let top_name = top.file_name().to_string_lossy().to_string();
            let Ok(mids) = fs::read_dir(top.path()) else {
                continue;
            };
            for mid in mids.flatten() {
                if !mid.file_type().is_ok_and(|ft| ft.is_dir()) {
                    continue;
                }
                let mid_name = mid.file_name().to_string_lossy().to_string();
                if VersionQuery::from_install_str(&mid_name).is_ok() {
                    // Legacy variant/version layout.
                    out.push((format!("{top_name}/{mid_name}"), mid.path()));
                    continue;
                }
                let Ok(leaves) = fs::read_dir(mid.path()) else {
                    continue;
                };
                for leaf in leaves.flatten() {
                    if !leaf.file_type().is_ok_and(|ft| ft.is_dir()) {
                        continue;
                    }
                    let leaf_name = leaf.file_name().to_string_lossy().to_string();
                    if VersionQuery::from_install_str(&leaf_name).is_ok() {
                        out.push((format!("{top_name}/{mid_name}/{leaf_name}"), leaf.path()));
                    }
                }
            }
        }
        out
    }

    /// Enumerate every file in the artifact cache directory.
    fn collect_cached_archives(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(self.artifact_cache.dir()) else {
            return out;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|ft| ft.is_file()) {
                out.push(entry.path());
            }
        }
        out
    }

    /// The most recent recorded use of an install, falling back to the
    /// directory's modification time when no usage is tracked.
    fn effective_install_last_used(&self, key: &str, path: &Path, state: &UsageState) -> u64 {
        if let Some(usage) = state.installs.get(key) {
            usage.last_used
        } else {
            modified_unix_secs(path).unwrap_or(0)
        }
    }

    /// The most recent recorded use of a cached archive, falling back to the
    /// file's modification time when no usage is tracked.
    fn effective_archive_last_used(&self, path: &Path, state: &UsageState) -> u64 {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(usage) = state.archives.get(&name) {
            usage.last_used
        } else {
            modified_unix_secs(path).unwrap_or(0)
        }
    }

    /// Get a user-friendly label for an install key.
    fn install_label(&self, key: &str) -> String {
        let parts: Vec<&str> = key.split('/').collect();
        let (registry, variant, version) = match parts.as_slice() {
            [store, variant, version] => {
                let registry = crate::registry_store::read(&self.paths.installs().join(store))
                    .ok()
                    .flatten()
                    .and_then(|m| {
                        self.library()
                            .display_registry_for_url(&m.url, m.display_name.as_deref())
                    });
                (registry, (*variant).to_string(), (*version).to_string())
            }
            [variant, version] => (None, (*variant).to_string(), (*version).to_string()),
            _ => return key.to_string(),
        };

        match VersionQuery::from_install_str(&version) {
            Ok(gv) => crate::version::display_version(
                &gv.to_resolved().to_display_str(),
                &Variant::from_option(Some(&variant)),
                registry.as_deref(),
            ),
            Err(_) => key.to_string(),
        }
    }
}
