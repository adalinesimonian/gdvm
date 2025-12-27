use anyhow::{Result, anyhow};

use crate::host::HostPlatform;
use crate::i18n::I18n;
use crate::registry::{registry_arch_key, registry_platform_key};
use crate::releases::ReleaseCatalog;
use crate::t_w;
use crate::version_utils::{GodotVersion, GodotVersionDeterminate, GodotVersionDeterminateVecExt};

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
            mode: ResolveMode::Installed { installed },
        }
    }

    pub fn available(query: GodotVersion, use_cache_only: bool) -> Self {
        Self {
            query,
            mode: ResolveMode::Available { use_cache_only },
        }
    }

    pub fn auto_install(query: GodotVersion) -> Self {
        Self {
            query,
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

    fn apply_csharp(
        &self,
        query: &GodotVersion,
        mut selected: GodotVersionDeterminate,
    ) -> GodotVersionDeterminate {
        if let Some(is_csharp) = query.is_csharp {
            selected.is_csharp = Some(is_csharp);
        }
        selected
    }

    pub fn resolve(&self, request: ResolveRequest<'_>) -> Result<ResolveOutcome> {
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
                let resolved = self.resolve_available_impl(&request.query, use_cache_only)?;
                Ok(resolved
                    .map(ResolveOutcome::Determinate)
                    .unwrap_or(ResolveOutcome::NotFound))
            }
            ResolveMode::AutoInstall => {
                let resolved = self.resolve_for_auto_install_impl(&request.query)?;
                Ok(resolved
                    .map(ResolveOutcome::Determinate)
                    .unwrap_or(ResolveOutcome::NotFound))
            }
        }
    }

    /// Return installed versions matching a query, sorted newest-first.
    pub fn resolve_installed(
        &self,
        query: &GodotVersion,
        installed: &[GodotVersionDeterminate],
    ) -> Vec<GodotVersionDeterminate> {
        match self
            .resolve(ResolveRequest::installed(query.clone(), installed))
            .expect("installed resolution cannot fail")
        {
            ResolveOutcome::Candidates(matches) => matches,
            _ => Vec::new(),
        }
    }

    /// Resolve an available version, preferring latest stable when present.
    pub fn resolve_available(
        &self,
        query: &GodotVersion,
        use_cache_only: bool,
    ) -> Result<Option<GodotVersionDeterminate>> {
        match self.resolve(ResolveRequest::available(query.clone(), use_cache_only)) {
            Ok(ResolveOutcome::Determinate(gv)) => Ok(Some(gv)),
            Ok(ResolveOutcome::NotFound) => Ok(None),
            Ok(ResolveOutcome::Candidates(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Resolves auto-install requests, preferring latest stable when present.
    pub fn resolve_for_auto_install(
        &self,
        query: &GodotVersion,
    ) -> Result<GodotVersionDeterminate> {
        match self.resolve(ResolveRequest::auto_install(query.clone()))? {
            ResolveOutcome::Determinate(gv) => Ok(gv),
            ResolveOutcome::NotFound => Err(anyhow!(t_w!(self.i18n, "error-version-not-found"))),
            ResolveOutcome::Candidates(_) => unreachable!("auto-install never yields candidates"),
        }
    }

    pub fn latest_stable(&self) -> Result<GodotVersionDeterminate> {
        let stable = GodotVersion {
            release_type: Some("stable".to_string()),
            ..Default::default()
        };
        self.latest_stable_from_query(&stable)
    }

    fn latest_stable_from_query(&self, query: &GodotVersion) -> Result<GodotVersionDeterminate> {
        let releases = self.catalog.list_releases(Some(query), false, self.i18n)?;

        releases
            .iter()
            .find(|r| r.release_type == "stable")
            .cloned()
            .ok_or_else(|| anyhow!(t_w!(self.i18n, "error-no-stable-releases-found")))
    }

    fn resolve_available_impl(
        &self,
        query: &GodotVersion,
        use_cache_only: bool,
    ) -> Result<Option<GodotVersionDeterminate>> {
        let releases = self
            .catalog
            .list_releases(Some(query), use_cache_only, self.i18n)?;

        let needs_csharp = query.is_csharp.unwrap_or(false);
        let compatible: Vec<_> = releases
            .into_iter()
            .filter(|gv| self.is_compatible(gv, needs_csharp).unwrap_or(false))
            .collect();

        let latest_stable = compatible
            .iter()
            .find(|r| r.release_type == "stable")
            .cloned();

        Ok(latest_stable
            .or_else(|| compatible.into_iter().next())
            .map(|gv| self.apply_csharp(query, gv)))
    }

    fn resolve_for_auto_install_impl(
        &self,
        query: &GodotVersion,
    ) -> Result<Option<GodotVersionDeterminate>> {
        if query.is_stable() && query.major.is_none() {
            return self.resolve_available_impl(query, false);
        }

        if query.is_incomplete() {
            return self.resolve_available_impl(query, false);
        }

        let determinate: GodotVersionDeterminate = query.clone().into();
        if self.is_compatible(&determinate, query.is_csharp.unwrap_or(false))? {
            Ok(Some(self.apply_csharp(query, determinate)))
        } else {
            Ok(None)
        }
    }

    fn is_compatible(&self, gv: &GodotVersionDeterminate, needs_csharp: bool) -> Result<bool> {
        let caps = self
            .catalog
            .capabilities_for(&gv.to_remote_str(), self.i18n)?;

        if needs_csharp && !caps.has_csharp {
            return Ok(false);
        }

        let platform_key = registry_platform_key(self.host, needs_csharp);
        let arch_key = registry_arch_key(self.host);
        let exact = format!("{platform_key}-{arch_key}");
        let universal = format!("{platform_key}-universal");

        Ok(caps
            .platforms
            .iter()
            .any(|p| p == &exact || p == &universal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{HostArch, HostOs, HostPlatform};
    use crate::metadata_cache::{
        CacheStore, FullCache, RegistryReleasesCache, ReleaseCache, ReleaseCapabilitiesCache,
        ReleaseCapabilitiesEntry,
    };
    use tempfile::TempDir;

    fn i18n() -> I18n {
        I18n::new(80).expect("i18n init")
    }

    fn catalog_with_tags(tags: &[&str]) -> (ReleaseCatalog, TempDir) {
        let tmp = TempDir::new().expect("tempdir");
        let cache_store = CacheStore::new(tmp.path().join("cache.json"));

        let releases: Vec<ReleaseCache> = tags
            .iter()
            .enumerate()
            .map(|(id, tag)| ReleaseCache {
                id: id as u64,
                tag_name: (*tag).to_string(),
            })
            .collect();

        let last_fetched = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let full = FullCache {
            gdvm: crate::metadata_cache::GdvmCache {
                last_update_check: 0,
                new_version: None,
                new_major_version: None,
            },
            godot_registry: RegistryReleasesCache {
                last_fetched,
                releases,
            },
            release_capabilities: ReleaseCapabilitiesCache {
                last_fetched,
                entries: tags
                    .iter()
                    .map(|tag| ReleaseCapabilitiesEntry {
                        tag_name: (*tag).to_string(),
                        has_csharp: true,
                        platforms: vec![
                            "linux-csharp-x86_64".to_string(),
                            "linux-x86_64".to_string(),
                            "linux-universal".to_string(),
                        ],
                    })
                    .collect(),
            },
        };
        cache_store
            .save_registry_cache(&full.godot_registry)
            .expect("write cache");
        cache_store
            .save_capabilities_cache(&full.release_capabilities)
            .expect("write caps cache");

        let registry = crate::registry::Registry::new().expect("registry client");
        (ReleaseCatalog::new(registry, cache_store), tmp)
    }

    #[test]
    fn resolve_available_prefers_stable() {
        let (catalog, _tmp) = catalog_with_tags(&["4.2-rc1", "4.2-stable"]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: Some(4),
            minor: Some(2),
            patch: None,
            subpatch: None,
            release_type: None,
            is_csharp: Some(true),
        };

        let resolved = resolver.resolve_available(&query, false).unwrap().unwrap();
        assert_eq!(resolved.to_remote_str(), "4.2-stable");
        assert_eq!(resolved.is_csharp, Some(true));
    }

    #[test]
    fn resolve_for_auto_install_uses_latest_stable() {
        let (catalog, _tmp) = catalog_with_tags(&["4.1-stable", "4.0-stable"]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: None,
            minor: None,
            patch: None,
            subpatch: None,
            release_type: Some("stable".to_string()),
            is_csharp: Some(true),
        };

        let resolved = resolver.resolve_for_auto_install(&query).unwrap();
        assert_eq!(resolved.to_remote_str(), "4.1-stable");
        assert_eq!(resolved.is_csharp, Some(true));
    }

    #[test]
    fn resolve_request_available_reports_not_found() {
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
                is_csharp: None,
            },
            false,
        );

        let outcome = resolver.resolve(request).expect("resolve available");
        assert!(matches!(outcome, ResolveOutcome::NotFound));
    }

    #[test]
    fn resolve_installed_sorts_and_filters() {
        let (catalog, _tmp) = catalog_with_tags(&[]);
        let intl = i18n();
        let resolver = RegistryVersionResolver::new(&catalog, &intl, host());
        let query = GodotVersion {
            major: Some(3),
            minor: None,
            patch: None,
            subpatch: None,
            release_type: None,
            is_csharp: None,
        };

        let installed = vec![
            GodotVersionDeterminate {
                major: 3,
                minor: 5,
                patch: 0,
                subpatch: 0,
                release_type: "stable".into(),
                is_csharp: None,
            },
            GodotVersionDeterminate {
                major: 4,
                minor: 0,
                patch: 0,
                subpatch: 0,
                release_type: "stable".into(),
                is_csharp: None,
            },
            GodotVersionDeterminate {
                major: 3,
                minor: 6,
                patch: 1,
                subpatch: 0,
                release_type: "rc1".into(),
                is_csharp: None,
            },
        ];

        let result = resolver
            .resolve(ResolveRequest::installed(query, &installed))
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
