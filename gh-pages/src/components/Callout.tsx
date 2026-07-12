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

import type { Component, JSX } from "solid-js";
import { Dynamic } from "solid-js/web";
import type { LucideProps } from "lucide-solid";
import Lightbulb from "lucide-solid/icons/lightbulb";
import styles from "./Callout.module.css";

export function Callout(props: {
  icon?: Component<LucideProps>;
  children?: JSX.Element;
}) {
  return (
    <div class={styles.callout}>
      <Dynamic
        component={props.icon ?? Lightbulb}
        class={styles.icon}
        aria-hidden="true"
      />
      <div>{props.children}</div>
    </div>
  );
}
