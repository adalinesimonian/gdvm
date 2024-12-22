# Godot Version Manager

Godot Version Manager (gdvm) is a tool designed to simplify the installation, management, and switching between different versions of the Godot Engine.

Whether you're working on multiple projects or need to test features across various Godot versions, you'll never need to manually fuss with Godot installations again.

## Installation

Install on \*nix systems (including MacOS):

```bash
curl -sSL https://raw.githubusercontent.com/adalinesimonian/gdvm/main/scripts/install.sh | bash
```

Install on Windows:

```powershell
powershell -Command "iwr -useb https://raw.githubusercontent.com/adalinesimonian/gdvm/main/scripts/install.ps1 | iex"
```

## Usage

```bash
gdvm install 4.3
gdvm use 4.3
gdvm remove 4.3
gdvm list
```

For more information, run

```bash
gdvm --help
```

## Contributing

Please see [Contributing](CONTRIBUTING.md) for guidelines.

## Code of Conduct

Please see [Code of Conduct](CODE_OF_CONDUCT.md) for details.

## Licence

[ISC](LICENCE)
