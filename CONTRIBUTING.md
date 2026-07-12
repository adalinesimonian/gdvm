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

# Contributing to Godot Version Manager

Thank you for considering contributing to GDVM! Here are some guidelines to help you get started.

## How to Contribute

1. **Fork the repository**: Click the "Fork" button at the top right of the repository page.
2. **Clone your fork**: Clone your forked repository to your local machine.
   ```sh
   git clone https://github.com/adalinesimonian/gdvm.git
   ```
3. **Create a branch**: Create a new branch for your feature or bugfix.
   ```sh
   git checkout -b my-feature-branch
   ```
4. **Make your changes**: Implement your feature or fix the bug.
5. **Commit your changes**: Commit your changes with a clear and concise commit message. All commits must be [signed](#signing-your-commits).
   ```sh
   git add .
   git commit -m "Description of your changes"
   ```
6. **Push to your fork**: Push your changes to your forked repository.
   ```sh
   git push origin my-feature-branch
   ```
7. **Create a Pull Request**: Open a pull request to the main repository. Provide a detailed description of your changes.

## Signing Your Commits

All commits in this repository must be signed and verified. Pull requests containing unsigned commits can't be merged, so you'll be asked to re-sign them before review can be completed.

You can sign commits with either an SSH or a GPG key. SSH is the quickest to set up. GitHub's documentation covers the full process in [About commit signature verification](https://docs.github.com/authentication/managing-commit-signature-verification/about-commit-signature-verification) and [Telling Git about your signing key](https://docs.github.com/authentication/managing-commit-signature-verification/telling-git-about-your-signing-key).

### Quick Start with an SSH Key

```sh
# Reuse an existing SSH key, or create one:
ssh-keygen -t ed25519 -C "your@commit-email-address.com"

# Tell Git to sign commits with it:
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
```

Then add the public key to GitHub under Settings -> SSH and GPG keys -> New SSH key. Make sure you choose "Signing Key" as the key type.

Signing and authentication keys are tracked separately, so you need to add it here even if the same key is already used in your account for authentication.

## Building GDVM for Development

To build the project, you need to have Rust installed. You can install Rust from [rustup.rs](https://rustup.rs/).

From the root of the repository, you can build and run the project with:

```sh
cargo run -p gdvm
```

## Building GDVM for Release

For a release build, just add the `--release` flag to the `cargo build` command.

```sh
cargo build -p gdvm --release
```

The compiled binary will be available in the `target/release` directory.

## Package Registry

GDVM uses a separate Git branch named [`registry`](https://github.com/adalinesimonian/gdvm/tree/registry) to store a machine-readable list of Godot versions. This branch is updated automatically and usually should not be modified directly by contributors. For more details on its structure and how it works, please see the [registry's `README.md`](https://github.com/adalinesimonian/gdvm/tree/registry?tab=readme-ov-file#gdvm-package-registry).

## Website

The website is built from`gh-pages/`. It's a static site that uses [Vike](https://vike.dev/) and [SolidJS](https://www.solidjs.com/). To run it locally, you need to have [Node.js](https://nodejs.org/) and [Yarn](https://yarnpkg.com/) installed (this repo configures [Proto](https://moonrepo.dev/proto) to use the correct Node version automatically, so you can just install Proto to avoid managing the versions yourself).

```sh
cd gh-pages

yarn install
yarn dev        # Start the dev server.
yarn build      # Build the site to gh-pages/dist/client.
yarn typecheck  # Check types.
```

Any content that is specific to the current release, such as for example install scripts or documentation, needs to be separated from the rest of the site into different files and listed in [`gh-pages/release-content.json`](gh-pages/release-content.json).

The build process will write over these files with the content from the latest release's tag, so that the site always shows the correct install commands and documentation for the currently published release, rather than whatever is in `main`.

## Internationalization (i18n)

GDVM supports multiple languages using the [Fluent](https://projectfluent.org/) localization system. If you want to add or update translations, follow these steps:

### Adding a New Language

1. **Add a new Fluent file**: Create a new Fluent file in the `crates/gdvm/i18n` directory with the appropriate locale code (e.g., `fr-FR.ftl` for French).
2. **Update the `i18n.rs` file**: Include the new Fluent file in `crates/gdvm/src/i18n.rs`. Remember to keep the locale variables/entries sorted alphabetically by language code.

   ```rust
   // Include the new Fluent file
   static FR_FR_FTL: &str = include_str!("../i18n/fr-FR.ftl");
   // ...

   // Add the new locale to the resources array
   let resources = [
       // Other locales
       (langid!("fr-FR"), FR_FR_FTL),
       // ...
   ];
   ```

### Updating Existing Translations

1. **Translate messages**: Add translations for all the keys present in the existing Fluent files.
2. **Test your translations**: Ensure that the translations are correctly loaded and displayed in the application.
3. **Check for missing translations**: You can use the scripts in the `scripts/` directory to find missing translations.

   ```sh
   # For Linux/macOS
   ./scripts/find-missing-i18n.sh
   ```

   ```powershell
   # For Windows PowerShell
   .\scripts\find-missing-i18n.ps1
   ```

4. **Format and sort translations**: This script will help you format and sort the translations in the Fluent files using the `en-US.ftl` file as a reference. It requires that the [pwsh](https://github.com/PowerShell/PowerShell) command is available in your PATH.

   ```sh
   # For PowerShell
   ./scripts/sort-i18n.ps1          # Checks the Fluent bundles for sort/format issues
   ./scripts/sort-i18n.ps1 --format # Formats/sorts the Fluent bundles
   ./scripts/sort-i18n.sh           # Alias for *nix systems, calls the PowerShell script
   ```

## Testing

GDVM has two types of tests: unit tests and integration tests.

- **Unit tests** cover individual components and do not require any special setup.
- **Integration tests** test the entire application and interact with the file system.

By default, `cargo test` runs both unit and integration tests. However, for the integration tests to function correctly and not interfere with your personal GDVM settings, the `integration-tests` feature flag is required. This flag ensures that tests use a temporary directory for GDVM's home directory.

**To run all tests correctly, you must use:**

```sh
cargo test --features integration-tests
```

Running `cargo test` without this flag may cause integration tests to fail or modify your local GDVM settings.

If you wish to run only the unit tests, you can use:

```sh
cargo test --lib
```

When adding new tests, please consider whether they should be unit or integration tests. If a test needs to interact with the file system - for example, by installing or using a Godot version - it should be an integration test.

## Code Style

- Follow the existing code style and conventions.
- Write clear and concise comments.
- Ensure your code is well-documented.

## Reporting Issues

- Use the [issue tracker](https://github.com/adalinesimonian/gdvm/issues) to report bugs or request features.
- Provide as much detail as possible, including steps to reproduce the issue.

## Code of Conduct

- Be respectful and considerate of others.
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md).

Thank you for your contributions!
