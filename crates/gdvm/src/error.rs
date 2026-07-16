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
use std::fmt;

/// Translation keys for errors and their codes.
pub const ERROR_CODES: &[(&str, &str)] = &[
    // NOTE: When editing these, if an error is removed, leave its code unused
    // so that codes are consistent between versions. Any changes to existing
    // error codes need to be agreed on first, and should be reserved for major
    // releases.
    //
    // GDVM0xxx: stuff external to gdvm, system errors, etc.
    //
    // GDVM0000 is reserved for unknown errors, which won't have translations.
    ("error-find-user-dirs", "GDVM0001"),
    ("error-system-time", "GDVM0002"),
    ("unsupported-platform", "GDVM0003"),
    ("unsupported-architecture", "GDVM0004"),
    ("error-reading-input", "GDVM0005"),
    ("error-create-symlink-windows", "GDVM0006"),
    //
    // GDVM1xxx: version specs, parsing, resolution.
    //
    ("error-unrecognized-version-format", "GDVM1000"),
    ("error-wildcard-position", "GDVM1001"),
    ("error-spec-empty-version", "GDVM1002"),
    ("error-spec-empty-variant", "GDVM1003"),
    ("error-spec-empty-registry", "GDVM1004"),
    ("error-version-not-found", "GDVM1005"),
    ("error-multiple-versions-found", "GDVM1006"),
    ("error-no-stable-releases-found", "GDVM1007"),
    ("no-default-set", "GDVM1008"),
    ("error-pin-version-not-found", "GDVM1009"),
    //
    // GDVM2xxx: registries, network, downloads.
    //
    ("error-download-failed", "GDVM2000"),
    ("error-insecure-url", "GDVM2001"),
    ("error-insecure-redirect", "GDVM2002"),
    ("error-too-many-redirects", "GDVM2003"),
    ("error-response-too-large", "GDVM2004"),
    ("error-response-not-utf8", "GDVM2005"),
    ("error-registry-invalid-name", "GDVM2006"),
    ("error-registry-unknown", "GDVM2007"),
    ("error-registry-not-configured", "GDVM2008"),
    ("error-registry-unsupported-url-scheme", "GDVM2009"),
    ("error-registry-fetch-failed", "GDVM2010"),
    ("error-registry-fetch-release-failed", "GDVM2011"),
    ("error-registry-missing-manifest", "GDVM2012"),
    ("error-registry-missing-index", "GDVM2013"),
    ("error-registry-parse-manifest", "GDVM2014"),
    ("error-registry-parse-index", "GDVM2015"),
    ("error-registry-unsupported-schema", "GDVM2016"),
    ("error-archive-not-cached", "GDVM2017"),
    ("error-invalid-sha-length", "GDVM2018"),
    ("error-checksum-mismatch", "GDVM2019"),
    ("error-size-mismatch", "GDVM2020"),
    ("registry-build-sha-mismatch", "GDVM2021"),
    ("registry-build-size-mismatch", "GDVM2022"),
    ("registry-trust-aborted", "GDVM2023"),
    ("error-publish-missing-manifest", "GDVM2024"),
    ("error-publish-already-initialized", "GDVM2025"),
    ("error-publish-invalid-segment", "GDVM2026"),
    ("error-publish-no-such-version", "GDVM2027"),
    ("error-publish-no-such-variant", "GDVM2028"),
    ("error-publish-no-such-platform", "GDVM2029"),
    ("error-publish-store-requires-file", "GDVM2030"),
    ("error-publish-store-or-url-required", "GDVM2031"),
    ("error-publish-url-requires-integrity", "GDVM2032"),
    ("error-publish-archive-not-found", "GDVM2033"),
    ("registry-validate-failed", "GDVM2034"),
    //
    // GDVM3xxx: installation, archives, filesystem.
    //
    ("error-invalid-path", "GDVM3000"),
    ("error-file-not-found", "GDVM3001"),
    ("error-access-file", "GDVM3002"),
    ("error-create-dir", "GDVM3003"),
    ("error-create-file", "GDVM3004"),
    ("error-write-file", "GDVM3005"),
    ("error-set-permissions", "GDVM3006"),
    ("error-open-zip", "GDVM3007"),
    ("error-read-zip", "GDVM3008"),
    ("error-reopen-zip", "GDVM3009"),
    ("error-read-zip-file", "GDVM3010"),
    ("error-invalid-file-name", "GDVM3011"),
    ("error-strip-prefix", "GDVM3012"),
    ("error-link-exists", "GDVM3013"),
    ("error-link-symlink", "GDVM3014"),
    ("error-link-copy", "GDVM3015"),
    ("error-ensure-godot-binaries-failed", "GDVM3016"),
    ("godot-executable-not-found", "GDVM3017"),
    //
    // GDVM4xxx: configuration.
    //
    ("error-parse-config", "GDVM4000"),
    ("error-unknown-config-key", "GDVM4001"),
    ("error-config-unknown-key", "GDVM4002"),
    ("error-config-invalid-number", "GDVM4003"),
    //
    // GDVM5xxx: running Godot and project detection.
    //
    ("error-starting-godot", "GDVM5000"),
    ("error-failed-reading-project-godot", "GDVM5001"),
    ("error-project-version-mismatch", "GDVM5002"),
    //
    // GDVM6xxx: gdvm upgrade.
    //
    ("error-fetching-gdvm-releases", "GDVM6000"),
    ("error-parsing-gdvm-releases", "GDVM6001"),
    ("error-unsupported-gdvm-schema", "GDVM6002"),
    ("upgrade-no-binary", "GDVM6003"),
    ("upgrade-checksum-required", "GDVM6004"),
    ("upgrade-install-dir-failed", "GDVM6005"),
    ("upgrade-file-create-failed", "GDVM6006"),
    ("upgrade-rename-failed", "GDVM6007"),
    ("upgrade-replace-failed", "GDVM6008"),
    ("error-post-upgrade-action-failed", "GDVM6009"),
    //
    // GDVM7xxx: command usage and CLI stuff.
    //
    ("error-invalid-config-subcommand", "GDVM7000"),
    ("error-invalid-registry-subcommand", "GDVM7001"),
    ("error-non-interactive-trust", "GDVM7002"),
    ("error-non-interactive-value", "GDVM7003"),
];

/// Get the error code for the given translation key.
pub fn code_for(key: &str) -> Option<&'static str> {
    ERROR_CODES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, code)| *code)
}

/// An error with a code and localized error message.
#[derive(Debug)]
pub struct CodedError {
    code: Option<&'static str>,
    message: String,
}

impl CodedError {
    pub fn new(key: &str, message: String) -> Self {
        let code = code_for(key);
        debug_assert!(code.is_some(), "no error code assigned for key {key}");
        Self { code, message }
    }

    pub fn code(&self) -> Option<&'static str> {
        self.code
    }
}

impl fmt::Display for CodedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CodedError {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const EN_US_FTL: &str = include_str!("../i18n/en-US.ftl");

    #[test]
    fn codes_are_unique_and_well_formed() {
        let mut codes = HashSet::new();
        let mut keys = HashSet::new();
        for (key, code) in ERROR_CODES {
            assert!(codes.insert(code), "duplicate error code {code}");
            assert!(keys.insert(key), "duplicate error key {key}");
            assert!(
                code.starts_with("GDVM") && code.len() == 8,
                "malformed code {code}"
            );
            assert!(
                code[4..].chars().all(|c| c.is_ascii_digit()),
                "malformed code {code}"
            );
        }
    }

    #[test]
    fn every_coded_key_is_localized() {
        let defined: HashSet<&str> = EN_US_FTL
            .lines()
            .filter_map(|line| line.split_once('=').map(|(key, _)| key.trim()))
            .filter(|key| !key.starts_with('-') && !key.starts_with('.'))
            .collect();
        for (key, code) in ERROR_CODES {
            assert!(
                defined.contains(key),
                "{code} maps to {key}, which does not exist in en-US.ftl"
            );
        }
    }

    #[test]
    fn code_is_found_through_a_context_chain() {
        let err = anyhow::Error::new(CodedError::new("error-version-not-found", "inner".into()))
            .context("outer context");

        let code = err
            .chain()
            .find_map(|cause| cause.downcast_ref::<CodedError>())
            .and_then(|coded| coded.code());
        assert_eq!(code, Some("GDVM1005"));
    }

    #[test]
    fn coded_errors_carry_their_code() {
        let err = CodedError::new("error-version-not-found", "message".into());
        assert_eq!(err.code(), Some("GDVM1005"));
        assert_eq!(err.to_string(), "message");
    }
}
