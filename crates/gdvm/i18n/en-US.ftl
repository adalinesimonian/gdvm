hello = Hello, World!

help-about = Godot Version Manager
help-help = Print help (see a summary with '-h')
help-help-command = Print this message or the help of the given subcommand(s)
help-gdvm-version = Display the version of Godot Version Manager

help-install = Install a new Godot version
help-run = Run a specific Godot version
help-list = List all installed Godot versions
help-remove = Remove an installed Godot version

help-branch = The branch (stable, beta, alpha, or custom).
help-csharp = Use the Godot version with C# support.
help-run-csharp-long = { help-csharp }

    If given, value overrides the default version set with "use". Otherwise, the default version is used. In other words, if you set a default version with "use --csharp", you can try to run the same version but without C# support with "run --csharp false". However, it may not work as expected if the version without C# support is not installed. (Just run "install" to install it.)
help-version = The version to install (e.g. 4), or "stable" for the latest stable version.
help-version-long =
    { help-version }

    Examples: 4.4 will install the latest stable release of Godot 4.4. If only pre-release versions exist, in which case, the latest pre-release version will be installed. 4.3-rc will install the latest release candidate of Godot 4.3, etc.
help-version-installed = The installed version (e.g. 4.2 or 4.2-stable).

help-search = List available releases from the registry
help-filter = Optional string to filter release tags
help-include-pre = Include pre-release versions (rc, beta, dev)
help-cache-only = Use only cached release information without querying the registry
help-limit = Number of releases to list, default is 10. Use 0 to list all
help-clear-cache = Clears the release cache
help-refresh = Refresh the release cache from the registry
help-refresh-flag = Refresh the release cache before running this command

help-force = Force reinstall even if the version is already installed.
help-redownload = Redownload the version even if it's already downloaded in the cache.
help-yes = Skip confirmation prompt for removal

cached-zip-stored = Saved Godot release archive to cache.
using-cached-zip = Using cached release archive, skipping download.
warning-cache-metadata-reset = Cached release index invalid or corrupted. Resetting.
cache-files-removed = Cache files have been successfully removed.
cache-metadata-removed = Cache metadata has been successfully removed.
error-cache-metadata-empty = Error: Cache metadata is empty, need to fetch releases first.
no-cache-files-found = No cache files were found.
no-cache-metadata-found = No cache metadata was found.

help-console = Run Godot with the console attached. Defaults to false on Windows, true on other platforms.

help-default = Manage the default version
help-default-version = The version to set as default (e.g. 4.2 or 4.2-stable).
no-default-set = No default version set. Run "gdvm use <version>" to set a default version system-wide, or "gdvm pin <version>" to set a default version for the current directory.

installing-version = Installing version {$version}
installed-success = Successfully installed {$version}

warning-prerelease = {"\u001b"}[33mWarning: You are installing a pre-release version ({$branch}).{"\u001b"}[0m

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
warning-sha-sums-missing = Checksum files not found for this release. Skipping verification.

error-find-user-dirs = Failed to find user directories.

fetching-releases = Fetching releases...
releases-fetched = Releases fetched.
error-fetching-releases = Error fetching releases: { $error }
warning-fetching-releases-using-cache = Error fetching releases: { $error }. Using cached releases instead.

error-version-not-found = Version not found.
error-multiple-versions-found = Multiple versions match your request:

error-invalid-godot-version = Invalid Godot version format. Expected formats: x, x.y, x.y.z, x.y.z.w, x.y.z-tag.
error-invalid-remote-version = Invalid remote Godot version format. Expected formats: x, x.y, x.y.z, x.y.z.w, x.y.z-tag, or "stable".

running-version = Running version {$version}
no-matching-releases = No matching releases found.
available-releases = Available releases:
cache-cleared = Cache cleared successfully.
cache-refreshed = Cache refreshed successfully.

version-already-installed = Version {$version} already installed.
godot-executable-not-found = Godot executable not found for version {$version}.

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
checking-updates = Checking for updates to gdvm...
upgrade-available = 💡 A new version of gdvm is available: {$version}. Run "gdvm upgrade" to update.
upgrade-available-major = 💡 A major version update of gdvm is available: {$version}. Run "gdvm upgrade -m" to update.
upgrade-available-both = 💡 A new version of gdvm is available: {$minor_version}. A major version update is also available: {$major_version}. Run "gdvm upgrade" to update within the current major version, or "gdvm upgrade -m" to upgrade to the latest version.

help-pin = Pin a version of Godot to the current directory.
help-pin-long = { help-pin }

    This will create a .gdvmrc file in the current directory with the pinned version. When you run "gdvm run" in this directory or any of its subdirectories, the pinned version will be used instead of the default version.

    This is useful when you want to use a specific version of Godot for a project without changing the default version system-wide.
help-pin-version = The version to pin
pinned-success = Successfully pinned version {$version} in .gdvmrc
error-pin-version-not-found = Could not pin version {$version}
pin-subcommand-description = Set or update .gdvmrc with the requested version

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
help-config-key = The configuration key (e.g., github.token)
help-config-value = The value to set for the configuration key
help-config-unset-key = The configuration key to unset (e.g., github.token)
help-config-show-sensitive = Show sensitive configuration values in plaintext
help-config-available = List all available configuration keys and their values, including defaults
warning-setting-sensitive = {"\u001b"}[33mWarning: You are setting a sensitive value which will be stored in plaintext in your home directory.{"\u001b"}[0m
config-set-prompt = Please enter the value for { $key }:
error-reading-input = Error reading input
config-set-success = Configuration updated successfully.
config-unset-success = Configuration key { $key } unset successfully.
config-key-not-set = Configuration key not set.
error-unknown-config-key = Unknown configuration key.
error-invalid-config-subcommand = Invalid config subcommand. Use "get", "set", or "list".
error-parse-config = Failed to parse configuration file: { $error }
error-parse-config-using-default = {"\u001b"}[33mUsing default configuration values.{"\u001b"}[0m
error-github-api = GitHub API error: { $error }
error-github-rate-limit = GitHub API rate limit exceeded.

  To resolve this, please create a personal access token on GitHub by visiting https://github.com/settings/tokens.

  Click "Generate new token", select only the minimal permissions required (e.g. public_repo), and then set the token via the GITHUB_TOKEN environment variable or by running:

    gdvm config set github.token

  Note: The token will be stored in plaintext in your home directory. Please ensure you keep it secure.
  It is recommended to regularly review and rotate your tokens for security purposes.

