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
import { gdvmSchemaVersion } from "./lib.mts";

const manifestPath = path.join(import.meta.dirname, "releases.json");
const repoRoot = path.join(import.meta.dirname, "..", "..");
const relManifestPath = path.relative(repoRoot, manifestPath);

const errors: string[] = [];
const err = (context: string, msg: string): void => {
  errors.push(`${context}: ${msg}`);
  console.error(`❌ ${context}: ${msg}`);
};

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function isSha256(v: unknown): v is string {
  return typeof v === "string" && /^[a-f0-9]{64}$/i.test(v);
}

function isHttpsUrl(v: unknown): v is string {
  return typeof v === "string" && /^https:\/\//.test(v);
}

const raw = await fs.readFile(manifestPath, "utf8");
let manifest: unknown;

try {
  manifest = JSON.parse(raw);
} catch (e) {
  err(manifestPath, `invalid JSON: ${(e as Error).message}`);
  process.exit(1);
}

if (!isRecord(manifest)) {
  err(manifestPath, "manifest must be an object");
  process.exit(1);
}

if (manifest.schema !== gdvmSchemaVersion) {
  err(
    manifestPath,
    `schema must be ${gdvmSchemaVersion}, got ${String(manifest.schema)}`,
  );
}

if (!Array.isArray(manifest.releases)) {
  err(manifestPath, "releases must be an array");
  process.exit(1);
}

const seen = new Set<string>();
for (const release of manifest.releases as unknown[]) {
  if (!isRecord(release)) {
    err(manifestPath, "release entries must be objects");
    continue;
  }

  const version = release.version;
  const context = typeof version === "string" ? version : "<unknown>";

  if (typeof version !== "string" || version.length === 0) {
    err(context, "version must be a non-empty string");
    continue;
  }

  if (seen.has(version)) {
    err(context, "duplicate version");
  }

  seen.add(version);

  if (typeof release.prerelease !== "boolean") {
    err(context, "prerelease must be a boolean");
  }

  if (
    !isRecord(release.binaries) ||
    Object.keys(release.binaries).length === 0
  ) {
    err(context, "binaries must be a non-empty object");
    continue;
  }

  for (const [target, binary] of Object.entries(release.binaries)) {
    const bctx = `${context} / ${target}`;

    if (!isRecord(binary)) {
      err(bctx, "binary must be an object");
      continue;
    }

    if (!Array.isArray(binary.urls) || binary.urls.length === 0) {
      err(bctx, "urls must be a non-empty array");
    } else if (!binary.urls.every(isHttpsUrl)) {
      err(bctx, "every URL must be https");
    }

    if (binary.sha256 !== undefined && !isSha256(binary.sha256)) {
      err(bctx, "sha256 must be 64 hex characters");
    }

    if (binary.size !== undefined && typeof binary.size !== "number") {
      err(bctx, "size must be a number");
    }
  }
}

if (errors.length > 0) {
  console.error(`\n${errors.length} error(s) found in ${manifestPath}.`);
  process.exit(1);
}

console.log(`✅ ${relManifestPath} is valid (${seen.size} release(s)).`);
