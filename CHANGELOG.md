<!--
SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
SPDX-License-Identifier: GPL-3.0-or-later

This file is part of gdvm.

gdvm is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>.
-->

# Changelog

## Unreleased

### Fixed

- Downloaded files are now checksummed as they're written to disk, which prevents anything modifying the file before gdvm checks it.
- Downloads are now checked against the size declared in the registry.
- Temporary directories used for downloads are now randomized so that their names cannot be guessed. This prevents planting files in the directory in advance to trick gdvm into using them.
- When extracting files, gdvm now checks that the file's decompressed size does not exceed the size declared in the registry, and will reject files that do, to help mitigate zip bombs.
- When extracting files, setuid, setgid, and sticky permission bits are no longer applied when extracting, so downloads can no longer create privileged executables on the local system.
- Corrupted timestamps or ones set to future dates no longer cause an integer underflow when computing the age of gdvm's caches.
- Crashes or early exits no longer leave metadata in a partially written state, and no longer leave downloaded files partially written.
- Version tags are now more strictly validated. It is no longer possible to use a version tag that could trick gdvm into placing an install outside of gdvm's data directory by using a version tag such as `4.4-x/../../evil`.
- Environment variables in `.env` files meant for Godot no longer bleed into gdvm's own environment, which could have been used to silently change gdvm's behavior.
- Registry responses are now limited to 64 MiB, so a compromised or misbehaving server can no longer blow up memory with a huge response.

### Changed

- gdvm will no longer follow redirects from `https://` to unencrypted `http://` URLs, which could silently downgrade a secure connection. `GDVM_ALLOW_INSECURE_URLS` can be set to bypass this check.
- `gdvm upgrade` now treats a missing checksum in the release manifest as an error, and will not install the binary.
- gdvm now refuses plain `http://` URLs for all requests. Set the `GDVM_ALLOW_INSECURE_URLS` environment variable to allow unencrypted `http://` URLs. Do not do this unless you are in the middle of developing gdvm or a custom registry. Otherwise, you are putting your system at risk by letting gdvm fetch data over an unencrypted connection.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.13.1...main

## v0.13.1

### Fixed

- An omission in the gdvm release URL caused gdvm's upgrade command to fail with a 404 error. This has been fixed, and gdvm can now upgrade itself again. If you were affected by this, you may need to reinstall gdvm to get the latest version.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.13.0...v0.13.1

## v0.13.0

### Breaking Changes

- Terminal output, aside from help text, is no longer wrapped to a fixed width, and instead relies on the terminal to wrap text. This resolves issues with scripts that parse gdvm output, such as URLs being split across multiple lines.
- The `github.token` configuration setting has been removed. gdvm no longer uses the GitHub API for its own update checks or upgrades, so a token is no longer needed. Any token previously stored in `~/.gdvm/config.toml` is removed automatically the next time gdvm runs.

### New Features

- Custom registries are now supported. Registries can be created using `gdvm registry init` and builds can be added to them with `gdvm registry add-build`. One can configure gdvm to use a custom registry with `gdvm registry add` or by configuring `gdvm.toml`. See the [README](README.md#custom-registries) for more information.
- If a `gdvm.toml` file is malformed, gdvm will now print a warning instead of silently ignoring it.
- `gdvm cache-path <version>` will now print the path to the cached download archive for a given version. This is useful for scripts that need to access the archive directly.
- `gdvm prune` removes installs and cached download archives that are no longer in use. By default it only removes things that have been idle longer than a configurable threshold (settable with `gdvm config set prune.max-age-days <days>`, default 30 days) while preserving any install that still has a link pointing into it, as well as the global install.
- `gdvm upgrade --pre` upgrades gdvm to the latest pre-release. Without the flag, `gdvm upgrade` only upgrades to stable releases, unless you are already running a pre-release and no newer stable release exists yet, in which case it moves to the latest pre-release of that same version.
- gdvm binaries on macOS and Windows are now signed with a code signing certificate. This should prevent warnings about untrusted binaries when running gdvm on those platforms and help identify modified binaries.
- All gdvm releases now provide a SHA256SUMS file with the checksums of all binaries in the release, signed by [sigstore](https://sigstore.dev/). This allows users to verify the integrity of the downloaded binaries and ensure they haven't been tampered with. Validation can be done by running `cosign verify-blob --bundle SHA256SUMS.bundle --certificate-identity-regexp '^https://github.com/adalinesimonian/gdvm/\.github/workflows/release\.yml@' --certificate-oidc-issuer https://token.actions.githubusercontent.com SHA256SUMS` to confirm the checksum file's signature, then `sha256sum --check --ignore-missing SHA256SUMS` to confirm the binaries against it.

### Fixed

- Version tags with multiple hyphens, e.g. `1.2-alpha-something`, are now correctly parsed and resolved.

### Changed

- Switch away from `raw.githubusercontent.com` to `registry.gdvm.io` for the Godot build registry.
- gdvm now reads its own release information from `registry.gdvm.io` instead of the GitHub API, so update checks and `gdvm upgrade` no longer require GitHub API access or are affected by rate limits.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.12.1...v0.13.0

## v0.12.1

### Fixed

- Running `gdvm run latest` would take a long time when the release cache wasn't up to date, because it would check the platform compatibility of every single matching release up front. Now it stops as soon as it finds a compatible release.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.12.0...v0.12.1

## v0.12.0

### Deprecated

The following features have been deprecated. They will continue to work for the time being, but will be removed in a future release. See [MIGRATION.md](MIGRATION.md) for details.

- The `--csharp` flag for selecting C# builds. Use the `csharp:` prefix instead, for example, `csharp:4.4`.
- The `stable` keyword for the latest stable release. Use `latest` instead.
- The `.gdvmrc` file for pinned versions. Use `gdvm.toml` instead.

### New Features

- Added a `latest` keyword that resolves to the latest stable release. If `--pre` is also passed, it resolves to the absolute latest release, including pre-releases and dev builds.
- gdvm now respects `.env` files in the current directory, so you can set environment variables for your projects without having to set them in your shell profile or terminal every time.

### Changed

- The version syntax now uses a `[variant:]version` form. C# builds are selected with the `csharp:` prefix, for example, `csharp:4.4`. The standard build can be named explicitly with the reserved `default:` prefix.
- Pinned versions are stored in a new `gdvm.toml` file.
- Relicensed from ISC to GPL-3.0-or-later. This ensures that improvements to gdvm will be shared with the community.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.11.0...v0.12.0

## v0.11.0

### New Features

- Added the `show` command, which will print the path to the Godot binary for a given version.
- Added the `link` command, which will create a symlink or copy of a specified Godot version at a given path. This is useful for any debugger setups or IDE integrations that can't use the `gdvm` or `godot` commands directly and need a path to the actual Godot binary.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.10.0...v0.11.0

## v0.10.0

### New Features

- Added a `--refresh` flag that updates the Godot build registry cache before running commands. This lets you pull from the latest builds without waiting for the automatic refresh or having to run `gdvm refresh` separately.

### Fixed

- Version resolution now filters registry entries by OS and architecture, so gdvm only picks Godot builds that actually exist for your platform.
- Handles downloads more robustly when the server does not provide a `Content-Length` header.
- Polished translations, including standardised wording across i18n strings.

### Changed

- Cache data is now written atomically to reduce the risk of corrupted registry or release cache files.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.9.0...v0.10.0

## v0.9.0

### New Features

- gdvm now supports a new `gdvm refresh` command to update the local version cache from the registry. This is useful if you want to ensure you have the latest versions available without waiting for the automatic cache refresh interval, or for cases where a build has been re-released with a different checksum. (Looking at you, [Godot 4.5-beta2](https://github.com/godotengine/godot/issues/108190))
- Not enough French in your life? Tu as de la chance ! gdvm now has a French translation thanks to Raphael Astier (@abclive). If you want to contribute a translation, please see the [i18n section in CONTRIBUTING.md](CONTRIBUTING.md#internationalization-i18n).

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.8.1...v0.9.0

## v0.8.1

### Fixed

- `gdvm upgrade` will no longer automatically upgrade across major version boundaries. This should prevent gdvm from automatically upgrading to a release with breaking changes. The new `-m/--major` flag can be used to explicitly allow major version upgrades.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.8.0...v0.8.1

## v0.8.0

### New Features

- gdvm now verifies the checksum of new gdvm binaries when running `gdvm upgrade`. The install scripts do this now as well.

### Changed

- Running gdvm commands is faster now. Instead of using copies of the gdvm binary for the godot/godot_console aliases, a small shim binary is now used. This means gdvm no longer has to check each alias for equivalence to the main binary every time a command is run.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.7.1...v0.8.0

## v0.7.1

### Fixed

- Doesn't keep rechecking for gdvm updates on every command run when the update check fails due to a network error. Now only rechecks if there was a GitHub API error.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.7.0...v0.7.1

## v0.7.0

### New Features

- gdvm now uses a brand-new [Godot build registry](https://github.com/adalinesimonian/gdvm/tree/registry) instead of the GitHub API for fetching releases. This drastically simplifies the gdvm client-side code and makes it more reliable, on top of fixing issues on certain platforms. The registry keeps SHA sums for _all_ Godot binaries ([even all the way back to 1.0!](https://github.com/adalinesimonian/gdvm/blob/1c0e7a1195f60af03aee26e8a4bb66e4bc831059/v1/releases/120826894_1.0-stable.json#L8)), so all downloads are now checked for integrity. (https://github.com/adalinesimonian/gdvm/pull/47)

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.6.2...v0.7.0

## v0.6.2

### Fixed

- #42 When running gdvm without an internet connection, it will no longer fail to resolve a version if the release cache is due for an update.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.6.1...v0.6.2

## v0.6.1

### Fixed

- Arguments are now properly passed to the Godot process on macOS. Thank you to @ryanbraganza. (#27)

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.6.0...v0.6.1

## v0.6.0

### New Features

- You can now configure authentication for the GitHub API to avoid rate limits. You can set the token for GitHub API requests by running `gdvm config set github.token` or by setting the `GITHUB_TOKEN` environment variable. Resolves #20

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.5.1...v0.6.0

## v0.5.1

### Fixed

- #8 When upgrading gdvm with `gdvm upgrade` on \*nix systems, the downloaded binary will now properly be marked as executable. Users upgrading to this release may need to run `gdvm upgrade && chmod +x ~/.gdvm/bin/gdvm && chmod +x ~/.gdvm/bin/godot`, but afterwards will be able to use the upgrade command without issues.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.5.0...v0.5.1

## v0.5.0

### Breaking changes

- No longer modifies arguments sent to Godot, which makes gdvm's behaviour less intrusive.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.4.3...v0.5.0

## v0.4.3

### Fixed

- When running Godot, sends gdvm's own output to stderr, which fixes issues with scripts expecting to operate off of Godot's output.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.4.2...v0.4.3

## v0.4.2

### Fixed

- Fixed false version mismatch errors that could appear on Godot 3.x projects when pinning versions or running them manually.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.4.1...v0.4.2

## v0.4.1

### New Features

- Automatically detects 3.x versions of Godot based on `project.godot`. Limited to just the major version, pinning still recommended.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.4.0...v0.4.1

## v0.4.0

### New Features

- Automatically detects versions specified in `project.godot`, if present. This means with a number of projects, pinning is not strictly required to start the correct version of Godot (but still recommended).

### Fixed

- Fixed a bug where the release cache would be broken by release cache updates, resulting in versions not being found even if they existed remotely.
- Fixed a bug where Godot started in a project folder would error out with "no main scene set".
- More consistently wrapped text displayed in different messages.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.3.2...v0.4.0

## v0.3.2

### New Features

- Install scripts can now optionally associate .godot files with gdvm.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.3.1...v0.3.2

## v0.3.1

### Fixed

- Fixed macOS builds breaking Godot.app on ZIP extraction.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.3.0...v0.3.1

## v0.3.0

### New Features

- On Windows, when `gdvm use` fails to create a symlink due to a permissions error, shows a helpful prompt asking the user to check if Developer Mode is enabled.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.2.3...v0.3.0

## v0.2.3

### Fixed

- Fixed bug where `gdvm upgrade` on all platforms would try to download Windows binaries with a broken path.
- Fixed bug where gdvm would continuously check for updates, even if it recently had done so.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.2.2...v0.2.3

## v0.2.2

### Fixed

- Fixed bug where starting `godot` may not have started Godot but gdvm.
- Fixed bug where pinning a major.minor.0 version would only pin major.minor.

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.2.1...v0.2.2

## v0.2.1

### Fixed

- Fixes bug where Godot would not start properly when passing a relative path to `godot`/`godot_console`
- Fixes bug where console would not be attached or detached as expected
- Fixes bug on Windows where certain builds could not be started in console mode

## v0.2.0

### New Features

- **Pin command** – Pin a Godot version to the current directory with `gdvm pin` which stores the version in a .gdvmrc file.
- **Upgrade command** – Update gdvm to the latest release using `gdvm upgrade`. gdvm will also check for updates to itself when it runs.
- **Automatic Godot symlinks** – You can now run `godot` or `godot_console` directly; they point to gdvm, which picks the correct version for the current directory.
  This lets you associate these symlinks with .godot files in your OS, and if you use `gdvm pin` in those directories, you can have the correct version of Godot open for you when you open a project file.

### Improvements

- **Better messaging** – Enhanced error messages.

## v0.1.0

First release!
