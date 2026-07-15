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

import { For } from "solid-js";
import { createStore, produce } from "solid-js/store";
import styles from "./Toast.module.css";

interface Toast {
  id: number;
  message: string;
  leaving: boolean;
}

const [toasts, setToasts] = createStore<Toast[]>([]);
let nextId = 0;

export function toast(message: string) {
  const id = nextId++;
  setToasts(produce((list) => list.push({ id, message, leaving: false })));

  setTimeout(() => {
    setToasts((t) => t.id === id, "leaving", true);

    setTimeout(() => {
      setToasts(
        produce((list) => {
          const index = list.findIndex((t) => t.id === id);

          if (index !== -1) {
            list.splice(index, 1);
          }
        }),
      );
    }, 300);
  }, 2400);
}

export function ToastHost() {
  return (
    <For each={toasts}>
      {(t) => (
        <div class={styles.toast} data-leaving={String(t.leaving)}>
          {t.message}
        </div>
      )}
    </For>
  );
}
