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

import fs from "node:fs/promises";
import path from "node:path";
import { releaseTypeFor } from "./lib.mts";

const releasesDir = "releases";
const downloadPrefix =
  "https://github.com/godotengine/godot-builds/releases/download/";

interface Issue {
  context: string;
  msg: string;
}

const ok = (msg: string): void => console.log(`✅ ${msg}`);
const warn = (msg: string): void => console.warn(`⚠️ ${msg}`);
const err = (msg: string): void => console.error(`❌ ${msg}`);

const warnings: Issue[] = [];
const errors: Issue[] = [];

function pushWarn(context: string, msg: string): void {
  warnings.push({ context, msg });
  warn(`${context}: ${msg}`);
}
function pushErr(context: string, msg: string): void {
  errors.push({ context, msg });
  err(`${context}: ${msg}`);
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}
function isStringArray(v: unknown): v is string[] {
  return (
    Array.isArray(v) && (v as unknown[]).every((x) => typeof x === "string")
  );
}
function isSha512(v: unknown): v is string {
  return typeof v === "string" && /^[a-f0-9]{128}$/i.test(v);
}
function isIsoDate(v: unknown): v is string {
  return typeof v === "string" && !Number.isNaN(Date.parse(v));
}
function errMsg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function validUrl(u: string, prefix: string): boolean {
  try {
    const url = new URL(u);
    return url.protocol === "https:" && u.startsWith(prefix);
  } catch {
    return false;
  }
}

class FileNotFoundError extends Error {}

async function readJson(file: string): Promise<unknown> {
  let data: string;

  try {
    data = await fs.readFile(file, "utf8");
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === "ENOENT") {
      throw new FileNotFoundError(`file not found: ${file}`);
    }
    throw e;
  }

  try {
    return JSON.parse(data) as unknown;
  } catch (e) {
    throw new Error(`invalid JSON (${errMsg(e).split("\n")[0]})`);
  }
}

const releaseFileFor = (version: string): string =>
  path.posix.join(releasesDir, `${version.replace(/[^\w._]+/g, "-")}.json`);

// Check registry.json.

const manifestPath = "registry.json";

try {
  const manifest: unknown = await readJson(manifestPath);

  if (!isRecord(manifest)) {
    pushErr(manifestPath, "must be an object");
  } else {
    if (manifest.schema !== 2) {
      pushErr(manifestPath, "`schema` must be 2");
    }
    if (manifest.name !== "gdvm-official") {
      pushErr(manifestPath, '`name` must be "gdvm-official"');
    }
    if (typeof manifest.description !== "string" || !manifest.description) {
      pushErr(manifestPath, "`description` missing or empty");
    }
    if (!isIsoDate(manifest.updated_at)) {
      pushErr(manifestPath, "`updated_at` is not an ISO date");
    }
  }

  ok("registry.json parsed");
} catch (e) {
  pushErr(manifestPath, errMsg(e));
}

// Check index.json.

let index: unknown;
const indexPath = "index.json";

try {
  index = await readJson(indexPath);
  ok("index.json parsed");
} catch (e) {
  pushErr(indexPath, errMsg(e));
  summary();
  process.exit(1);
}

if (!isRecord(index) || index.schema !== 2 || !Array.isArray(index.releases)) {
  pushErr(
    indexPath,
    "must be an object with `schema` 2 and a `releases` array",
  );
  summary();
  process.exit(1);
}

const indexReleases: unknown[] = index.releases;

const seenVersions = new Set<string>();

interface ExpectedFile {
  version: string;
  variants: Record<string, string[]>;
}

const expectedFiles = new Map<string, ExpectedFile>();

for (const [i, raw] of indexReleases.entries()) {
  const ctx = `${indexPath}.releases[${i}]`;

  if (!isRecord(raw)) {
    pushErr(ctx, "entry is not an object");
    continue;
  }

  const version = raw.version;

  if (typeof version !== "string") {
    pushErr(ctx, "`version` must be string");
    continue;
  }

  const releaseType = raw.release_type;

  if (typeof releaseType !== "string") {
    pushErr(ctx, "`release_type` must be string");
  } else if (releaseType !== releaseTypeFor(version)) {
    pushErr(
      ctx,
      `release_type "${releaseType}" does not match expected "${releaseTypeFor(
        version,
      )}"`,
    );
  }

  const rawVariants = raw.variants;
  let variants: Record<string, string[]> = {};

  if (
    isRecord(rawVariants) &&
    Object.values(rawVariants).every(isStringArray)
  ) {
    variants = rawVariants as Record<string, string[]>;
  } else {
    pushErr(ctx, "`variants` must map variant -> string[]");
  }

  const expectedPath = releaseFileFor(version);

  if (raw.path !== expectedPath) {
    pushErr(ctx, `\`path\` "${String(raw.path)}" should be "${expectedPath}"`);
  }

  if (seenVersions.has(version)) {
    pushErr(ctx, `duplicate version "${version}"`);
  }
  seenVersions.add(version);

  expectedFiles.set(expectedPath, { version, variants });
}

// Check release files.

const actualFiles = (await fs.readdir(releasesDir))
  .filter((f) => f.endsWith(".json"))
  .map((f) => path.posix.join(releasesDir, f));

for (const file of actualFiles) {
  if (!expectedFiles.has(file)) {
    pushWarn(file, "file not referenced in index.json");
  }
}

// Validate each release file.

for (const [file, entry] of expectedFiles) {
  let rel: unknown;

  try {
    rel = await readJson(file);
  } catch (e) {
    pushErr(
      file,
      e instanceof FileNotFoundError ? "missing release file" : errMsg(e),
    );
    continue;
  }

  const ctx = file;

  if (!isRecord(rel)) {
    pushErr(ctx, "must be an object");
    continue;
  }

  // Basic fields.
  if (rel.schema !== 2) pushErr(ctx, "`schema` must be 2");
  if (rel.version !== entry.version) pushErr(ctx, "`version` mismatch");
  if (!isIsoDate(rel.updated_at))
    pushErr(ctx, "`updated_at` is not an ISO date");

  // Variants (optional but normally present).
  const variants = rel.variants;

  if (!isRecord(variants) || Object.keys(variants).length === 0) {
    pushWarn(ctx, "no variants defined");
    continue; // Nothing more to check.
  }

  for (const [variant, platforms] of Object.entries(variants)) {
    if (!isRecord(platforms) || Object.keys(platforms).length === 0) {
      pushWarn(ctx, `variant "${variant}" has no platforms`);
      continue;
    }

    for (const [platform, info] of Object.entries(platforms)) {
      const platCtx = `${ctx} → ${variant}/${platform}`;

      if (!isRecord(info)) {
        pushErr(platCtx, "platform entry is not an object");
        continue;
      }

      if (!isSha512(info.sha512)) {
        pushErr(platCtx, "sha512 missing or invalid");
      }

      if (!isStringArray(info.urls) || info.urls.length === 0) {
        pushErr(platCtx, "urls array missing or empty");
      } else {
        for (const u of info.urls) {
          if (!validUrl(u, downloadPrefix)) {
            pushErr(platCtx, `url "${u}" has wrong prefix or is not https`);
          }
        }
      }
    }
  }

  // Cross-check the index entry's variants/platforms against the file.
  for (const [variant, platforms] of Object.entries(variants)) {
    if (!isRecord(platforms)) {
      continue; // Already warned above.
    }

    const listed = entry.variants[variant];

    if (!listed) {
      pushErr(ctx, `variant "${variant}" not listed in index.json`);
      continue;
    }

    const have = Object.keys(platforms).sort().join(",");
    const want = [...listed].sort().join(",");

    if (have !== want) {
      pushErr(
        ctx,
        `index platforms for "${variant}" [${want}] do not match file [${have}]`,
      );
    }
  }
  for (const variant of Object.keys(entry.variants)) {
    if (!(variant in variants)) {
      pushErr(ctx, `index lists variant "${variant}" missing from file`);
    }
  }
}

// Print summary and exit.

function summary(): void {
  if (warnings.length === 0 && errors.length === 0) {
    ok("Registry validation passed.");
    return;
  }

  console.log("\n──────── Summary ────────");
  if (warnings.length) console.warn(`⚠️  ${warnings.length} warning(s)`);
  if (errors.length) console.error(`❌ ${errors.length} error(s)`);

  // Group messages.
  const group = (arr: Issue[]): Record<string, string[]> =>
    arr.reduce<Record<string, string[]>>((m, { context, msg }) => {
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
