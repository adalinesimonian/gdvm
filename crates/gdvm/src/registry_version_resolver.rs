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

use anyhow::{Result, anyhow};

use crate::host::HostPlatform;
use crate::i18n::I18n;
use crate::registry::{registry_arch_key, registry_os_key};
use crate::releases::ReleaseCatalog;
use crate::t;
use crate::version_utils::{
    GodotVersion, GodotVersionDeterminate, GodotVersionDeterminateVecExt, Variant,
};

/// Provides an API for resolving Godot versions against installed and available releases.
pub struct RegistryVersionResolver<'a> {
    catalog: &'a ReleaseCatalog,
    i18n: &'a I18n,
    host: HostPlatform,
}

#[derive(Debug, Clone)]
pub enum ResolveMode<'a> {
    Installed {
        installed: &'a [GodotVersionDeterminate],
    },
    Available {
        use_cache_only: bool,
    },
    AutoInstall,
}

#[derive(Debug, Clone)]
pub struct ResolveRequest<'a> {
    pub query: GodotVersion,
    pub variant: Option<String>,
    /// The registry this request targets. `None` for gdvm's own registry.
    pub registry: Option<String>,
    pub include_pre: bool,
    pub mode: ResolveMode<'a>,
}

#[derive(Debug)]
pub enum ResolveOutcome {
    Candidates(Vec<GodotVersionDeterminate>),
    Determinate(GodotVersionDeterminate),
    NotFound,
}

impl<'a> ResolveRequest<'a> {
    pub fn installed(query: GodotVersion, installed: &'a [GodotVersionDeterminate]) -> Self {
        Self {
            query,
            variant: None,
            registry: None,
            include_pre: false,
            mode: ResolveMode::Installed { installed },
        }
    }

    pub fn installed_with_variant(
        query: GodotVersion,
        variant: Option<String>,
        installed: &'a [GodotVersionDeterminate],
    ) -> Self {
        Self {
            query,
            variant,
            registry: None,
            include_pre: false,
            mode: ResolveMode::Installed { installed },
        }
    }

    pub fn available(query: GodotVersion, variant: Option<String>, use_cache_only: bool) -> Self {
        Self {
            query,
            variant,
            registry: None,
            include_pre: false,
            mode: ResolveMode::Available { use_cache_only },
        }
    }

    pub fn auto_install(query: GodotVersion, variant: Option<String>) -> Self {
        Self {
            query,
            variant,
            registry: None,
            include_pre: false,
            mode: ResolveMode::AutoInstall,
        }
    }
}

impl<'a> RegistryVersionResolver<'a> {
    pub fn new(catalog: &'a ReleaseCatalog, i18n: &'a I18n, host: HostPlatform) -> Self {
        Self {
            catalog,
            i18n,
            host,
        }
    }

    pub async fn resolve(&self, request: ResolveRequest<'_>) -> Result<ResolveOutcome> {
        match request.mode {
            ResolveMode::Installed { installed } => {
                let mut matches: Vec<_> = installed
                    .iter()
                    .filter(|&v| request.query.matches(v))
                    .cloned()
                    .collect();
                matches.sort_by_version();
                Ok(ResolveOutcome::Candidates(matches))
            }
            ResolveMode::Available { use_cache_only } => {
                let resolved = self
                    .resolve_available_impl(
                        &request.query,
                        request.variant.as_deref(),
                        request.include_pre,
                        use_cache_only,
                    )
                    .await?;
                Ok(resolved
                    .map(ResolveOutcome::Determinate)
                    .unwrap_or(ResolveOutcome::NotFound))
            }
            ResolveMode::AutoInstall => {
                let resolved = self
                    .resolve_for_auto_install_impl(
                        &request.query,
                        request.variant.as_deref(),
                        request.include_pre,
                    )
                    .await?;
                Ok(resolved
                    .map(ResolveOutcome::Determinate)
                    .unwrap_or(ResolveOutcome::NotFound))
            }
        }
    }

    /// Return installed versions matching a query, sorted newest-first.
    pub async fn resolve_installed(
        &self,
        query: &GodotVersion,
        installed: &[GodotVersionDeterminate],
    ) -> Vec<GodotVersionDeterminate> {
        match self
            .resolve(ResolveRequest::installed(query.clone(), installed))
            .await
            .expect("installed resolution cannot fail")
        {
            ResolveOutcome::Candidates(matches) => matches,
            _ => Vec::new(),
        }
    }

    /// Resolve an available version, preferring latest stable when present.
    pub async fn resolve_available(
        &self,
        query: &GodotVersion,
        variant: Option<&str>,
        include_pre: bool,
        use_cache_only: bool,
    ) -> Result<Option<GodotVersionDeterminate>> {
        let mut request = ResolveRequest::available(
            query.clone(),
            variant.map(|s| s.to_string()),
            use_cache_only,
        );
        request.include_pre = include_pre;
        match self.resolve(request).await {
            Ok(ResolveOutcome::Determinate(gv)) => Ok(Some(gv)),
            Ok(ResolveOutcome::NotFound) => Ok(None),
            Ok(ResolveOutcome::Candidates(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Resolves auto-install requests, preferring latest stable when present.
    pub async fn resolve_for_auto_install(
        &self,
        query: &GodotVersion,
        variant: Option<&str>,
        include_pre: bool,
    ) -> Result<GodotVersionDeterminate> {
        let mut request =
            ResolveRequest::auto_install(query.clone(), variant.map(|s| s.to_string()));
        request.include_pre = include_pre;
        match self.resolve(request).await? {
            ResolveOutcome::Determinate(gv) => Ok(gv),
            ResolveOutcome::NotFound => Err(anyhow!(t!(self.i18n, "error-version-not-found"))),
            ResolveOutcome::Candidates(_) => unreachable!("auto-install never yields candidates"),
        }
    }

    pub async fn latest_stable(&self) -> Result<GodotVersionDeterminate> {
        let stable = GodotVersion {
            release_type: Some("stable".to_string()),
            ..Default::default()
        };
        self.latest_stable_from_query(&stable).await
    }

    async fn latest_stable_from_query(
        &self,
        query: &GodotVersion,
    ) -> Result<GodotVersionDeterminate> {
        let releases = self
            .catalog
            .list_releases(Some(query), false, self.i18n)
            .await?;

        releases
            .iter()
            .find(|r| r.release_type == "stable")
            .cloned()
            .ok_or_else(|| anyhow!(t!(self.i18n, "error-no-stable-releases-found")))
    }

    async fn resolve_available_impl(
        &self,
        query: &GodotVersion,
        variant: Option<&str>,
        include_pre: bool,
        use_cache_only: bool,
    ) -> Result<Option<GodotVersionDeterminate>> {
        let releases = self
            .catalog
            .list_releases(Some(query), use_cache_only, self.i18n)
            .await?;

        let mut newest_compatible_pre_release: Option<GodotVersionDeterminate> = None;

        for gv in releases {
            if !self.is_compatible(&gv, variant).await? {
                continue;
            }

            if include_pre {
                return Ok(Some(gv));
            }

            if gv.release_type == "stable" {
                return Ok(Some(gv));
            }

            if newest_compatible_pre_release.is_none() {
                newest_compatible_pre_release = Some(gv);
            }
        }

        Ok(newest_compatible_pre_release)
    }

    async fn resolve_for_auto_install_impl(
        &self,
        query: &GodotVersion,
        variant: Option<&str>,
        include_pre: bool,
    ) -> Result<Option<GodotVersionDeterminate>> {
        if query.is_stable() && query.major.is_none() {
            return self
                .resolve_available_impl(query, variant, include_pre, false)
                .await;
        }

        if query.is_incomplete() {
            return self
                .resolve_available_impl(query, variant, include_pre, false)
                .await;
        }

        let determinate: GodotVersionDeterminate = query.clone().into();
        if self.is_compatible(&determinate, variant).await? {
            Ok(Some(determinate))
        } else {
            Ok(None)
        }
    }

    async fn is_compatible(
        &self,
        gv: &GodotVersionDeterminate,
        variant: Option<&str>,
    ) -> Result<bool> {
        let platforms_per_variant = self
            .catalog
            .platforms_by_variant(&gv.to_remote_str(), self.i18n)
            .await?;

        let variant = Variant::from_option(variant);
        let Some(platforms) = platforms_per_variant.get(variant.as_str()) else {
            return Ok(false);
        };

        let os_key = registry_os_key(self.host);
        let arch_key = registry_arch_key(self.host);
        let exact = format!("{os_key}-{arch_key}");
        let universal = format!("{os_key}-universal");

        Ok(platforms.iter().any(|p| p == &exact || p == &universal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{HostArch, HostOs, HostPlatform};
    use crate::metadata_cache::{CacheStore, RegistryReleasesCache, ReleaseCache};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn i18n() -> I18n {
        I18n::new().expect("i18n init")
    }

    fn catalog_with_tags(tags: &[&str]) -> (ReleaseCatalog, TempDir) {
        let tmp = TempDir::new().expect("tempdir");
        let cache_store = CacheStore::new(tmp.path().join("cache.json"));

        let variants = HashMap::from([
            (
                "default".to_string(),
                vec!["linux-x86_64".to_string(), "linux-universal".to_string()],
            ),
            ("csharp".to_string(), vec!["linux-x86_64".to_string()]),
        ]);

        let releases: Vec<ReleaseCache> = tags
            .iter()
            .map(|tag| ReleaseCache {
                tag_name: (*tag).to_string(),
                variants: Some(variants.clone()),
                source: crate::registry::ReleaseRef::V2 {
                    path: format!("releases/{tag}.json"),
                },
            })
            .collect();

        let last_fetched = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let registry_cache = RegistryReleasesCache {
            last_fetched,
            releases,
        };
        let registry =
            crate::registry::Registry::official(&crate::i18n::I18n::new().expect("i18n init"))
                .expect("registry client");
        cache_store
            .save_registry_cache(&registry.cache_key(), &registry_cache)
            .expect("write cache");

        (ReleaseCatalog::new(registry, cache_store), tmp)
    }

    #[tokio::test]
    async fn resolve_available_prefers_stable() {
        let (catalog, _tmp) = catalog_with_tags(&["4.2-rc1", "4.2-stable"]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: Some(4),
            minor: Some(2),
            patch: None,
            subpatch: None,
            release_type: None,
        };

        let resolved = resolver
            .resolve_available(&query, Some("csharp"), false, false)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved.to_remote_str(), "4.2-stable");
    }

    #[tokio::test]
    async fn resolve_available_standard_variant() {
        let (catalog, _tmp) = catalog_with_tags(&["4.2-rc1", "4.2-stable"]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: Some(4),
            minor: Some(2),
            patch: None,
            subpatch: None,
            release_type: None,
        };

        for variant in [None, Some("default")] {
            let resolved = resolver
                .resolve_available(&query, variant, false, false)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(resolved.to_remote_str(), "4.2-stable");
        }
    }

    #[tokio::test]
    async fn resolve_for_auto_install_uses_latest_stable() {
        let (catalog, _tmp) = catalog_with_tags(&["4.1-stable", "4.0-stable"]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: None,
            minor: None,
            patch: None,
            subpatch: None,
            release_type: Some("stable".to_string()),
        };

        let resolved = resolver
            .resolve_for_auto_install(&query, Some("csharp"), false)
            .await
            .unwrap();
        assert_eq!(resolved.to_remote_str(), "4.1-stable");
    }

    #[tokio::test]
    async fn resolve_request_available_reports_not_found() {
        let (catalog, _tmp) = catalog_with_tags(&["4.2-stable"]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let request = ResolveRequest::available(
            GodotVersion {
                major: Some(3),
                minor: Some(6),
                patch: None,
                subpatch: None,
                release_type: None,
            },
            None,
            false,
        );

        let outcome = resolver.resolve(request).await.expect("resolve available");
        assert!(matches!(outcome, ResolveOutcome::NotFound));
    }

    #[tokio::test]
    async fn resolve_installed_sorts_and_filters() {
        let (catalog, _tmp) = catalog_with_tags(&[]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: Some(3),
            minor: None,
            patch: None,
            subpatch: None,
            release_type: None,
        };

        let installed = vec![
            GodotVersionDeterminate {
                major: 3,
                minor: 5,
                patch: 0,
                subpatch: 0,
                release_type: "stable".into(),
            },
            GodotVersionDeterminate {
                major: 4,
                minor: 0,
                patch: 0,
                subpatch: 0,
                release_type: "stable".into(),
            },
            GodotVersionDeterminate {
                major: 3,
                minor: 6,
                patch: 1,
                subpatch: 0,
                release_type: "rc1".into(),
            },
        ];

        let result = resolver
            .resolve(ResolveRequest::installed(query, &installed))
            .await
            .expect("resolve installed");

        match result {
            ResolveOutcome::Candidates(matches) => {
                assert_eq!(matches.len(), 2);
                assert_eq!(matches[0].minor, 6);
                assert_eq!(matches[1].minor, 5);
            }
            _ => panic!("expected candidates"),
        }
    }

    fn host() -> HostPlatform {
        HostPlatform {
            os: HostOs::Linux,
            arch: HostArch::X86_64,
        }
    }
}
