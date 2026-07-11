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

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use async_trait::async_trait;

use crate::version::{QuerySelection, ResolvedSelection, ResolvedVersion, Variant, VersionQuery};
use crate::{eprintln_i18n, t};

#[async_trait(?Send)]
pub trait RunVersionSource {
    async fn get_pinned_version(&self) -> Option<QuerySelection>;
    async fn get_default(&self) -> Result<Option<ResolvedSelection>>;
    async fn determine_version<P: AsRef<Path> + Send + Sync>(
        &self,
        path: Option<P>,
    ) -> Option<(VersionQuery, Option<String>)>;
    async fn auto_install_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
        include_pre: bool,
    ) -> Result<ResolvedVersion>
    where
        T: Into<VersionQuery> + Clone + Send + Sync;
    async fn ensure_installed_version<T>(
        &self,
        gv: &T,
        variant: Option<&str>,
        registry: Option<&str>,
    ) -> Result<ResolvedVersion>
    where
        T: Into<VersionQuery> + Clone + Send + Sync;
}

pub struct RunVersionResolver<'a, S: RunVersionSource> {
    source: &'a S,
}

#[derive(Debug)]
pub struct RunResolutionResult {
    pub version: ResolvedVersion,
    pub variant: Variant,
    pub registry: Option<String>,
}

pub struct RunResolutionRequest<'a> {
    pub explicit: Option<VersionQuery>,
    pub variant: Option<String>,
    pub registry: Option<String>,
    pub include_pre: bool,
    pub possible_paths: &'a [&'a str],
    pub force_on_mismatch: bool,
    pub install_if_missing: bool,
}

impl RunResolutionRequest<'_> {
    fn path_bufs(&self) -> Vec<PathBuf> {
        self.possible_paths
            .iter()
            .map(|p| PathBuf::from(*p))
            .collect()
    }
}

/// Which kind of source was the one that got selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunSource {
    Explicit,
    Pin,
    Project,
    Default,
}

/// The result of selecting which source to use.
#[derive(Debug, Clone)]
pub struct RunSelection {
    pub source: RunSource,
    pub version: VersionQuery,
    pub variant: Option<String>,
    pub registry: Option<String>,
}

impl<'a, S: RunVersionSource> RunVersionResolver<'a, S> {
    pub fn new(source: &'a S) -> Self {
        Self { source }
    }

    /// Determine which source to use for the Godot version.
    pub async fn select(&self, request: &RunResolutionRequest<'_>) -> Result<Option<RunSelection>> {
        if let Some(version) = &request.explicit {
            return Ok(Some(RunSelection {
                source: RunSource::Explicit,
                version: version.clone(),
                variant: request.variant.clone(),
                registry: request.registry.clone(),
            }));
        }

        if let Some(selection) = self.source.get_pinned_version().await {
            return Ok(Some(RunSelection {
                source: RunSource::Pin,
                version: selection.version,
                variant: request.variant.clone().or(selection.variant),
                registry: request.registry.clone().or(selection.registry),
            }));
        }

        let path_bufs = request.path_bufs();
        if let Some((project_version, project_variant)) =
            self.detect_project_version(&path_bufs).await
        {
            return Ok(Some(RunSelection {
                source: RunSource::Project,
                version: project_version,
                variant: request.variant.clone().or(project_variant),
                registry: request.registry.clone(),
            }));
        }

        if let Some(selection) = self.source.get_default().await? {
            return Ok(Some(RunSelection {
                source: RunSource::Default,
                version: selection.version.into(),
                variant: request
                    .variant
                    .clone()
                    .or_else(|| Some(selection.variant.as_str().to_string())),
                registry: request.registry.clone().or(selection.registry),
            }));
        }

        Ok(None)
    }

    /// Resolve the Godot version to use. Installs it if requested.
    pub async fn resolve(&self, request: RunResolutionRequest<'_>) -> Result<RunResolutionResult> {
        let Some(selection) = self.select(&request).await? else {
            return Err(anyhow!(t!("no-default-set")));
        };

        match selection.source {
            RunSource::Explicit => {
                let path_bufs = request.path_bufs();
                if warn_project_version_mismatch::<S, PathBuf>(
                    self.source,
                    &selection.version,
                    false,
                    Some(&path_bufs),
                )
                .await
                    && !request.force_on_mismatch
                {
                    return Err(anyhow!(t!("error-project-version-mismatch", pinned = 0)));
                }
            }
            RunSource::Pin => {
                if warn_project_version_mismatch::<S, PathBuf>(
                    self.source,
                    &selection.version,
                    true,
                    None,
                )
                .await
                    && !request.force_on_mismatch
                {
                    return Err(anyhow!(t!("error-project-version-mismatch", pinned = 1)));
                }
            }
            RunSource::Project => {
                eprintln_i18n!(
                    "warning-using-project-version",
                    version = selection.version.to_display_str()
                );
            }
            RunSource::Default => {}
        }

        let version = if request.install_if_missing {
            self.source
                .auto_install_version(
                    &selection.version,
                    selection.variant.as_deref(),
                    selection.registry.as_deref(),
                    request.include_pre,
                )
                .await?
        } else {
            self.source
                .ensure_installed_version(
                    &selection.version,
                    selection.variant.as_deref(),
                    selection.registry.as_deref(),
                )
                .await?
        };

        Ok(RunResolutionResult {
            version,
            variant: Variant::from_option(selection.variant.as_deref()),
            registry: selection.registry,
        })
    }

    async fn detect_project_version(
        &self,
        paths: &[PathBuf],
    ) -> Option<(VersionQuery, Option<String>)> {
        for path in paths {
            if let Some(result) = self.source.determine_version(Some(path)).await {
                return Some(result);
            }
        }

        self.source.determine_version::<&Path>(None).await
    }
}

pub async fn warn_project_version_mismatch<S: RunVersionSource, P: AsRef<Path> + Send + Sync>(
    source: &S,
    requested: &VersionQuery,
    is_pin: bool,
    paths: Option<&[P]>,
) -> bool {
    let determined = match paths {
        Some(paths) => {
            let mut found = None;
            for path in paths {
                if let Some(result) = source.determine_version(Some(path.as_ref())).await {
                    found = Some(result);
                    break;
                }
            }
            if found.is_none() {
                found = source.determine_version::<&Path>(None).await;
            }
            found
        }
        None => source.determine_version::<&Path>(None).await,
    };

    if let Some((project_version, _variant)) = determined
        && project_version.conflicts_with(requested)
    {
        eprintln_i18n!(
            "warning-project-version-mismatch",
            project_version = project_version.to_display_str(),
            requested_version = requested.to_display_str(),
            pinned = is_pin as i32,
        );
        eprintln!();

        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeSource {
        pinned: Option<VersionQuery>,
        pin_registry: Option<String>,
        project_versions: HashMap<String, VersionQuery>,
        default: Option<ResolvedVersion>,
        auto_result: Option<ResolvedVersion>,
    }

    impl FakeSource {
        fn new() -> Self {
            Self {
                pinned: None,
                pin_registry: None,
                project_versions: HashMap::new(),
                default: None,
                auto_result: None,
            }
        }

        fn with_project(mut self, path: &str, gv: VersionQuery) -> Self {
            self.project_versions.insert(path.to_string(), gv);
            self
        }

        fn with_pinned(mut self, gv: VersionQuery) -> Self {
            self.pinned = Some(gv);
            self
        }

        fn with_pin_registry(mut self, registry: &str) -> Self {
            self.pin_registry = Some(registry.to_string());
            self
        }

        fn with_default(mut self, gv: ResolvedVersion) -> Self {
            self.default = Some(gv);
            self
        }

        fn with_auto(mut self, gv: ResolvedVersion) -> Self {
            self.auto_result = Some(gv);
            self
        }
    }

    #[async_trait::async_trait(?Send)]
    impl RunVersionSource for FakeSource {
        async fn get_pinned_version(&self) -> Option<QuerySelection> {
            self.pinned.clone().map(|version| QuerySelection {
                version,
                variant: None,
                registry: self.pin_registry.clone(),
            })
        }

        async fn get_default(&self) -> Result<Option<ResolvedSelection>> {
            Ok(self.default.clone().map(|version| ResolvedSelection {
                version,
                variant: Variant::default(),
                registry: None,
            }))
        }

        async fn determine_version<P: AsRef<Path> + Send + Sync>(
            &self,
            path: Option<P>,
        ) -> Option<(VersionQuery, Option<String>)> {
            match path {
                Some(p) => self
                    .project_versions
                    .get(&p.as_ref().to_string_lossy().to_string())
                    .cloned()
                    .map(|v| (v, None)),
                None => self
                    .project_versions
                    .get("<cwd>")
                    .cloned()
                    .map(|v| (v, None)),
            }
        }

        async fn auto_install_version<T>(
            &self,
            gv: &T,
            _variant: Option<&str>,
            _registry: Option<&str>,
            _include_pre: bool,
        ) -> Result<ResolvedVersion>
        where
            T: Into<VersionQuery> + Clone + Send + Sync,
        {
            if let Some(result) = self.auto_result.clone() {
                return Ok(result);
            }
            let gv: VersionQuery = gv.clone().into();
            Ok(ResolvedVersion::from(gv))
        }

        async fn ensure_installed_version<T>(
            &self,
            gv: &T,
            _variant: Option<&str>,
            _registry: Option<&str>,
        ) -> Result<ResolvedVersion>
        where
            T: Into<VersionQuery> + Clone + Send + Sync,
        {
            if let Some(result) = self.auto_result.clone() {
                return Ok(result);
            }
            let gv: VersionQuery = gv.clone().into();
            Ok(ResolvedVersion::from(gv))
        }
    }

    fn gv(major: u32, minor: u32, release: &str) -> VersionQuery {
        VersionQuery {
            major: Some(major),
            minor: Some(minor),
            patch: Some(0),
            subpatch: None,
            release_type: Some(release.to_string()),
        }
    }

    fn gvd(major: u32, minor: u32, release: &str) -> ResolvedVersion {
        ResolvedVersion {
            major,
            minor,
            patch: 0,
            subpatch: 0,
            release_type: release.to_string(),
        }
    }

    #[tokio::test]
    async fn prefers_explicit_over_others() {
        let source = FakeSource::new()
            .with_pinned(gv(3, 5, "stable"))
            .with_default(gvd(4, 0, "stable"))
            .with_auto(gvd(5, 0, "stable"));
        let resolver = RunVersionResolver::new(&source);
        let explicit = gv(5, 0, "stable");
        let request = RunResolutionRequest {
            explicit: Some(explicit),
            variant: None,
            registry: None,
            include_pre: false,
            possible_paths: &[],
            force_on_mismatch: false,
            install_if_missing: true,
        };

        let resolved = resolver.resolve(request).await.unwrap();
        assert_eq!(resolved.version.major, 5);
    }

    #[tokio::test]
    async fn pinned_used_when_no_explicit() {
        let source = FakeSource::new()
            .with_pinned(gv(3, 5, "stable"))
            .with_default(gvd(4, 0, "stable"));
        let resolver = RunVersionResolver::new(&source);

        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: None,
                variant: None,
                registry: None,
                include_pre: false,
                possible_paths: &[],
                force_on_mismatch: false,
                install_if_missing: true,
            })
            .await
            .unwrap();

        assert_eq!(resolved.version.major, 3);
    }

    #[tokio::test]
    async fn picks_project_when_available() {
        let source = FakeSource::new()
            .with_project("/proj", gv(4, 2, "stable"))
            .with_default(gvd(4, 0, "stable"));
        let resolver = RunVersionResolver::new(&source);

        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: None,
                variant: None,
                registry: None,
                include_pre: false,
                possible_paths: &["/proj"],
                force_on_mismatch: false,
                install_if_missing: true,
            })
            .await
            .unwrap();

        assert_eq!(resolved.version.minor, 2);
    }

    #[tokio::test]
    async fn falls_back_to_default() {
        let source = FakeSource::new().with_default(gvd(4, 0, "stable"));
        let resolver = RunVersionResolver::new(&source);

        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: None,
                variant: None,
                registry: None,
                include_pre: false,
                possible_paths: &[],
                force_on_mismatch: false,
                install_if_missing: true,
            })
            .await
            .unwrap();

        assert_eq!(resolved.version.major, 4);
    }

    #[tokio::test]
    async fn mismatches_error_when_not_forced() {
        let source = FakeSource::new().with_project("<cwd>", gv(4, 1, "stable"));
        let resolver = RunVersionResolver::new(&source);
        let requested = gv(3, 5, "stable");

        resolver
            .resolve(RunResolutionRequest {
                explicit: Some(requested.clone()),
                variant: None,
                registry: None,
                include_pre: false,
                possible_paths: &[],
                force_on_mismatch: false,
                install_if_missing: true,
            })
            .await
            .unwrap_err();

        // Force allows it to proceed.
        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: Some(requested.clone()),
                variant: None,
                registry: None,
                include_pre: false,
                possible_paths: &[],
                force_on_mismatch: true,
                install_if_missing: true,
            })
            .await
            .unwrap();

        assert_eq!(resolved.version.major, requested.major.unwrap());
    }

    #[tokio::test]
    async fn select_explicit_version_ignores_pinned_registry() {
        let source = FakeSource::new()
            .with_pinned(gv(4, 3, "stable"))
            .with_pin_registry("mybuilds");
        let resolver = RunVersionResolver::new(&source);
        let request = RunResolutionRequest {
            explicit: Some(gv(4, 4, "stable")),
            variant: None,
            registry: None,
            include_pre: false,
            possible_paths: &[],
            force_on_mismatch: false,
            install_if_missing: true,
        };

        let selection = resolver.select(&request).await.unwrap().unwrap();
        assert_eq!(selection.source, RunSource::Explicit);
        assert_eq!(selection.registry, None);
    }

    #[tokio::test]
    async fn select_inherits_pinned_registry_without_explicit() {
        let source = FakeSource::new()
            .with_pinned(gv(4, 3, "stable"))
            .with_pin_registry("mybuilds");
        let resolver = RunVersionResolver::new(&source);
        let request = RunResolutionRequest {
            explicit: None,
            variant: None,
            registry: None,
            include_pre: false,
            possible_paths: &[],
            force_on_mismatch: false,
            install_if_missing: true,
        };

        let selection = resolver.select(&request).await.unwrap().unwrap();
        assert_eq!(selection.source, RunSource::Pin);
        assert_eq!(selection.registry.as_deref(), Some("mybuilds"));
    }
}
