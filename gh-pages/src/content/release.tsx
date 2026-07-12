// SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
// SPDX-License-Identifier: CC-BY-SA-4.0 AND GPL-3.0-or-later

// Stuff in this file is specific to the current gdvm release. It is safe to
// make changes here. On publish, the version of this file from the latest
// release will be used.

import { Callout } from "../components/Callout.tsx";

export const installCommands = {
  nix: "curl -sSL https://gdvm.io/install.sh | bash",
  win: `powershell -NoProfile -Command "(iwr -useb 'https://gdvm.io/install.ps1.txt').Content | iex"`,
};

export function Usage() {
  return (
    <>
      <h2>Getting Started</h2>
      <p>
        Once installed, you can use the <code>gdvm</code> command to manage your
        Godot installations. Here are some common commands:
      </p>
      <ul>
        <li>
          <code>gdvm use latest</code> — Set the global default to the latest
          stable.
        </li>
        <li>
          <code>gdvm pin csharp:latest</code> — Pin the current folder to latest
          stable with C#, using a <code>.gdvmrc</code> file.
        </li>
      </ul>

      <Callout>
        Associate <code>.godot</code> files with{" "}
        <code>~/.gdvm/bin/godot.exe</code> to auto-use the correct version. gdvm
        can also detect the required version from <code>project.godot</code>.
      </Callout>

      <ul>
        <li>
          <code>gdvm run</code> — Run the default Godot for the folder.
        </li>
        <li>
          <code>godot</code> — Alias for <code>gdvm run</code>.
        </li>
        <li>
          <code>godot_console</code> — Windows variant keeping the console open.
        </li>
        <li>
          <code>gdvm run csharp:3.5</code> — Run Godot 3.5 with C#.
        </li>
        <li>
          <code>gdvm remove 3.5</code> — Remove Godot 3.5 without C#.
        </li>
        <li>
          <code>gdvm list</code> — List installed versions.
        </li>
        <li>
          <code>gdvm search 4</code> — Search available 4.x versions.
        </li>
        <li>
          <code>gdvm upgrade</code> — Upgrade gdvm.
        </li>
      </ul>

      <p>
        For more information, run <code>gdvm --help</code>.
      </p>

      <p>
        For examples of using gdvm with debuggers, see the{" "}
        <a
          href="https://github.com/adalinesimonian/gdvm/blob/main/docs/debuggers.md"
          rel="noopener"
        >
          guide to using gdvm with debuggers
        </a>
        .
      </p>
    </>
  );
}
