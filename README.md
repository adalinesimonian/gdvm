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

# gdvm — Godot Version Manager

![Project banner](https://gdvm.io/gdvm-github-banner.png)

<!--[Follow on Bluesky](https://bsky.app/profile/gdvm.io)-->

[![GitHub Release](https://img.shields.io/github/v/release/adalinesimonian/gdvm)](https://github.com/adalinesimonian/gdvm/releases/latest) [![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/adalinesimonian/gdvm/build-and-test.yml?branch=main)](https://github.com/adalinesimonian/gdvm/actions/workflows/build-and-test.yml) [![License](https://img.shields.io/github/license/adalinesimonian/gdvm)](https://github.com/adalinesimonian/gdvm/blob/main/COPYING) [![Bluesky](https://img.shields.io/badge/Bluesky-follow-blue?logo=bluesky&style=social)](https://bsky.app/profile/gdvm.io) [![GitHub Stargazers](https://img.shields.io/github/stars/adalinesimonian/gdvm?style=social)](https://github.com/adalinesimonian/gdvm/stargazers)

Godot Version Manager (gdvm) is a tool designed to simplify the installation, management, and switching between different versions of the Godot Engine.

Whether you're working on multiple projects or need to test features across various Godot versions, you'll never need to manually fuss with Godot installations again.

gdvm is a community-driven project, not affiliated with Godot Engine or the Godot Foundation.

## Installation

Install on \*nix systems (including MacOS):

```bash
curl -sSL https://gdvm.io/install.sh | bash
```

Install on Windows:

```powershell
powershell -NoProfile -Command "(iwr -useb 'https://gdvm.io/install.ps1.txt').Content | iex"
```

### Supported Platforms

- Windows (64-bit, 32-bit, and 64-bit ARM)
- macOS (64-bit Intel and Apple Silicon)
- Linux (64-bit, 32-bit, and 64-bit ARM)

## Usage

Once installed, you can use the `gdvm` command to manage your Godot installations. Here are some common commands:

```bash
gdvm use latest         # Set the global default to the latest stable.
gdvm use --pre latest   # Set the global default to the latest build of any kind
                        # (stable/rc/beta/dev).

gdvm pin csharp:latest  # Pin the current folder to latest stable with C#,
                        # using a gdvm.toml file.
```

> [!TIP]
> Associate `.godot` files with `~/.gdvm/bin/godot.exe` to auto-use the correct version. gdvm can also detect the required version from `project.godot`.

```bash
gdvm run             # Run the default Godot for the folder.
godot                # Alias for `gdvm run`.
godot_console        # Windows variant keeping the console open.
gdvm run csharp:3.5  # Run Godot 3.5 with C#.
gdvm remove 3.5      # Removes Godot 3.5 without C#.
gdvm list            # List installed versions.
gdvm search 4        # Search available 4.x versions.
gdvm prune           # Remove idle installs and cached archives.
gdvm upgrade         # Upgrade gdvm.
```

> [!NOTE]
> Upgrading from an older version of gdvm? The Godot version syntax changed.
>
> The `csharp:` prefix replaces the old `--csharp` flag, and pins are stored in `gdvm.toml`.
>
> Your existing commands and `.gdvmrc` files still work. See [Migrating to the new version syntax](MIGRATION.md) for the details.

For more information, run `gdvm --help`.

### Using gdvm with debuggers

While for most purposes it is more than enough to run `godot`, the shim provided by gdvm, or `gdvm run` directly, debuggers often need to attach directly to the Godot process. To do so, they typically require a path to the Godot binary to launch or attach to.

See the dedicated guide for details and examples (including a Visual Studio Code debugger configuration): [Using gdvm with debuggers](docs/debuggers.md).

### Shell Completions

The installer sets up tab completions for your shell automatically if you are using bash, zsh, fish, or PowerShell. To set them up manually, add the matching line to your shell's profile:

```sh
eval "$(gdvm completions bash)"           # bash (~/.bashrc)
eval "$(gdvm completions zsh)"            # zsh (~/.zshrc)
gdvm completions fish | source            # fish (~/.config/fish/config.fish)
gdvm completions powershell | Out-String | Invoke-Expression  # PowerShell ($PROFILE)
```

## Registries

gdvm installs official Godot builds by default. Custom registries let you install from anywhere else, such as builds from your own CI pipeline or wherever else.

A version can be qualified by registry and variant, as `registry/variant:version`:

```bash
gdvm install 4.4               # Official registry, default variant.
gdvm install csharp:4.4        # Official registry, C# variant.
gdvm install mybuilds/4.4      # "mybuilds" registry, default variant.
gdvm install mybuilds/web:4.4  # "mybuilds" registry, "web" variant.
```

If you don't specify a registry, gdvm uses the official one. Similarly, if you don't specify a variant, gdvm uses the `default` variant.

You must add a registry to your gdvm configuration before installing from it:

```bash
gdvm registry add mybuilds https://builds.example.com/godot
# You can also add a registry on your local filesystem:
gdvm registry add mybuilds file:///home/user/godot-builds

gdvm registry list     # Show configured registries.
gdvm registry refresh  # Refresh cached index of all registries.

gdvm registry remove mybuilds
```

> [!WARNING]
> A custom registry can serve anything, even potentially malicious payloads. gdvm checks that a download matches the registry's `sha512`, but cannot guarantee the safety of anything it runs from it.
>
> Therefore, you must confirm that you trust the source of any registry the first time it's used. You will also be warned on every use of it later. You can use `-y` or `--yes` to skip the prompt in scripts. However, **make sure that you trust the source of any custom registry you add.**

### Sharing with a project

You can point to a custom registry in `gdvm.toml`. This way, everyone who clones the repo and runs `gdvm` in it will automatically use the right build from the right registry, without needing to configure it themselves.

```toml
[godot]
version = "mybuilds/4.4-stable"

[registries.mybuilds]
url = "https://builds.example.com/godot"
```

Keep in mind that when you run `gdvm pin mybuilds/4.4-stable`, it only writes the version. **You must also add a `[registries]` section to make the alias resolve for others.** Otherwise, it will use whatever `mybuilds` registry is configured in their gdvm config, or fail if they don't have one.

The first time you clone a repo with a pinned registry, gdvm will prompt you to confirm that you trust it.

### Hosting your own

A registry is essentially just a bunch of static JSON files (and optionally, the archives themselves). You can host one on any web server or keep one on the filesystem and point to it with a `file://` URL.

gdvm can build one from archives you already have:

```bash
gdvm registry init ./my-registry --name "My Builds"

# Add a build from a URL, such as a GitHub release or S3 bucket:
gdvm registry add-build ./my-registry --version 4.4-stable \
    --platform linux-x86_64 \
    --url https://builds.example.com/godot/Godot_v4.4-stable_linux.x86_64.zip

# Add a build from a local file and store it in the registry:
gdvm registry add-build ./my-registry --version 4.4-stable \
    --platform linux-x86_64 \
    --store \
    --file ./Godot_v4.4-stable_linux.x86_64.zip

gdvm registry validate ./my-registry  # Non-zero exit code if invalid.
```

You can combine this with a CI pipeline that builds Godot to your needs to automatically add the build to a registry. Then you can share that registry with your team or the public, and they can use it with gdvm without needing to build Godot themselves.

## Contributing

Please see [Contributing](CONTRIBUTING.md) for guidelines.

## Code of Conduct

Please see [Code of Conduct](CODE_OF_CONDUCT.md) for details.

## Licence

This project is licensed under the GNU General Public License v3.0 or later. See [COPYING](COPYING) for more information.
