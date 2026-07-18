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

use anyhow::Result;

use crate::host::HostPlatform;
use crate::registry::{self, BinarySelectionError};
use crate::registry_version_resolver::RegistryVersionResolver;
use crate::releases::{CatalogSet, ReleaseCatalog};
use crate::terr;
use crate::version::{ResolvedVersion, Variant, VersionQuery};

#[derive(Clone, Copy)]
pub struct Catalogs<'a> {
    pub(super) catalogs: &'a CatalogSet,
    pub(super) host: &'a HostPlatform,
}

impl<'a> Catalogs<'a> {
    /// Select a release catalog by registry name. If `registry` is `None`, the
    /// official catalog is returned.
    pub(super) fn catalog(&self, registry: Option<&str>) -> Result<&'a ReleaseCatalog> {
        self.catalogs.catalog(registry)
    }

    /// Fetch available releases with caching.
    pub async fn fetch_available_releases(
        &self,
        registry: Option<&str>,
        filter: &Option<VersionQuery>,
        use_cache_only: bool,
    ) -> Result<Vec<ResolvedVersion>> {
        self.catalog(registry)?
            .list_releases(filter.as_ref(), use_cache_only)
            .await
    }

    /// Refresh the gdvm release cache by re-downloading the registry index.
    pub async fn refresh_cache(&self) -> Result<()> {
        self.catalog(None)?.refresh_cache().await
    }

    /// Refresh the cache for a single registry.
    pub async fn refresh_registry_cache(&self, registry: Option<&str>) -> Result<()> {
        self.catalog(registry)?.refresh_cache().await
    }

    /// Refresh the caches of every configured registry. Stops at the first failure.
    pub async fn refresh_all_registry_caches(&self) -> Result<()> {
        for name in self.catalogs.names() {
            self.catalog(Some(name))?.refresh_cache().await?;
        }
        Ok(())
    }

    /// True when `registry` is the official registry. Also true when `registry` is `None`, since
    /// the official registry is the default.
    pub fn is_official_registry(&self, registry: Option<&str>) -> bool {
        self.catalogs.is_official(registry)
    }

    /// The base URL configured for a registry.
    pub fn registry_base_url(&self, registry: &str) -> Result<String> {
        Ok(self.catalog(Some(registry))?.registry_base_url())
    }

    /// Summaries of all configured registries, official first, marking the default.
    pub fn registry_list(&self) -> Vec<crate::releases::RegistryInfo> {
        self.catalogs.list()
    }

    /// Resolve the Godot version from a string, for an available version
    /// Returns a single version, whichever is the latest that matches the
    /// input.
    /// Accepts full and partial versions.
    pub async fn resolve_available_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
        use_cache_only: bool,
    ) -> Result<Option<ResolvedVersion>>
    where
        T: Into<VersionQuery> + Clone,
    {
        let gv: VersionQuery = gv.clone().into();
        let resolver = RegistryVersionResolver::new(self.catalog(registry)?, *self.host);
        resolver
            .resolve_available(&gv, variant, include_pre, use_cache_only)
            .await
    }

    /// Resolve a query against available releases, returning an error if no
    /// version is found.
    pub async fn resolve_available_or_not_found(
        &self,
        query: &crate::version::VersionQuery,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
        use_cache_only: bool,
    ) -> Result<crate::version::ResolvedVersion> {
        match self
            .resolve_available_version(query, variant, registry, include_pre, use_cache_only)
            .await?
        {
            Some(resolved) => Ok(resolved),
            None => Err(self.version_not_found_error(query, variant, registry).await),
        }
    }

    /// Get an error for when a query fails to resolve to a version.
    pub async fn version_not_found_error(
        &self,
        query: &VersionQuery,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> anyhow::Error {
        match self.catalog(registry) {
            Ok(catalog) => {
                RegistryVersionResolver::new(catalog, *self.host)
                    .version_not_found_error(query, variant)
                    .await
            }
            Err(err) => err,
        }
    }

    /// Get a suggestion for the user to try using a wildcard.
    pub async fn wildcard_suggestion(
        &self,
        query: &VersionQuery,
        registry: Option<&str>,
    ) -> Option<crate::registry_version_resolver::WildcardSuggestion> {
        let catalog = self.catalog(registry).ok()?;
        RegistryVersionResolver::new(catalog, *self.host)
            .wildcard_suggestion(query, None)
            .await
    }

    /// Select the correct binary for the current host and `variant`.
    pub(super) fn select_platform_binary<'r>(
        &self,
        meta: &'r registry::ReleaseMetadata,
        variant: &Variant,
    ) -> Result<&'r registry::BinaryInfo> {
        registry::select_binary(meta, *self.host, variant).map_err(|err| match err {
            BinarySelectionError::UnsupportedPlatform => terr!("unsupported-platform").into(),
            BinarySelectionError::UnsupportedArch => terr!("unsupported-architecture").into(),
            BinarySelectionError::MissingUrl => terr!("error-file-not-found").into(),
        })
    }
}
