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
help-run-csharp-long = Run the Godot version with C# support.

    If given, value overrides the default version set with "use". Otherwise, the
    default version is used. In other words, if you set a default version with
    "use --csharp", you can try to run the same version but without C# support with
    "run --csharp false". However, it may not work as expected if the version
    without C# support is not installed. (Just run "install" to install it.)
help-version = The version to install (e.g. 4), or "stable" for the latest stable version.
help-version-long =
    The version to install (e.g. 4), or "stable" for the latest stable version.

    Examples: 4.4 will install the latest stable release of Godot 4.4. If only pre-
    release versions exist, in which case, the latest pre-release version will be
    installed. 4.3-rc will install the latest release candidate of Godot 4.3, etc.
help-version-installed = The installed version (e.g. 4.2 or 4.2-stable).

help-search = List remote releases from godot-builds
help-filter = Optional string to filter release tags
help-include-pre = Include pre-release versions (rc, beta, dev)
help-cache-only = Use only cached release information without querying the GitHub API
help-limit = Number of releases to list, default is 10. Use 0 to list all
help-clear-cache = Clears the gdvm release cache

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
no-default-set = No default version set.

installing-version = Installing version {$version}
installed-success = Successfully installed {$version}

warning-prerelease = Warning: You are installing a pre-release version ({$branch}).

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
warning-sha-sums-missing = Checksum files not found for this release. Skipping verification.

error-find-user-dirs = Failed to find user directories.

fetching-releases = Fetching releases...
releases-fetched = Releases fetched.

error-version-not-found = Version not found.
error-multiple-versions-found = Multiple versions match your request:

error-invalid-godot-version = Invalid Godot version format. Expected formats: x, x.y, x.y.z, x.y.z.w, x.y.z-tag.
error-invalid-remote-version = Invalid remote Godot version format. Expected formats: x, x.y, x.y.z, x.y.z.w, x.y.z-tag, or "stable".

running-version = Running version {$version}
no-matching-releases = No matching releases found.
available-releases = Available releases:
cache-cleared = Cache cleared successfully.

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
