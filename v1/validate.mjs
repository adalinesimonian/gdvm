#!/usr/bin/env node

/*
 Validates the Godot build registry.

 Exit-code 0 - no errors (warnings allowed)
 Exit-code 1 - at least one error found
*/

import fs from "node:fs/promises";
import path from "node:path";

// Helpers.

const releasesDir = "releases";

const ok = (msg) => console.log(`✅ ${msg}`);
const warn = (msg) => console.warn(`⚠️ ${msg}`);
const err = (msg) => console.error(`❌ ${msg}`);

const warnings = [];
const errors = [];

function pushWarn(context, msg) {
  warnings.push({ context, msg });
  warn(`${context}: ${msg}`);
}
function pushErr(context, msg) {
  errors.push({ context, msg });
  err(`${context}: ${msg}`);
}

function isSha512(hex) {
  return /^[a-f0-9]{128}$/i.test(hex);
}

function validUrl(u, prefix) {
  try {
    const url = new URL(u);
    return url.protocol === "https:" && u.startsWith(prefix);
  } catch {
    return false;
  }
}

async function exists(file) {
  try {
    await fs.access(file);
    return true;
  } catch {
    return false;
  }
}
async function readJson(file) {
  try {
    const data = await fs.readFile(file, "utf8");
    return JSON.parse(data);
  } catch (e) {
    throw new Error(`invalid JSON (${e.message.split("\n")[0]})`);
  }
}

// Check index.json.

let index;
const indexPath = "index.json";

try {
  index = await readJson(indexPath);
  ok("index.json parsed");
} catch (err) {
  pushErr(indexPath, err.message);
  summary();
  process.exit(1);
}

if (!Array.isArray(index)) {
  pushErr(indexPath, "must be an array");
  summary();
  process.exit(1);
}

const seenIds = new Set();
const seenNames = new Set();

const expectedFiles = new Map(); // id -> name

for (const [i, entry] of index.entries()) {
  const ctx = `${indexPath}[${i}]`;

  if (typeof entry !== "object" || entry === null) {
    pushErr(ctx, "entry is not an object");
    continue;
  }

  const { id, name } = entry;

  if (typeof id !== "number") {
    pushErr(ctx, "`id` must be number");
  }
  if (typeof name !== "string") {
    pushErr(ctx, "`name` must be string");
  }

  if (seenIds.has(id)) {
    pushErr(ctx, `duplicate id ${id}`);
  }
  if (seenNames.has(name)) {
    pushErr(ctx, `duplicate name "${name}"`);
  }

  seenIds.add(id);
  seenNames.add(name);

  const file = path.join(releasesDir, `${id}_${name}.json`);
  expectedFiles.set(file, { id, name });
}

// Check release files.

const actualFiles = (await fs.readdir(releasesDir))
  .filter((f) => f.endsWith(".json"))
  .map((f) => path.join(releasesDir, f));

for (const file of expectedFiles.keys()) {
  if (!(await exists(file))) {
    pushErr(file, "missing release file");
  }
}

for (const file of actualFiles) {
  if (!expectedFiles.has(file)) {
    pushWarn(file, "file not referenced in index.json");
  }
}

// Validate each release file.

for (const file of expectedFiles.keys()) {
  if (!(await exists(file))) {
    continue; // Already counted as error.
  }

  let rel;

  try {
    rel = await readJson(file);
  } catch (e) {
    pushErr(file, e.message);
    continue;
  }

  const ctx = file;

  // basic fields
  if (rel.id !== expectedFiles.get(file).id) pushErr(ctx, "`id` mismatch");
  if (rel.name !== expectedFiles.get(file).name)
    pushErr(ctx, "`name` mismatch");

  if (
    typeof rel.url !== "string" ||
    !validUrl(
      rel.url,
      "https://github.com/godotengine/godot-builds/releases/tag/"
    )
  ) {
    pushErr(ctx, "`url` invalid or wrong prefix");
  }

  // Binaries (optional but normally present).
  const binaries = rel.binaries;

  if (
    !binaries ||
    typeof binaries !== "object" ||
    Object.keys(binaries).length === 0
  ) {
    pushWarn(ctx, "no binaries defined");
    continue; // Nothing more to check.
  }

  for (const [buildName, archObj] of Object.entries(binaries)) {
    if (typeof archObj !== "object" || Object.keys(archObj).length === 0) {
      pushWarn(ctx, `build "${buildName}" has no architectures`);
      continue;
    }

    for (const [arch, info] of Object.entries(archObj)) {
      const archCtx = `${ctx} → ${buildName}/${arch}`;

      if (!info || typeof info !== "object") {
        pushErr(archCtx, "architecture entry is not an object");
        continue;
      }

      if (
        !info.sha512 ||
        typeof info.sha512 !== "string" ||
        !isSha512(info.sha512)
      ) {
        pushErr(archCtx, "sha512 missing or invalid");
      }

      if (!Array.isArray(info.urls) || info.urls.length === 0) {
        pushErr(archCtx, "urls array missing or empty");
      } else {
        for (const u of info.urls) {
          if (
            !validUrl(
              u,
              "https://github.com/godotengine/godot-builds/releases/download/"
            )
          ) {
            pushErr(archCtx, `url "${u}" has wrong prefix or is not https`);
          }
        }
      }
    }
  }
}

// Print summary and exit.

function summary() {
  if (warnings.length === 0 && errors.length === 0) {
    ok("Registry validation passed.");
    return;
  }

  console.log("\n──────── Summary ────────");
  if (warnings.length) console.warn(`⚠️  ${warnings.length} warning(s)`);
  if (errors.length) console.error(`❌ ${errors.length} error(s)`);

  // Group messages.
  const group = (arr) =>
    arr.reduce((m, { context, msg }) => {
      (m[context] ??= []).push(msg);
      return m;
    }, {});

  const groupedWarn = group(warnings);
  const groupedErr = group(errors);

  for (const [ctx, msgs] of Object.entries(groupedErr)) {
    console.error(`\n${ctx}`);
    for (const m of msgs) console.error(`  ❌ ${m}`);
  }
  for (const [ctx, msgs] of Object.entries(groupedWarn)) {
    console.warn(`\n${ctx}`);
    for (const m of msgs) console.warn(`  ⚠️  ${m}`);
  }

  console.log("\n───────── End ───────────");
}

summary();
process.exit(errors.length === 0 ? 0 : 1);
