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

# Migrating to the new version syntax

gdvm 0.12.0 changes how you specify Godot versions on the command line and how pinned versions are stored on disk. This guide explains what changed and how to move to the new syntax.

The old syntax still works, so nothing breaks if you change nothing. Though, you should know that some options are now deprecated and will eventually be removed, so it's a good idea to switch to the new syntax sooner rather than later.

---

| Old                         | New                       |
| --------------------------- | ------------------------- |
| `gdvm install 4.4 --csharp` | `gdvm install csharp:4.4` |
| `gdvm run 4.4 --csharp`     | `gdvm run csharp:4.4`     |
| `gdvm pin stable --csharp`  | `gdvm pin csharp:latest`  |
| `.gdvmrc` pin file          | `gdvm.toml` pin file      |

## `csharp:` prefix replaces `--csharp`

The build flavour is now part of the version you ask for, written as a prefix. Previously you added `--csharp` to ask for a build with C# support:

```bash
gdvm install 4.4 --csharp
gdvm run 4.4 --csharp
```

Now, put `csharp:` in front of the version instead:

```bash
gdvm install csharp:4.4
gdvm run csharp:4.4
```

The `--csharp` flag still works for now, but it is deprecated. When you use it, gdvm prints a warning and treats it the same as the `csharp:` prefix. It will be removed in a future release.

The standard - that is, non-C# - build is what you get when you omit the variant, so most of the time you just write the version. You can also name it explicitly with the `default:` prefix. `default:4.4` and `4.4` mean the same thing.

```bash
gdvm run 4.4          # Standard build
gdvm run csharp:4.4   # C# build
gdvm run default:4.4  # Standard build, stated explicitly
```

## `latest` keyword and `--pre`

`latest` resolves to the newest stable release. It replaces the `stable` keyword, which still works, but is deprecated and will be removed in a future release.

```bash
gdvm install latest  # Newest stable release
```

Before, there was no way to ask for the newest pre-release or dev build short of naming an exact version.

The new `--pre` flag (or `-p`) fills that gap. It tells gdvm to consider pre-release builds when resolving `latest` or a version pattern.

```bash
gdvm install latest        # Newest stable release
gdvm install latest --pre  # Newest build of any kind (stable/rc/beta/dev)
gdvm install 4.5 --pre     # Newest 4.5 build of any kind
```

`--pre`/`-p` is available on `install`, `run`, `search`, `show`, `use`, and `pin`.

## `gdvm.toml` replaces `.gdvmrc`

`.gdvmrc` is now replaced by `gdvm.toml`. `gdvm pin` writes the pinned version to `gdvm.toml` in a new format:

```toml
[godot]
version = "csharp:4.3-stable"
```

Existing `.gdvmrc` files are still read for now, and `gdvm pin` also writes a `.gdvmrc` file in the old format so that older versions of gdvm can still read your pin. If both files are present, `gdvm.toml` takes precedence.

If you would rather only write the new file, pass `--no-legacy` to skip writing `.gdvmrc`:

```bash
gdvm pin csharp:4.3 --no-legacy
```

Reading and writing `.gdvmrc` will be dropped in a future release.

## Installation directory layout

Internally, gdvm now stores every build under a variant subdirectory. The standard build moves from `4.4.1-stable/` to `default/4.4.1-stable/`, and the C# build from `4.4.1-stable-csharp/` to `csharp/4.4.1-stable/`.

This is handled automatically the next time you run gdvm, and the old locations are kept as links so existing setups keep working.

If you use Windows, [make sure you have Developer Mode enabled](https://learn.microsoft.com/windows/advanced-settings/developer-mode#enable-developer-mode) to allow gdvm to create symlinks. If you don't have Developer Mode enabled, you'll need to run gdvm with administrator privileges the first time to create the links, or they will fail to create.
