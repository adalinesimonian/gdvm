#!/usr/bin/env node

import { execSync } from "node:child_process";

function sh(cmd) {
  return execSync(cmd, { encoding: "utf8" }).trim();
}

let oldIndexRaw = "[]";
try {
  oldIndexRaw = sh("git show HEAD:v1/index.json");
} catch {
  // No previous commit on branch.
}

const oldIndex = JSON.parse(oldIndexRaw);

// Read staged index.json.
let newIndexRaw = "[]";
try {
  newIndexRaw = sh("git show :v1/index.json");
} catch {
  newIndexRaw = oldIndexRaw;
}
const newIndex = JSON.parse(newIndexRaw);

const oldById = new Map(oldIndex.map((release) => [release.id, release.name]));
const newById = new Map(newIndex.map((release) => [release.id, release.name]));

const added = [];
const removed = [];

// added = new files under v1/releases.
for (const [id, name] of newById) {
  if (!oldById.has(id)) {
    added.push({ id, name });
  }
}

// removed = old files under v1/releases.
for (const [id, name] of oldById) {
  if (!newById.has(id)) {
    removed.push({ id, name });
  }
}

// updated = modified files under v1/releases.
const diff = sh("git diff --name-status --cached");
const updated = [];
const otherLines = [];

diff
  .split("\n")
  .filter(Boolean)
  .forEach((line) => {
    const [status, file] = line.split(/\s+/);
    const match = file.match(/^v1\/releases\/(\d+)_([^/]+)\.json$/);

    if (status === "M" && match) {
      updated.push({ id: Number(match[1]), name: match[2] });
    } else if (status === "A" && match) {
      added.push({ id: Number(match[1]), name: match[2] });
    } else if (status === "D" && match) {
      removed.push({ id: Number(match[1]), name: match[2] });
    } else if (!/^v1\/releases\/|^v1\/index\.json$/.test(file)) {
      otherLines.push(file);
    }
  });

// Dedupe lists to avoid double reporting.
for (const list of [added, updated, removed]) {
  const ids = new Set();
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
const byDesc = (a, b) => b.id - a.id;

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

  first = `chore(v1): ${verb} release${
    names.length > 1 ? "s" : ""
  } ${oneliner}`;

  if (first.length > 50) {
    first = `chore(v1): ${verb} releases`;
  }
} else {
  first = "chore(v1): Update registry";
}

const commitMessage = `${first}\n\n${body}`.trim();

// Print the commit message to stdout.
console.log(commitMessage);
