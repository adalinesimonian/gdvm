# Registry v2

https://registry.gdvm.io/v2/

This directory contains the v2 Godot release registry consumed by [gdvm](https://github.com/adalinesimonian/gdvm). This is the current version of the registry used by gdvm v0.13.0 and above.

## Structure

### Registry manifest

[`registry.json`](registry.json) describes the registry itself:

```json
{
  "schema": 2,
  "name": "gdvm-official",
  "description": "Official Godot builds, mirrored from github.com/godotengine/godot-builds.",
  "updated_at": "2026-07-02T16:47:01.502Z"
}
```

`updated_at` _must_ be updated whenever any of the registry files are changed.

### Release index

[`index.json`](index.json) lists all known releases from newest to oldest, with a summary of available variants and platforms for each:

```json
{
  "schema": 2,
  "releases": [
    {
      "version": "4.7.1-rc1",
      "variants": {
        "csharp": ["linux-arm64", "linux-x86_64", "macos-universal", "windows-x86_64"],
        "default": ["linux-arm64", "linux-x86_64", "macos-universal", "windows-x86_64"]
      },
      "path": "releases/4.7.1-rc1.json"
    },
    ...
  ]
}
```

Each entry's `variants` object maps variant names, such as `default` or `csharp`, to the list of platform/architecture strings available for that variant.

### Release files

Each entry in the index has a `path` pointing to a file in [`releases/`](releases/) named `<version>.json`. Those files contain complete download information:

```json
{
  "schema": 2,
  "updated_at": "2026-06-26T23:43:13.641Z",
  "version": "4.0-stable",
  "variants": {
    "default": {
      "linux-x86_64": {
        "sha512": "...",
        "urls": [
          "https://github.com/godotengine/godot-builds/releases/download/4.0-stable/Godot_v4.0-stable_linux.x86_64.zip"
        ]
      },
      "macos-universal": {
        "sha512": "...",
        "urls": [
          "https://github.com/godotengine/godot-builds/releases/download/4.0-stable/Godot_v4.0-stable_macos.universal.zip"
        ]
      }
    },
    "csharp": {
      "linux-x86_64": {
        "sha512": "...",
        "urls": [
          "https://github.com/godotengine/godot-builds/releases/download/4.0-stable/Godot_v4.0-stable_mono_linux_x86_64.zip"
        ]
      }
    }
  }
}
```

`variants` keys denote the build variants. Each platform/architecture key stores a SHA-512 checksum and one or more download URLs.

## Automatic updates

The registry is kept current by [an automated workflow in the main repository](https://github.com/adalinesimonian/gdvm/blob/main/.github/workflows/update-registry-v2.yml). It periodically runs [update-registry.mts](update-registry.mts) to fetch new Godot builds from [GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases) in the [godotengine/godot-builds repository](https://github.com/godotengine/godot-builds), updates the JSON files, and commits the results to this branch after [validating them](validate.mts).
