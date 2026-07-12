# SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
# SPDX-License-Identifier: GPL-3.0-or-later
#
# This file is part of gdvm.
#
# gdvm is free software: you can redistribute it and/or modify it under the
# terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later
# version.
#
# gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE. See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.

hello = Hello, World!

help-about = Godot Version Manager
help-help = Print help (see a summary with '-h')
help-help-command = Print this message or the help of the given subcommand(s)
help-gdvm-version = Display the version of Godot Version Manager

help-install = Install a new Godot version
help-run = Run a specific Godot version
help-show = Show the path to the executable for the given version of Godot
help-cache-path = Show the path to the cached download archive for the given version of Godot
help-link = Link the executable of a Godot version to a specified path
help-list = List all installed Godot versions
help-remove = Remove an installed Godot version

help-branch = The branch (stable, beta, alpha, or custom).
help-csharp = [deprecated] Use the Godot version with C# support. Use the "csharp" variant specifier instead (e.g. csharp:4.4).
help-run-csharp-long = { help-csharp }

    If given, value overrides the default version set with "use". Otherwise, the default version is used. In other words, if you set a default version with "use --csharp", you can try to run the same version but without C# support with "run --csharp false". However, it may not work as expected if the version without C# support is not installed. (Just run "install" to install it.)
help-version = The version to install (e.g. 4, csharp:4.4, stable, latest).
help-version-long =
    { help-version }

    Format: [variant:]version_or_keyword

    Keywords: "latest" resolves to the newest version. By default, this includes only stable releases, but pre-releases can be included with the --pre flag.

    Variants: prefix with a variant name and colon, e.g. "csharp:4.4" for the C# build.

    Examples: 4.4 will install the latest stable release of Godot 4.4. If only pre-release versions exist, the latest pre-release version will be installed. 4.3-rc will install the latest release candidate of Godot 4.3, etc.
help-version-installed = The installed version (e.g. 4.2 or 4.2-stable).

help-search = List available releases from the registry
help-filter = Optional string to filter release tags
help-include-pre = Include pre-release versions (rc, beta, dev)
help-cache-only = Use only cached release information without querying the registry
help-limit = Number of releases to list, default is 10. Use 0 to list all
help-clear-cache = Clears the release cache
help-refresh = Refresh the release cache from the registry
help-refresh-flag = Refresh the release cache before running this command

help-prune = Remove installs and cached archives that are no longer in use
help-prune-long = { help-prune }

    By default, prune removes installs that have not been used in a while and cached download archives that have aged out, while preserving any install that still has a link pointing into it. The install set as the default is never removed, whatever flags are given. The age threshold is configurable with "gdvm config set prune.max-age-days <days>" (default { $default_days } days).
help-prune-all = Remove all installs and cached archives regardless of age. Installs that still have a live link are kept unless --force is also given.
help-prune-force = Ignore links, so installs referenced only by a link may also be removed.
help-prune-dry-run = Show what would be removed without deleting anything.

prune-dry-run-header = The following would be removed (dry run):
prune-removed-header = Removed the following:
prune-installs-header = Installs:
prune-archives-header = Cached archives:
prune-nothing-dry-run = Nothing would be removed.
prune-nothing-removed = Nothing to remove; everything is in use or within the age threshold.
prune-preserved-by-link = Kept { $count } install(s) still referenced by a link.
prune-freed = Freed approximately { $size }.
prune-would-free = Would free approximately { $size }.

help-force = Force reinstall even if the version is already installed.
help-redownload = Redownload the version even if it's already downloaded in the cache.
help-yes = Skip confirmation prompt for removal
help-link-version = The version to link. If not provided, resolves the version based on the current directory or default version.
help-link-path = The path where the link or copy will be created, e.g. "{ $platform ->
    [windows] godot.exe
    [macos] godot.app
    *[other] godot
    }".
help-link-force = Overwrite existing link if it exists
help-link-copy = Copy the executable instead of creating a link

cached-zip-stored = Saved Godot release archive to cache.
using-cached-zip = Using cached release archive, skipping download.
warning-cache-metadata-reset = Cached release index invalid or corrupted. Resetting.
cache-files-removed = Cache files have been successfully removed.
cache-metadata-removed = Cache metadata has been successfully removed.
error-cache-metadata-empty = Error: Cache metadata is empty, need to fetch releases first.
no-cache-files-found = No cache files were found.
no-cache-metadata-found = No cache metadata was found.
gdvm-toml-malformed = {"\u001b"}[33mWarning: ignoring gdvm.toml at { $path } because it could not be parsed: { $error }{"\u001b"}[0m

help-console = Run Godot with the console attached. Defaults to false on Windows, true on other platforms.

help-default = Manage the default version
help-default-version = The version to set as default (e.g. 4.2 or 4.2-stable).
no-default-set = No default version set. Run "gdvm use <version>" to set a default version system-wide, or "gdvm pin <version>" to set a default version for the current directory.

installing-version = Installing version {$version}
installed-success = Successfully installed {$version}

warning-prerelease = {"\u001b"}[33mWarning: You are installing a pre-release version ({$branch}).{"\u001b"}[0m
warning-deprecated-csharp-flag = {"\u001b"}[33mWarning: The --csharp flag is deprecated. Use the "csharp" variant specifier instead (e.g. csharp:4.4).{"\u001b"}[0m

force-reinstalling-version = Forcing reinstallation of version {$version}.

auto-installing-version = Auto-installing version { $version }

no-versions-installed = No versions installed.
installed-versions = Installed Godot versions:
removed-version = Removed version {$version}
removing-version = Removing version {$version}

force-redownload = Forcing redownload of version {$version}.
operation-downloading-url = Downloading {$url}...
operation-download-complete = Download complete.
operation-extracting = Extracting...
operation-extract-complete = Extraction complete.

unsupported-platform = Unsupported platform
unsupported-architecture = Unsupported architecture

verifying-checksum = Verifying checksum...
checksum-verified = Checksum verified.
error-checksum-mismatch = Checksum mismatch for file { $file }
error-invalid-sha-length = Invalid SHA length { $length }
error-size-mismatch = Size mismatch for file { $file }: expected { $expected } bytes, got { $actual } bytes.
error-insecure-url = Refusing to fetch { $url } over an unencrypted connection. Only https:// and file:// URLs are allowed. Set the GDVM_ALLOW_INSECURE_URLS environment variable to allow unencrypted http:// URLs.
error-insecure-redirect = Refusing to follow a redirect from https:// to an unencrypted http:// URL. Set the GDVM_ALLOW_INSECURE_URLS environment variable to allow unencrypted http:// URLs.
error-response-not-utf8 = The response from { $url } is not valid UTF-8: { $error }
error-response-too-large = The response from { $url } exceeds the maximum allowed size of { $limit } bytes.
error-too-many-redirects = Too many redirects.
error-config-invalid-number = Invalid value for { $key }: { $value } (expected a number)
error-config-unknown-key = Unknown configuration key: { $key }
error-invalid-path = Invalid path: { $path }
error-publish-missing-manifest = missing registry.json
error-publish-no-such-version = no such version: { $version }
error-publish-store-or-url-required = either --store or --url must be provided
error-publish-store-requires-file = --store requires a local --file
error-publish-url-requires-integrity = --url requires either a local --file or explicit --sha512 and --size
error-registry-fetch-failed = Failed to fetch { $url }: HTTP { $status }
error-registry-fetch-release-failed = Failed to fetch release metadata
error-registry-invalid-name = Invalid registry name: { $name }
error-registry-missing-index = Registry '{ $name }' is missing index.json
error-registry-missing-manifest = Registry '{ $name }' is missing registry.json
error-registry-not-configured = Registry '{ $name }' is not configured
error-registry-parse-index = Failed to parse index for '{ $name }': { $error }
error-registry-parse-manifest = Failed to parse manifest for '{ $name }': { $error }
error-registry-unknown = Unknown registry '{ $name }'
error-registry-unsupported-url-scheme = Unsupported registry URL scheme: { $url }
error-spec-empty-registry = Empty registry name in '{ $input }'
error-spec-empty-variant = Empty variant name in '{ $input }'
error-spec-empty-version = Empty version in '{ $input }'
error-system-time = System time before UNIX EPOCH
error-unrecognized-version-format = Unrecognized version format: { $input }
download-retrying = Download interrupted, retrying (attempt { $attempt } of { $max })...
lock-waiting = Waiting for another gdvm process to finish (lock: { $resource })...
prune-skipped-error = Skipping { $item }: { $error }
prune-skipped-in-use = Skipping { $item }: it is in use by another gdvm process.

error-find-user-dirs = Failed to find user directories.

fetching-releases = Fetching releases...
releases-fetched = Releases fetched.
error-fetching-releases = Error fetching releases: { $error }
warning-fetching-releases-using-cache = Error fetching releases: { $error }. Using cached releases instead.

error-version-not-found = Version not found.
error-archive-not-cached = No cached archive found for {$version}. Install it first to populate the cache.
error-multiple-versions-found = Multiple versions match your request:

running-version = Running version {$version}
link-created = Linked {$version} to {$path}
copy-created = Copied {$version} to {$path}
no-matching-releases = No matching releases found.
available-releases = Available releases:
cache-cleared = Cache cleared successfully.
cache-refreshed = Cache refreshed successfully.

version-already-installed = Version {$version} already installed.
godot-executable-not-found = Godot executable not found for version {$version}.
error-link-exists = Path {$path} already exists. Use --force to overwrite.
error-link-symlink = Failed to create link from {$link} to {$target}: {$error}
error-link-copy = Failed to copy executable: {$error}

error-no-stable-releases-found = No stable releases found.

error-starting-godot = Failed to start Godot: { $error }

confirm-remove = Are you sure you want to remove this version? (yes/no):
confirm-yes = yes
remove-cancelled = Removal cancelled.

default-set-success = Successfully set {$version} as the default Godot version.
default-unset-success = Successfully unset the default Godot version.
provide-version-or-unset = Please provide a version to set as default or 'unset' to remove the default version.

error-open-zip = Failed to open ZIP file { $path }: { $error }
error-read-zip = Failed to read ZIP archive { $path }: { $error }
error-access-file = Failed to access file at index { $index }: { $error }
error-reopen-zip = Failed to reopen ZIP file { $path }: { $error }
error-invalid-file-name = Invalid file name in ZIP archive
error-create-dir = Failed to create directory { $path }: { $error }
error-create-file = Failed to create file { $path }: { $error }
error-read-zip-file = Failed to read from ZIP file { $file }: { $error }
error-write-file = Failed to write to file { $path }: { $error }
error-strip-prefix = Error stripping prefix: { $error }
error-set-permissions = Failed to set permissions for { $path }: { $error }
error-create-symlink-windows = Could not create symlink. Please ensure {"\u001b"}]8;;ms-settings:developers{"\u001b"}\Developer Mode{"\u001b"}]8;;{"\u001b"}\ is enabled or run as admin.

help-upgrade = Upgrade gdvm to the latest version
help-upgrade-major = Allow upgrading across major versions
help-upgrade-pre = Upgrade to the latest pre-release version
upgrade-starting = Starting gdvm upgrade...
upgrade-downloading-latest = Downloading the latest gdvm version...
upgrade-complete = gdvm was successfully upgraded!
upgrade-not-needed = gdvm is already at the latest version: { $version }.
upgrade-current-version-newer = The current gdvm version ({ $current }) is newer than the latest available version ({ $latest }). No upgrade needed.
upgrade-failed = Upgrade failed: { $error }
upgrade-download-failed = Upgrade download failed: { $error }
upgrade-file-create-failed = Failed to create upgrade file: { $error }
upgrade-file-write-failed = Failed to write to upgrade file: { $error }
upgrade-install-dir-failed = Failed to create the installation directory: { $error }
upgrade-rename-failed = Failed to rename the current executable: { $error }
upgrade-replace-failed = Failed to replace the executable with the new one: { $error }
upgrade-no-binary = No gdvm binary is available for version { $version } and target { $target }.
upgrade-checksum-required = The release manifest does not include a checksum for this gdvm binary. Refusing to upgrade.
error-fetching-gdvm-releases = Error fetching gdvm releases: { $error }
error-parsing-gdvm-releases = Error parsing gdvm releases: { $error }
error-unsupported-gdvm-schema = Unsupported gdvm release manifest schema version: { $schema }. Try upgrading gdvm manually.
upgrade-available = 💡 A new version of gdvm is available: {$version}. Run "gdvm upgrade" to update.
upgrade-available-major = 💡 A major version update of gdvm is available: {$version}. Run "gdvm upgrade -m" to update.
upgrade-available-both = 💡 A new version of gdvm is available: {$minor_version}. A major version update is also available: {$major_version}. Run "gdvm upgrade" to update within the current major version, or "gdvm upgrade -m" to upgrade to the latest version.
upgrade-prerelease-available = 💡 A newer pre-release of gdvm is available. Run "gdvm upgrade --pre" to install it.

help-pin = Pin a version of Godot to the current directory.
help-pin-long = { help-pin }

    This will create a gdvm.toml file in the current directory with the pinned version. When you run "gdvm run" in this directory or any of its subdirectories, the pinned version will be used instead of the default version.

    This is useful when you want to use a specific version of Godot for a project without changing the default version system-wide.

    This currently also writes the legacy .gdvmrc file for compatibility with older versions of gdvm. This will be removed in a future release, so it is recommended to update to the new gdvm.toml format and remove the .gdvmrc file if it exists.

    You can disable writing a .gdvmrc file using the --no-legacy flag.
help-pin-version = The version to pin
help-no-legacy = Do not write the legacy .gdvmrc compatibility file
pinned-success = Successfully pinned version {$version} in gdvm.toml
error-pin-version-not-found = Could not pin version {$version}
pin-subcommand-description = Set or update gdvm.toml with the requested version

error-file-not-found = File not found. It may not exist on the server.
error-download-failed = Download failed due to an unexpected error: { $error }
error-ensure-godot-binaries-failed = Failed to ensure Godot binaries.
    Error: { $error }.
    Try removing { $path } and then run gdvm again.

error-failed-reading-project-godot = Failed reading project.godot, cannot automatically determine project version.
warning-using-project-version = Using version { $version } defined in project.godot.

warning-project-version-mismatch =
    {"\u001b"}[33mWarning: The version defined in project.godot does not match the { $pinned ->
        [1] pinned
        *[0] requested
    } version. Opening the project with the { $pinned ->
        [1] pinned
        *[0] requested
    } version may overwrite the project file.{"\u001b"}[0m

    { $pinned ->
        [1] Project version: { $project_version }
            Pinned version:  { $requested_version }
        *[0] Project version:   { $project_version }
             Requested version: { $requested_version }
    }

error-project-version-mismatch = {"\u001b"}[31m{ $pinned ->
        [1] If you are sure you want to run the project with the pinned version, run {"\u001b"}[0mgdvm run --force{"\u001b"}[31m. Otherwise, update the pinned version in .gdvmrc to match the project version, or remove the .gdvmrc file to use the project version.
        *[0] If you are sure you want to run the project with the requested version, run {"\u001b"}[0mgdvm run --force <version>{"\u001b"}[31m.
    }{"\u001b"}[0m
warning-project-version-mismatch-force = {"\u001b"}[33mSkipping confirmation prompt and continuing with { $pinned ->
        [1] pinned
        *[0] requested
    } version {"\u001b"}[0m({ $requested_version }){"\u001b"}[33m.{"\u001b"}[0m

help-run-args = Additional arguments to pass to the Godot executable (e.g. -- path/to/project.godot).
help-run-force =
    Force running the project with the requested or pinned version even if it doesn't match the project version.
help-run-force-long =
    { help-run-force }

    If you do this, the requested or pinned version of Godot may overwrite the project file. If pinning versions, it is instead recommended to update the pinned version in .gdvmrc to match the project version, or remove the .gdvmrc file to use the project version.

help-config = Manage gdvm configuration
help-config-get = Get a configuration value
help-config-set = Set a configuration value
help-config-unset = Unset a configuration value
help-config-list = List all configuration values
help-config-key = The configuration key (e.g., prune.max-age-days)
help-config-value = The value to set for the configuration key
help-config-unset-key = The configuration key to unset (e.g., prune.max-age-days)
help-config-show-sensitive = Show sensitive configuration values in plaintext
help-config-available = List all available configuration keys and their values, including defaults
warning-setting-sensitive = {"\u001b"}[33mWarning: You are setting a sensitive value which will be stored in plaintext in your home directory.{"\u001b"}[0m
config-set-prompt = Please enter the value for { $key }:
error-reading-input = Error reading input
config-set-success = Configuration updated successfully.
config-unset-success = Configuration key { $key } unset successfully.
config-key-not-set = Configuration key not set.
error-unknown-config-key = Unknown configuration key.
error-invalid-config-value = Invalid value for configuration key { $key }.
error-invalid-config-subcommand = Invalid config subcommand. Use "get", "set", or "list".
error-parse-config = Failed to parse configuration file: { $error }
error-parse-config-using-default = {"\u001b"}[33mUsing default configuration values.{"\u001b"}[0m

help-registry = Manage registries to install Godot builds from
help-registry-add = Add a registry
help-registry-remove = Remove a registry
help-registry-list = List configured registries
help-registry-refresh = Refresh one or all registry caches
help-registry-name = The registry name
help-registry-url = The registry URL. Can be an http(s):// or file:// URL.

registry-added = Added registry { $registry } ({ $url }).
registry-removed = Removed registry { $registry }.
registry-list-header = Configured registries:
registry-tag-official = official
registry-error = Registry error: { $error }

error-invalid-registry-subcommand = Invalid registry subcommand. Use "add", "remove", "list", or "refresh".
registry-trust-warning = {"\u001b"}[33m{ $registry } ({ $url }) is a custom registry, not the official one. gdvm makes sure downloads match what the registry says to expect, but it can't tell whether they are safe to run. Only install from it if you trust whoever runs it.{"\u001b"}[0m
registry-trust-prompt = Do you trust this registry and want to continue? (yes/no):
registry-trust-bypass = {"\u001b"}[1;31mSkipping the trust check for { $registry } ({ $url }) because you used --yes. gdvm can't tell whether its files are safe to run. Pausing for a moment; press Ctrl+C now to stop.{"\u001b"}[0m
registry-trust-aborted = Aborted: registry not trusted.
registry-project-override-conflict = {"\u001b"}[33mThe project's gdvm.toml redefines the registry { $registry } (your configuration: { $machine_url }) as { $project_url }. The project's definition takes precedence.{"\u001b"}[0m

help-registry-init = Initialize a new registry directory
help-registry-add-build = Add a build to a registry
help-registry-remove-build = Remove a build from a registry
help-registry-validate = Validate a registry directory
help-registry-dir = The registry directory
help-registry-init-name = The registry name. Defaults to the directory name.

help-registry-build-version = The version tag, e.g. 4.4-stable.
help-registry-build-variant = The variant name. Defaults to "default".
help-registry-build-platform = The platform key, e.g. linux-x86_64.
help-registry-build-file = Path to the build archive to hash
help-registry-build-store = Copy the archive into the registry and record a relative URL
help-registry-build-url = The URL where the archive will be served (when not using --store)
help-registry-build-sha512 = The archive's SHA-512, in lieu of computing it. Requires --size.
help-registry-build-size = The archive's size in bytes, in lieu of measuring it. Requires --sha512.

registry-init-success = Initialized registry { $name } at { $path }.
registry-build-added = Added build { $version } for { $platform }.
registry-build-removed = Removed build { $version }.
registry-build-downloading = Downloading { $url } to compute its size and SHA-512…
registry-build-warn-local-hash = {"\u001b"}[33mHashing the local file and assuming it matches { $url }. gdvm is not downloading the URL to verify it.{"\u001b"}[0m
registry-build-warn-unverified = {"\u001b"}[33mUsing the SHA-512 and size you provided without downloading the artifact to verify them. Make sure they are correct.{"\u001b"}[0m
registry-build-warn-explicit-store = {"\u001b"}[33mUsing the SHA-512 and/or size you provided instead of measuring the stored archive.{"\u001b"}[0m
registry-build-sha-mismatch = The provided SHA-512 ({ $expected }) does not match the artifact ({ $actual }).
registry-build-size-mismatch = The provided size ({ $expected }) does not match the artifact ({ $actual }).
registry-validate-ok = Registry is valid ({ $count } artifacts checked).
registry-validate-failed = Registry validation failed:
