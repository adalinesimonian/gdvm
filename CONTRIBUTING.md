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
5. **Commit your changes**: Commit your changes with a clear and concise commit message.
   ```sh
   git add .
   git commit -m "Description of your changes"
   ```
6. **Push to your fork**: Push your changes to your forked repository.
   ```sh
   git push origin my-feature-branch
   ```
7. **Create a Pull Request**: Open a pull request to the main repository. Provide a detailed description of your changes.

## Building GDVM for Development

To build the project, you need to have Rust installed. You can install Rust from [rustup.rs](https://rustup.rs/).

1. **Navigate to the project directory**:
   ```sh
   cd gdvm
   ```
2. **Build and run the project**:
   ```sh
   cargo run
   ```

## Building GDVM for Release

For a release build, just add the `--release` flag to the `cargo build` command.

```sh
cargo build --release
```

The compiled binary will be available in the `target/release` directory.

## Internationalization (i18n)

GDVM supports multiple languages. If you want to add or update translations, follow these steps:

### Adding a New Language

1. **Add a new Fluent file**: Create a new Fluent file in the `i18n` directory with the appropriate locale code (e.g., `fr-FR.ftl` for French).
2. **Update the `i18n.rs` file**: Include the new Fluent file in the `src/i18n.rs` file. Remember to keep the locale variables/entries sorted alphabetically by language code.

   ```rust
   // Include the new Fluent file
   static FR_FR_FTL: &str = include_str!("../i18n/fr-FR.ftl");

   // Add the new locale to the resources array
   let resources = [
       // Other locales
       (langid!("fr-FR"), FR_FR_FTL),
   ];
   ```

### Updating Existing Translations

1. **Translate messages**: Add translations for all the keys present in the existing Fluent files.
2. **Test your translations**: Ensure that the translations are correctly loaded and displayed in the application.

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
