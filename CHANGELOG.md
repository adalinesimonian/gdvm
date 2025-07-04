# Changelog

## Unreleased

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v0.9.0...main

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
