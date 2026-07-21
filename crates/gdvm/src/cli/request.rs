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
use clap::ArgMatches;
use gdvm::app::Gdvm;
use gdvm::version::{VersionQuery, VersionSpec, VersionTarget};
use gdvm::{t, terr};

use super::{ensure_registry_trusted, keyword_to_version_filter, refresh_cache_if_requested};

/// Version request from the CLI.
pub(crate) struct VersionRequest {
    variant: Option<String>,
    registry: Option<String>,
    filter: Option<VersionQuery>,
    /// Whether the filter came from a keyword like `latest` or `stable`.
    from_keyword: bool,
}

impl VersionRequest {
    /// Parse the command's `version` argument.
    pub(crate) fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let input = matches.get_one::<String>("version");
        let spec = input.map(|v| VersionSpec::parse(v)).transpose()?;

        let spec_variant = spec.as_ref().and_then(|s| s.variant.clone());
        let variant = apply_deprecated_csharp_flag(matches, spec_variant);
        let registry = spec.as_ref().and_then(|s| s.registry.clone());

        let from_keyword = matches!(
            spec.as_ref().map(|s| &s.target),
            Some(VersionTarget::Keyword(_))
        );
        let filter = spec.map(|s| match s.target {
            VersionTarget::Keyword(kw) => keyword_to_version_filter(&kw),
            VersionTarget::Pattern(gv) => gv,
        });

        Ok(Self {
            variant,
            registry,
            filter,
            from_keyword,
        })
    }

    /// The effective variant.
    pub(crate) fn variant(&self) -> Option<&str> {
        self.variant.as_deref()
    }

    /// The effective variant, owned.
    pub(crate) fn variant_owned(&self) -> Option<String> {
        self.variant.clone()
    }

    /// The registry named in the spec.
    pub(crate) fn registry(&self) -> Option<&str> {
        self.registry.as_deref()
    }

    /// The registry, owned.
    pub(crate) fn registry_owned(&self) -> Option<String> {
        self.registry.clone()
    }

    /// The version filter.
    pub(crate) fn filter_owned(&self) -> Option<VersionQuery> {
        self.filter.clone()
    }

    /// The version filter for a command whose `version` argument is marked as
    /// required by clap.
    pub(crate) fn required_filter(&self) -> &VersionQuery {
        self.filter
            .as_ref()
            .expect("clap guarantees the version argument for this command")
    }

    /// The version filter for matching against installed versions.
    pub(crate) fn installed_filter(&self) -> Result<&VersionQuery> {
        match &self.filter {
            Some(_) if self.from_keyword => Err(terr!("error-version-not-found").into()),
            Some(filter) => Ok(filter),
            None => Err(terr!("error-version-not-found").into()),
        }
    }

    /// Resolve this request using run resolution rules.
    pub(crate) async fn resolve_selection(
        &self,
        gdvm: &Gdvm,
        include_pre: bool,
        install_if_missing: bool,
        force_on_mismatch: bool,
    ) -> Result<gdvm::run_version_resolver::RunResolutionResult> {
        use gdvm::run_version_resolver::{RunResolutionRequest, RunVersionResolver};

        RunVersionResolver::new(gdvm)
            .resolve(RunResolutionRequest {
                explicit: self.filter_owned(),
                variant: self.variant_owned(),
                registry: self.registry_owned(),
                include_pre,
                possible_paths: &[],
                force_on_mismatch,
                install_if_missing,
            })
            .await
    }

    /// Perform common preparation steps. Handles `--refresh` and `--yes` flags.
    /// Commands without these don't need to run this.
    pub(crate) async fn prepare(&self, gdvm: &Gdvm, matches: &ArgMatches) -> Result<()> {
        refresh_cache_if_requested(gdvm, flag(matches, "refresh")).await?;
        ensure_registry_trusted(gdvm, self.registry(), flag(matches, "yes")).await
    }
}

/// Get a flag's value.
fn flag(matches: &ArgMatches, id: &str) -> bool {
    matches!(matches.try_contains_id(id), Ok(true)) && matches.get_flag(id)
}

/// Handle the deprecated `--csharp` flag.
fn apply_deprecated_csharp_flag(
    matches: &ArgMatches,
    spec_variant: Option<String>,
) -> Option<String> {
    if matches.try_contains_id("csharp").is_err() {
        // This command never had the legacy flag, ignore it.
        return spec_variant;
    }

    let explicitly_given = matches
        .value_source("csharp")
        .is_some_and(|source| source != clap::parser::ValueSource::DefaultValue);
    if !explicitly_given {
        return spec_variant;
    }
    gdvm::ui::warn(t!("warning-deprecated-csharp-flag"));

    // The csharp variant takes precedence.
    if spec_variant.is_some() {
        return spec_variant;
    }

    let enabled = matches
        .try_get_one::<bool>("csharp")
        .ok()
        .flatten()
        .copied()
        .unwrap_or(false);
    enabled.then(|| "csharp".to_string())
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    use super::*;

    /// Simple flag from install/remove/use/pin.
    fn cmd_with_csharp() -> Command {
        Command::new("test")
            .arg(Arg::new("version").num_args(0..=1))
            .arg(Arg::new("csharp").long("csharp").action(ArgAction::SetTrue))
    }

    /// Flag with value from run/show/cache-path/link.
    fn cmd_with_valued_csharp() -> Command {
        Command::new("test")
            .arg(Arg::new("version").num_args(0..=1))
            .arg(
                Arg::new("csharp")
                    .long("csharp")
                    .value_parser(clap::value_parser!(bool))
                    .num_args(0..=1)
                    .default_missing_value("true")
                    .default_value("false")
                    .require_equals(true),
            )
    }

    fn cmd_bare() -> Command {
        Command::new("test").arg(Arg::new("version").num_args(0..=1))
    }

    fn parse(cmd: Command, argv: &[&str]) -> VersionRequest {
        let matches = cmd.try_get_matches_from(argv).expect("argv parses");
        VersionRequest::from_matches(&matches).expect("request parses")
    }

    #[test]
    fn parses_variant_registry_and_filter() {
        let request = parse(cmd_bare(), &["test", "myreg/csharp:4.3"]);
        assert_eq!(request.variant(), Some("csharp"));
        assert_eq!(request.registry(), Some("myreg"));
        assert_eq!(request.required_filter().major, Some(4));
        assert_eq!(request.required_filter().minor, Some(3));
    }

    #[test]
    fn keywords_convert_but_never_match_installed() {
        let request = parse(cmd_bare(), &["test", "latest"]);
        assert!(request.filter_owned().is_some());
        assert!(request.installed_filter().is_err());

        let request = parse(cmd_bare(), &["test", "4.3"]);
        assert!(request.installed_filter().is_ok());
    }

    #[test]
    fn missing_version_yields_no_filter() {
        let request = parse(cmd_bare(), &["test"]);
        assert!(request.filter_owned().is_none());
        assert!(request.installed_filter().is_err());
        assert_eq!(request.variant(), None);
        assert_eq!(request.registry(), None);
    }

    #[test]
    fn commands_without_the_legacy_flag_share_the_same_path() {
        let request = parse(cmd_bare(), &["test", "4.3"]);
        assert_eq!(request.variant(), None);
    }

    #[test]
    fn absent_legacy_flag_is_never_treated_as_given() {
        let request = parse(cmd_with_csharp(), &["test", "4.3"]);
        assert_eq!(request.variant(), None);

        let request = parse(cmd_with_valued_csharp(), &["test", "4.3"]);
        assert_eq!(request.variant(), None);
    }

    #[test]
    fn valued_legacy_flag_reads_both_forms() {
        let request = parse(cmd_with_valued_csharp(), &["test", "4.3", "--csharp"]);
        assert_eq!(request.variant(), Some("csharp"));

        let request = parse(cmd_with_valued_csharp(), &["test", "4.3", "--csharp=true"]);
        assert_eq!(request.variant(), Some("csharp"));

        let request = parse(cmd_with_valued_csharp(), &["test", "4.3", "--csharp=false"]);
        assert_eq!(request.variant(), None);
    }

    #[test]
    fn deprecated_csharp_flag_folds_into_the_variant() {
        let request = parse(cmd_with_csharp(), &["test", "4.3", "--csharp"]);
        assert_eq!(request.variant(), Some("csharp"));

        let request = parse(cmd_with_csharp(), &["test", "standard:4.3", "--csharp"]);
        assert_eq!(request.variant(), Some("standard"));

        let request = parse(cmd_with_csharp(), &["test", "4.3"]);
        assert_eq!(request.variant(), None);
    }
}
