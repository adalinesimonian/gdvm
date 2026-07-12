// SPDX-FileCopyrightText: Copyright (C) 2026 Adaline Simonian
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

import type { JSX } from "solid-js";
import { ToastHost } from "../components/Toast.tsx";
import "@fontsource/fira-code/400.css";
import "@fontsource/fira-code/500.css";
import "../styles/global.css";

export function Layout(props: { children?: JSX.Element }) {
  return (
    <>
      {props.children}
      <ToastHost />
    </>
  );
}
