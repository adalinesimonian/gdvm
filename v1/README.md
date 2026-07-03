# Registry v1

https://registry.gdvm.io/v1/

This directory contains the v1 Godot release registry consumed by [gdvm](https://github.com/adalinesimonian/gdvm). Older versions of gdvm use this version.

## Structure

The primary entry point is [`index.json`](index.json), which lists all known releases in order of their GitHub release IDs, from newest to oldest:

```json
[
  {"id": 222423369, "name": "4.5-dev5"},
  {"id": 218372904, "name": "4.5-dev4"},
  ...
]
```

Each entry corresponds to a file in [`releases/`](releases/) named `<id>_<name>.json`. Those files contain the full metadata for a release:

```json
{
  "id": 120827717,
  "name": "4.0-stable",
  "url": "https://github.com/godotengine/godot-builds/releases/tag/4.0-stable",
  "binaries": {
    "linux": {
      "x86_64": {
        "sha512": "...",
        "urls": [
          "https://github.com/godotengine/godot-builds/releases/download/4.0-stable/Godot_v4.0-stable_linux.x86_64.zip"
        ]
      }
    }
  }
}
```

`binaries` keys represent build variants (for example `linux`, `macos`, `windows`, or `linux-csharp`). Each architecture entry (`x86`, `x86_64`, `arm64`, `universal`) stores a SHA-512 checksum and one or more download URLs.

## Automatic updates

The registry is kept current by [an automated workflow in the main repository](https://github.com/adalinesimonian/gdvm/blob/main/.github/workflows/update-registry-v1.yml). It periodically runs [update-registry.mjs](update-registry.mjs) to fetch new Godot builds from [GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases) in the [godotengine/godot-builds repository](https://github.com/godotengine/godot-builds), updates the JSON files, and commits the results to this branch after [validating them](validate.mjs).

Every PR and push to the registry branch triggers the validation workflow defined in [.github/workflows/v1-validate.yml](../.github/workflows/v1-validate.yml) to ensure the data remains consistent.
