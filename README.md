# gdvm — Godot Version Manager

![Project banner](https://gdvm.io/gdvm-github-banner.png)

<!--[Follow on Bluesky](https://bsky.app/profile/gdvm.io)-->

[![GitHub Release](https://img.shields.io/github/v/release/adalinesimonian/gdvm)](https://github.com/adalinesimonian/gdvm/releases/latest) [![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/adalinesimonian/gdvm/build-and-test.yml?branch=main)](https://github.com/adalinesimonian/gdvm/actions/workflows/build-and-test.yml) [![License](https://img.shields.io/github/license/adalinesimonian/gdvm)](https://github.com/adalinesimonian/gdvm/blob/main/LICENCE) [![Bluesky](https://img.shields.io/badge/Bluesky-follow-blue?logo=bluesky&style=social)](https://bsky.app/profile/gdvm.io) [![GitHub Stargazers](https://img.shields.io/github/stars/adalinesimonian/gdvm?style=social)](https://github.com/adalinesimonian/gdvm/stargazers)

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
gdvm use stable              # Set the global default to the latest stable.

gdvm pin stable --csharp     # Pin the current folder to latest stable with C#,
                             # using a .gdvmrc file.
```

> [!TIP]
> Associate `.godot` files with `~/.gdvm/bin/godot.exe` to auto-use the correct version. gdvm can also detect the required version from `project.godot`.

```bash
gdvm run                     # Run the default Godot for the folder.
godot                        # Alias for `gdvm run`.
godot_console                # Windows variant keeping the console open.
gdvm run 3.5 --csharp        # Run Godot 3.5 with C#.
gdvm remove 3.5              # Removes Godot 3.5 without C#.
gdvm list                    # List installed versions.
gdvm search 4                # Search available 4.x versions.
gdvm upgrade                 # Upgrade gdvm.
```

> [!NOTE]
> Hitting GitHub rate limits? Create a [fine-grained token](https://github.com/settings/personal-access-tokens/new) with access to public repositories, and run `gdvm config set github.token` (stored plaintext in `~/.gdvm/config.toml`).

For more information, run `gdvm --help`.

## Contributing

Please see [Contributing](CONTRIBUTING.md) for guidelines.

## Code of Conduct

Please see [Code of Conduct](CODE_OF_CONDUCT.md) for details.

## Licence

[ISC](LICENCE)
