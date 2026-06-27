// SPDX-FileCopyrightText: Copyright (C) 2025 Adaline Simonian
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

import fs from "node:fs/promises";
import path from "node:path";

const skip = new Set([
  ".git",
  ".vscode",
  ".wrangler",
  "package.json",
  "package-lock.json",
  "tsconfig.json",
  ".state.json",
]);

function formatSize(bytes) {
  if (bytes < 0) {
    return ["-", ""];
  }

  const units = ["B", "KiB", "MiB", "GiB"];
  let remain = 0;
  let unit = 0;

  while (bytes >= 1024 && unit < units.length - 1) {
    remain = bytes % 1024;
    bytes = Math.floor(bytes / 1024);
    unit++;
  }

  if (unit > 0 && bytes < 10) {
    // Round up the last 0.1.
    let frac = Math.floor((remain * 5 + 256) / 512);

    if (frac >= 10) {
      bytes++;
      frac = 0;
    }

    if (bytes < 10) {
      return [`${bytes}.${frac}`, units[unit]];
    }
  }

  if (remain >= 512) {
    bytes++;
  }

  return [String(bytes), units[unit]];
}

function formatDate(mtime) {
  const iso = mtime.toISOString();
  return iso.slice(0, 16).replace("T", " ");
}

async function build(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  const subdirNames = entries
    .filter((e) => e.isDirectory() && !skip.has(e.name))
    .map((e) => e.name)
    .sort();
  const files = entries
    .filter((e) => e.isFile() && e.name.endsWith(".json") && !skip.has(e.name))
    .map((e) => e.name)
    .sort();

  const subdirResults = await Promise.all(
    subdirNames.map(async (subdir) => {
      const hasContent = await build(path.join(dir, subdir));
      return hasContent ? subdir : null;
    }),
  );
  const dirs = subdirResults.filter((subdir) => subdir !== null);

  const hasContent = dirs.length > 0 || files.length > 0;
  const isRoot = path.resolve(dir) === path.resolve(".");

  if (!hasContent && !isRoot) {
    return false;
  }

  const here = "/" + path.relative(".", dir).split(path.sep).join("/");
  const title = isRoot ? "/" : here + "/";

  const [dirStats, fileStats] = await Promise.all([
    Promise.all(
      dirs.map(async (d) => {
        const { mtime } = await fs.stat(path.join(dir, d));
        return { name: d, mtime };
      }),
    ),
    Promise.all(
      files.map(async (f) => {
        const { mtime, size } = await fs.stat(path.join(dir, f));
        return { name: f, mtime, size };
      }),
    ),
  ]);

  const rows = [];

  if (!isRoot) {
    rows.push(
      `<tr><td><a href="../">Parent Directory</a></td><td></td><td colspan="2" style="text-align:center">-</td></tr>`,
    );
  }

  for (const { name, mtime } of dirStats) {
    const dt = mtime.toISOString();
    rows.push(
      `<tr><td><a href="${name}/">${name}/</a></td><td><time datetime="${dt}">${formatDate(mtime)}</time></td><td colspan="2" style="text-align:center">-</td></tr>`,
    );
  }

  for (const { name, mtime, size } of fileStats) {
    const dt = mtime.toISOString();
    const [num, unit] = formatSize(size);
    rows.push(
      `<tr><td><a href="${name}">${name}</a></td><td><time datetime="${dt}">${formatDate(mtime)}</time></td><td>${num}</td><td>${unit}</td></tr>`,
    );
  }

  const html =
    `<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n` +
    `<title>Index of ${title}</title>\n` +
    `<style>td,th{padding-right:1em}td:nth-child(3){text-align:right;padding-right:0}td:last-child,th:last-child{padding-left:0;padding-right:0}</style>\n` +
    `</head>\n<body>\n<h1>Index of ${title}</h1>\n` +
    `<table>\n<thead>\n<tr><th scope="col">Name</th>` +
    `<th scope="col">Last modified</th><th scope="col" colspan="2">Size</th></tr>\n</thead>\n` +
    `<tbody>\n${rows.join("\n")}\n</tbody>\n</table>\n</body>\n</html>`;

  await fs.writeFile(path.join(dir, "index.html"), html);
  return true;
}

build(".").catch((err) => {
  console.error(err);
  process.exit(1);
});
