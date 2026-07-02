#!/usr/bin/env node
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

import { execSync } from "node:child_process";

interface Index {
  schema: 2;
  releases: {
    version: string;
    variants: Record<string, string[]>;
    path: string;
  }[];
}

interface State {
  [version: string]: { id: number };
}

function sh(cmd: string): string {
  return execSync(cmd, { encoding: "utf8" }).trim();
}

let oldIndexRaw = '{"schema":2,"releases":[]}';
try {
  oldIndexRaw = sh("git show HEAD:v2/index.json");
} catch {
  // No previous commit on branch.
}

const oldIndex: Index = JSON.parse(oldIndexRaw);

// Read staged index.json.
let newIndexRaw = '{"schema":2,"releases":[]}';
try {
  newIndexRaw = sh("git show :v2/index.json");
} catch {
  newIndexRaw = oldIndexRaw;
}

const newIndex: Index = JSON.parse(newIndexRaw);

// Read state map of release names to IDs.

let oldStateRaw = "{}";
try {
  oldStateRaw = sh("git show HEAD:v2/.state.json");
} catch {
  // No previous commit on branch.
}

const oldState: State = JSON.parse(oldStateRaw);

let newStateRaw = "{}";
try {
  newStateRaw = sh("git show :v2/.state.json");
} catch {
  newStateRaw = oldStateRaw;
}

const newState: State = JSON.parse(newStateRaw);

const oldById = new Map(
  oldIndex.releases.map((release) => [
    oldState[release.version]?.id,
    release.version,
  ]),
);
const newById = new Map(
  newIndex.releases.map((release) => [
    newState[release.version]?.id,
    release.version,
  ]),
);

const added: { id: number; name: string }[] = [];
const removed: { id: number; name: string }[] = [];

// added = new files under v2/releases.
for (const [id, name] of newById) {
  if (!oldById.has(id)) {
    added.push({ id, name });
  }
}

// removed = old files under v2/releases.
for (const [id, name] of oldById) {
  if (!newById.has(id)) {
    removed.push({ id, name });
  }
}

// updated = modified files under v2/releases.
const diff = sh("git diff --name-status --cached");
const updated: { id: number; name: string }[] = [];
const otherLines: string[] = [];

diff
  .split("\n")
  .filter(Boolean)
  .forEach((line) => {
    const [status, file] = line.split(/\s+/);
    const match = file.match(/^v2\/releases\/([^.]+)\.json$/);

    if (status === "M" && match) {
      updated.push({ id: newState[match[1]]?.id, name: match[1] });
    } else if (status === "A" && match) {
      added.push({ id: newState[match[1]]?.id, name: match[1] });
    } else if (status === "D" && match) {
      removed.push({ id: oldState[match[1]]?.id, name: match[1] });
    } else if (
      !/^v2\/releases\/|^v2\/(index|registry|\.state)\.json$/.test(file)
    ) {
      otherLines.push(file);
    }
  });

// Dedupe lists to avoid double reporting.
for (const list of [added, updated, removed]) {
  const ids = new Set<number>();
  for (const item of list) {
    if (ids.has(item.id)) {
      // If we already have this ID, remove it from the list.
      const index = list.indexOf(item);
      if (index !== -1) {
        list.splice(index, 1);
      }
    } else {
      // Otherwise, add the ID to the set.
      ids.add(item.id);
    }
  }
}

// Sort all arrays by ID in descending order to have the latest changes first.
const byDesc = (
  a: { id: number; name: string },
  b: { id: number; name: string },
) => b.id - a.id;

added.sort(byDesc);
updated.sort(byDesc);
removed.sort(byDesc);

// Create the commit message body.
const body = [
  ...added.map((r) => `- Added ${r.name}`),
  ...updated.map((r) => `- Updated ${r.name}`),
  ...removed.map((r) => `- Removed ${r.name}`),
  otherLines.length ? "- Other changes" : null,
]
  .filter(Boolean)
  .join("\n");

// Create the first line of the commit message.
let first;
const actions = [
  { verb: "Add", items: added },
  { verb: "Update", items: updated },
  { verb: "Remove", items: removed },
];
const single = actions.filter((a) => a.items.length > 0);

if (single.length === 1 && !otherLines.length) {
  const { verb, items } = single[0];
  const names = items.map((r) => r.name);
  const oneliner = names.join(", ");

  first = `chore(v2): ${verb} release${
    names.length > 1 ? "s" : ""
  } ${oneliner}`;

  if (first.length > 50) {
    first = `chore(v2): ${verb} releases`;
  }
} else {
  first = "chore(v2): Update registry";
}

const commitMessage = `${first}\n\n${body}`.trim();

// Print the commit message to stdout.
console.log(commitMessage);
