use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::version_utils::{GodotVersion, GodotVersionDeterminate};
use crate::{eprintln_i18n, i18n::I18n, t, t_w};

pub trait RunVersionSource {
    fn get_pinned_version(&self) -> Option<GodotVersion>;
    fn get_default(&self) -> Result<Option<GodotVersionDeterminate>>;
    fn determine_version<P: AsRef<Path>>(&self, path: Option<P>) -> Option<GodotVersion>;
    fn auto_install_version<T>(&self, gv: &T) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone;
}

pub struct RunVersionResolver<'a, S: RunVersionSource> {
    source: &'a S,
    i18n: &'a I18n,
}

pub struct RunResolutionRequest<'a> {
    pub explicit: Option<GodotVersion>,
    pub possible_paths: &'a [&'a str],
    pub csharp_given: bool,
    pub csharp_flag: bool,
    pub force_on_mismatch: bool,
}

impl<'a, S: RunVersionSource> RunVersionResolver<'a, S> {
    pub fn new(source: &'a S, i18n: &'a I18n) -> Self {
        Self { source, i18n }
    }

    /// Resolve the Godot version to use based on the provided request.
    pub fn resolve(&self, request: RunResolutionRequest<'_>) -> Result<GodotVersionDeterminate> {
        let path_bufs: Vec<PathBuf> = request
            .possible_paths
            .iter()
            .map(|p| PathBuf::from(*p))
            .collect();

        if let Some(mut requested_version) = request.explicit {
            requested_version.is_csharp = Some(request.csharp_flag);

            if warn_project_version_mismatch::<S, PathBuf>(
                self.source,
                self.i18n,
                &requested_version,
                false,
                Some(&path_bufs),
            ) && !request.force_on_mismatch
            {
                return Err(anyhow!(t_w!(
                    self.i18n,
                    "error-project-version-mismatch",
                    pinned = 0
                )));
            }

            return self.source.auto_install_version(&requested_version);
        }

        if let Some(pinned) = self.source.get_pinned_version() {
            if warn_project_version_mismatch::<S, PathBuf>(
                self.source,
                self.i18n,
                &pinned,
                true,
                None,
            ) && !request.force_on_mismatch
            {
                return Err(anyhow!(t_w!(
                    self.i18n,
                    "error-project-version-mismatch",
                    pinned = 1
                )));
            }

            return self.source.auto_install_version(&pinned);
        }

        if let Some(project_version) = self.detect_project_version(&path_bufs) {
            eprintln_i18n!(
                self.i18n,
                "warning-using-project-version",
                version = project_version.to_display_str()
            );
            return self.source.auto_install_version(&project_version);
        }

        if let Some(mut default_ver) = self.source.get_default()? {
            if request.csharp_given {
                default_ver.is_csharp = Some(request.csharp_flag);
            }
            return Ok(default_ver);
        }

        Err(anyhow!(t!(self.i18n, "no-default-set")))
    }

    fn detect_project_version(&self, paths: &[PathBuf]) -> Option<GodotVersion> {
        if let Some(version) = paths
            .iter()
            .find_map(|p| self.source.determine_version(Some(p)))
        {
            return Some(version);
        }

        self.source.determine_version::<&Path>(None)
    }
}

pub fn warn_project_version_mismatch<S: RunVersionSource, P: AsRef<Path>>(
    source: &S,
    i18n: &I18n,
    requested: &GodotVersion,
    is_pin: bool,
    paths: Option<&[P]>,
) -> bool {
    let determined_version = match paths {
        Some(paths) => paths
            .iter()
            .find_map(|p| source.determine_version(Some(p.as_ref())))
            .or_else(|| source.determine_version::<&Path>(None)),
        None => source.determine_version::<&Path>(None),
    };

    if let Some(project_version) = determined_version {
        if project_version.conflicts_with(requested) {
            eprintln_i18n!(
                i18n,
                "warning-project-version-mismatch",
                project_version = project_version.to_display_str(),
                requested_version = requested.to_display_str(),
                pinned = is_pin as i32,
            );
            eprintln!();

            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeSource {
        pinned: Option<GodotVersion>,
        project_versions: HashMap<String, GodotVersion>,
        default: Option<GodotVersionDeterminate>,
        auto_result: Option<GodotVersionDeterminate>,
    }

    impl FakeSource {
        fn new() -> Self {
            Self {
                pinned: None,
                project_versions: HashMap::new(),
                default: None,
                auto_result: None,
            }
        }

        fn with_project(mut self, path: &str, gv: GodotVersion) -> Self {
            self.project_versions.insert(path.to_string(), gv);
            self
        }

        fn with_pinned(mut self, gv: GodotVersion) -> Self {
            self.pinned = Some(gv);
            self
        }

        fn with_default(mut self, gv: GodotVersionDeterminate) -> Self {
            self.default = Some(gv);
            self
        }

        fn with_auto(mut self, gv: GodotVersionDeterminate) -> Self {
            self.auto_result = Some(gv);
            self
        }
    }

    impl RunVersionSource for FakeSource {
        fn get_pinned_version(&self) -> Option<GodotVersion> {
            self.pinned.clone()
        }

        fn get_default(&self) -> Result<Option<GodotVersionDeterminate>> {
            Ok(self.default.clone())
        }

        fn determine_version<P: AsRef<Path>>(&self, path: Option<P>) -> Option<GodotVersion> {
            match path {
                Some(p) => self
                    .project_versions
                    .get(&p.as_ref().to_string_lossy().to_string())
                    .cloned(),
                None => self.project_versions.get("<cwd>").cloned(),
            }
        }

        fn auto_install_version<T>(&self, gv: &T) -> Result<GodotVersionDeterminate>
        where
            T: Into<GodotVersion> + Clone,
        {
            if let Some(result) = self.auto_result.clone() {
                return Ok(result);
            }
            let gv: GodotVersion = gv.clone().into();
            Ok(GodotVersionDeterminate::from(gv))
        }
    }

    fn gv(major: u32, minor: u32, release: &str) -> GodotVersion {
        GodotVersion {
            major: Some(major),
            minor: Some(minor),
            patch: Some(0),
            subpatch: None,
            release_type: Some(release.to_string()),
            is_csharp: Some(false),
        }
    }

    fn gvd(major: u32, minor: u32, release: &str) -> GodotVersionDeterminate {
        GodotVersionDeterminate {
            major,
            minor,
            patch: 0,
            subpatch: 0,
            release_type: release.to_string(),
            is_csharp: Some(false),
        }
    }

    fn intl() -> I18n {
        I18n::new(80).expect("i18n init")
    }

    #[test]
    fn prefers_explicit_over_others() {
        let source = FakeSource::new()
            .with_pinned(gv(3, 5, "stable"))
            .with_default(gvd(4, 0, "stable"))
            .with_auto(gvd(5, 0, "stable"));
        let intl = intl();
        let resolver = RunVersionResolver::new(&source, &intl);
        let explicit = gv(5, 0, "stable");
        let request = RunResolutionRequest {
            explicit: Some(explicit),
            possible_paths: &[],
            csharp_given: false,
            csharp_flag: false,
            force_on_mismatch: false,
        };

        let resolved = resolver.resolve(request).unwrap();
        assert_eq!(resolved.major, 5);
    }

    #[test]
    fn pinned_used_when_no_explicit() {
        let source = FakeSource::new()
            .with_pinned(gv(3, 5, "stable"))
            .with_default(gvd(4, 0, "stable"));
        let intl = intl();
        let resolver = RunVersionResolver::new(&source, &intl);

        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: None,
                possible_paths: &[],
                csharp_given: false,
                csharp_flag: false,
                force_on_mismatch: false,
            })
            .unwrap();

        assert_eq!(resolved.major, 3);
    }

    #[test]
    fn picks_project_when_available() {
        let source = FakeSource::new()
            .with_project("/proj", gv(4, 2, "stable"))
            .with_default(gvd(4, 0, "stable"));
        let intl = intl();
        let resolver = RunVersionResolver::new(&source, &intl);

        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: None,
                possible_paths: &["/proj"],
                csharp_given: false,
                csharp_flag: false,
                force_on_mismatch: false,
            })
            .unwrap();

        assert_eq!(resolved.minor, 2);
    }

    #[test]
    fn falls_back_to_default() {
        let source = FakeSource::new().with_default(gvd(4, 0, "stable"));
        let intl = intl();
        let resolver = RunVersionResolver::new(&source, &intl);

        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: None,
                possible_paths: &[],
                csharp_given: false,
                csharp_flag: false,
                force_on_mismatch: false,
            })
            .unwrap();

        assert_eq!(resolved.major, 4);
    }

    #[test]
    fn mismatches_error_when_not_forced() {
        let source = FakeSource::new().with_project("<cwd>", gv(4, 1, "stable"));
        let intl = intl();
        let resolver = RunVersionResolver::new(&source, &intl);
        let requested = gv(3, 5, "stable");

        resolver
            .resolve(RunResolutionRequest {
                explicit: Some(requested.clone()),
                possible_paths: &[],
                csharp_given: false,
                csharp_flag: false,
                force_on_mismatch: false,
            })
            .unwrap_err();

        // Force allows it to proceed.
        let resolved = resolver
            .resolve(RunResolutionRequest {
                explicit: Some(requested.clone()),
                possible_paths: &[],
                csharp_given: false,
                csharp_flag: false,
                force_on_mismatch: true,
            })
            .unwrap();

        assert_eq!(resolved.major, requested.major.unwrap());
    }
}
