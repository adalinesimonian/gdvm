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

import { readFile } from "node:fs/promises";
import { join } from "node:path";
import type { Plugin } from "vite";

const scriptsDir = join(import.meta.dirname, "../../scripts");
const scripts: { name: string; source: string }[] = [
  { name: "install.sh", source: "install.sh" },
  { name: "install.ps1", source: "install.ps1" },
  { name: "install.ps1.txt", source: "install.ps1" },
];

export function installScripts(): Plugin {
  const read = (source: string) => readFile(join(scriptsDir, source), "utf8");

  return {
    name: "gdvm:install-scripts",

    configureServer(server) {
      for (const { name, source } of scripts) {
        server.middlewares.use(`/${name}`, (_req, res) => {
          read(source).then(
            (contents) => {
              res.setHeader("Content-Type", "text/plain; charset=utf-8");
              res.end(contents);
            },
            () => {
              res.statusCode = 404;
              res.end();
            },
          );
        });
      }
    },

    async generateBundle() {
      for (const { name, source } of scripts) {
        this.emitFile({
          type: "asset",
          fileName: name,
          source: await read(source),
        });
      }
    },
  };
}
