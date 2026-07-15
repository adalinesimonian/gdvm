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

import { createSignal, onMount } from "solid-js";
import { CommandBox } from "./CommandBox.tsx";
import { installCommands } from "../content/release.tsx";
import styles from "./InstallTabs.module.css";

type Platform = "nix" | "win";

export function InstallTabs() {
  const [platform, setPlatform] = createSignal<Platform>("nix");

  onMount(() => {
    if (/windows|win32|win64/i.test(navigator.userAgent)) {
      setPlatform("win");
    }
  });

  const tab = (id: Platform, label: string) => (
    <button
      class={styles.tabButton}
      id={`tab-${id}`}
      role="tab"
      aria-controls={`panel-${id}`}
      aria-selected={String(platform() === id) as "true" | "false"}
      onClick={() => setPlatform(id)}
    >
      {label}
    </button>
  );

  const panel = (id: Platform, command: string) => (
    <div
      id={`panel-${id}`}
      class={styles.panel}
      data-active={String(platform() === id)}
      role="tabpanel"
      aria-labelledby={`tab-${id}`}
    >
      <CommandBox command={command} />
    </div>
  );

  return (
    <>
      <div class={styles.tabs} role="tablist">
        {tab("nix", "macOS / Linux")}
        {tab("win", "Windows")}
      </div>
      {panel("nix", installCommands.nix)}
      {panel("win", installCommands.win)}
    </>
  );
}
