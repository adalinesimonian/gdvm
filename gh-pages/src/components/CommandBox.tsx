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

import { toast } from "./Toast.tsx";
import styles from "./CommandBox.module.css";

export function CommandBox(props: { command: string }) {
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(props.command);
      toast("Copied!");
    } catch {
      toast("Failed to copy");
    }
  };

  return (
    <div class={styles.commandBox} onClick={copy}>
      {props.command}
    </div>
  );
}
