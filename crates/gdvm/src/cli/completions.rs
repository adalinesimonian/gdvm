// SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of gdvm.
//
// gdvm is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use clap::ArgMatches;

/// Handle the 'completions' subcommand.
pub(crate) fn sub_completions(matches: &ArgMatches) -> Result<()> {
    let shell = *matches
        .get_one::<clap_complete::Shell>("shell")
        .expect("shell is a required argument");

    // Avoid panics in pipes by writing to a buffer first, then writing the
    // buffer to stdout.
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut super::build_cli(), "gdvm", &mut script);
    let script = augment_shell_argument(shell, script);

    use std::io::Write;
    match std::io::stdout().write_all(&script) {
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        result => Ok(result?),
    }
}

/// Work around clap_complete's lack of support for argument value candidates in
/// PowerShell and fish.
fn augment_shell_argument(shell: clap_complete::Shell, script: Vec<u8>) -> Vec<u8> {
    use clap::ValueEnum;

    let names = || {
        clap_complete::Shell::value_variants()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    };

    let Ok(text) = String::from_utf8(script) else {
        unreachable!("generated scripts are UTF-8");
    };

    let text = match shell {
        clap_complete::Shell::Fish => {
            let reference = "complete -c gdvm -n \"__fish_gdvm_using_subcommand completions\"";

            if text.contains(reference) {
                let line = format!(
                    "complete -c gdvm -n \"__fish_gdvm_using_subcommand completions\" -f -a \"{}\"\n",
                    names().join(" ")
                );
                let insert_at = text.find(reference).expect("checked above");

                format!("{}{}{}", &text[..insert_at], line, &text[insert_at..])
            } else {
                text
            }
        }
        clap_complete::Shell::PowerShell => {
            let reference = "'gdvm;completions' {";

            if let Some(pos) = text.find(reference) {
                let insert_at = pos + reference.len();
                let entries: String = names()
                    .iter()
                    .map(|name| {
                        format!(
                            "\n            [CompletionResult]::new('{name}', '{name}', [CompletionResultType]::ParameterValue, '{name}')"
                        )
                    })
                    .collect();

                format!("{}{}{}", &text[..insert_at], entries, &text[insert_at..])
            } else {
                text
            }
        }
        _ => text,
    };

    text.into_bytes()
}

#[cfg(test)]
mod tests {
    use clap::ValueEnum;
    use clap_complete::Shell;

    #[test]
    fn generates_scripts_for_all_shells() {
        for shell in Shell::value_variants() {
            let mut out = Vec::new();
            clap_complete::generate(*shell, &mut crate::cli::build_cli(), "gdvm", &mut out);
            let out = super::augment_shell_argument(*shell, out);
            let script = String::from_utf8(out).expect("script is valid UTF-8");
            assert!(script.contains("gdvm"), "{shell}: mentions the binary");
            assert!(script.contains("install"), "{shell}: lists subcommands");
            assert!(script.contains("completions"), "{shell}: completes itself");
            if matches!(
                shell,
                Shell::Bash | Shell::Zsh | Shell::Fish | Shell::PowerShell
            ) {
                assert!(
                    script.contains("powershell"),
                    "{shell}: offers the shell names for the completions argument"
                );
            }
        }
    }
}
