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
import os from "node:os";
import crypto from "node:crypto";
import {
  Worker,
  isMainThread,
  parentPort,
  workerData,
} from "node:worker_threads";
import {
  releaseTypeFor,
  type V2Release,
  type V2ReleaseIndex,
  type V2ReleaseVariants,
  type V2RegistryManifest,
} from "./lib.mts";

interface UpdateWorkerData {
  releases: ReleaseInfo[];
  owner: string;
  repo: string;
  outDir: string;
}

interface ReleaseAssetKey {
  os: string;
  arch: string;
  platform: string;
  variant: string;
}

interface ReleaseAsset {
  url: string;
  name: string;
}

interface ReleaseInfo {
  id: number;
  tag_name: string;
  assets: ReleaseAsset[];
  isRerelease?: boolean;
}

interface GitHubRelease {
  id: number;
  tag_name: string;
  assets: {
    name: string;
    browser_download_url: string;
  }[];
}

/** Map of release names (also tag names) to their GitHub release IDs. */
interface UpdateState {
  [releaseName: string]: { id: number };
}

// Shared helpers.

const githubToken =
  (await fs.readFile(".github_token", "utf8").catch(() => "")).trim() ||
  process.env.GITHUB_TOKEN ||
  "";

let state: UpdateState = {};

const saveState = async () => {
  const sorted = Object.fromEntries(
    Object.entries(state).sort(([, a], [, b]) => b.id - a.id),
  );

  state = sorted;

  await fs.writeFile(
    path.join(import.meta.dirname, ".state.json"),
    JSON.stringify(state, null, "\t") + "\n",
  );
};

const updateRegistryManifest = async () => {
  const manifest: V2RegistryManifest = {
    schema: 2,
    name: "gdvm-official",
    description:
      "Official Godot builds, mirrored from github.com/godotengine/godot-builds.",
    updated_at: new Date().toISOString(),
  };

  await fs.writeFile(
    path.join(import.meta.dirname, "registry.json"),
    JSON.stringify(manifest, null, "\t") + "\n",
  );
};

const fetchOpts = {
  headers: {
    "User-Agent": "gdvm-indexer",
    ...(githubToken ? { Authorization: `token ${githubToken}` } : {}),
  },
};

if (!githubToken) {
  console.warn(
    "Warning: no GitHub token found. Unauthenticated requests are limited to 60/hour.",
  );
}

async function fetchJSON(url: string | URL | Request) {
  const response = await fetch(url, fetchOpts);

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} - ${url}`);
  }

  return response.json();
}

async function fetchText(url: string | URL | Request) {
  const response = await fetch(url, fetchOpts);

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} - ${url}`);
  }

  return response.text();
}

async function sha512ForUrl(url: string | URL | Request) {
  const response = await fetch(url, fetchOpts);

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} - ${url}`);
  }

  if (!response.body) {
    throw new Error(`No response body for ${url}`);
  }

  const hash = crypto.createHash("sha512");

  for await (const chunk of response.body) {
    hash.update(chunk);
  }

  return hash.digest("hex");
}

const fileNameForUrl = (url: string | URL) =>
  path.basename(new URL(url).pathname);

const isX11 = (n: string) => /x11/i.test(n);
const hasExtraInfo = (n: string) =>
  /(portable|headless|server|dedicated|symbols|debug|pdb)/i.test(n);

function assetKeyFor(filename: string): ReleaseAssetKey | null {
  const name = filename.toLowerCase();

  let os = /win(dows)?/.test(name)
    ? "windows"
    : /(mac|osx)/.test(name)
      ? "macos"
      : /(linux|x11)/.test(name)
        ? "linux"
        : null;

  if (!os) {
    return null;
  }

  let arch = null;

  if (/arm64|aarch64/.test(name)) {
    arch = "arm64";
  } else if (/arm(v7|v6|32|hf)?/.test(name)) {
    return null; // Ignore 32-bit ARM, unsupported by gdvm.
  }

  if (/(?:^|[^a-z])(x86[_\-]?64|amd64|64)(?:[^a-z]|$)|win64/.test(name)) {
    arch = "x86_64";
  } else if (
    /(?:^|[^a-z])(x86|i[3-6]86|(?<!arm)32|32)(?:[^a-z]|$)|win32/.test(name)
  ) {
    arch = "x86";
  } else if (os === "macos") {
    if (/arm64|aarch64/.test(name)) {
      arch = "arm64";
    } else if (/32/.test(name)) {
      arch = "x86";
    } else if (/64/.test(name)) {
      arch = "x86_64";
    } else {
      arch = "universal"; // Universal binaries (x86_64 + arm64).
    }
  }

  if (!arch) {
    return null;
  }

  return {
    os,
    arch,
    platform: `${os}-${arch}`,
    variant: /mono/.test(name) ? "csharp" : "default",
  };
}

const sortAssets = (array: ReleaseAsset[]) =>
  [...array].sort((a, b) => {
    // Prioritize X11 builds over others (X11 is Linux, graphical).
    const x11 = Number(isX11(b.name)) - Number(isX11(a.name));

    if (x11) {
      return x11;
    }

    // Prefer assets without extra info in the name.
    // This is to prioritize the main binary over variants like "server".
    const extra = Number(hasExtraInfo(a.name)) - Number(hasExtraInfo(b.name));

    if (extra) {
      return extra;
    }

    // Shorter names first, another method to distinguish assets amongst each
    // other that may have extra info.
    return a.name.length - b.name.length;
  });

function sortObjectDeep<T extends Record<string, unknown>>(obj: T): T {
  return Object.fromEntries(
    Object.keys(obj)
      .sort()
      .map((key) => [
        key,
        obj[key] && typeof obj[key] === "object" && !Array.isArray(obj[key])
          ? sortObjectDeep(obj[key] as Record<string, unknown>)
          : obj[key],
      ]),
  ) as T;
}

const retryOperation = async <T,>(
  fn: () => Promise<T>,
  attempts: number,
  onRetry: (attempt: number, error: unknown) => void,
) => {
  let attempt = 1;

  while (true) {
    try {
      return await fn();
    } catch (err) {
      if (attempt === attempts) {
        throw err;
      }

      onRetry?.(attempt, err);
    }

    attempt++;
  }
};

const sameUrls = (a: string[] = [], b: string[] = []): boolean =>
  a.length === b.length && a.every((u) => b.includes(u));

type Status =
  | "download"
  | "retry"
  | "start"
  | "saved"
  | "skip-unknown"
  | "skip-extra"
  | "ok"
  | "updated"
  | "unchanged"
  | "changed"
  | "verified"
  | "sums-mismatch"
  | "inconsistent"
  | "error";

const Status = Object.freeze({
  Download: "download",
  Retry: "retry",
  Start: "start",
  Saved: "saved",
  SkipUnknown: "skip-unknown",
  SkipExtra: "skip-extra",
  Ok: "ok",
  Updated: "updated",
  Unchanged: "unchanged",
  Changed: "changed",
  Verified: "verified",
  SumsMismatch: "sums-mismatch",
  Inconsistent: "inconsistent",
  Error: "error",
});

const statusWidth = Math.max(...Object.values(Status).map((s) => s.length));

interface LogFields {
  status: Status;
  tag?: string;
  variant?: string;
  platform?: string;
  file?: string;
  sha?: string;
  note?: string;
}

const logColumns: {
  key: keyof LogFields;
  width: number;
  format?: (value: string) => string;
}[] = [
  { key: "status", width: statusWidth + 2, format: (s) => `[${s}]` },
  { key: "tag", width: 21 },
  { key: "variant", width: 9 },
  { key: "platform", width: 16 },
  { key: "file", width: 50 },
  { key: "sha", width: 0, format: (s) => `${s.slice(0, 8)}…` },
  { key: "note", width: 0 },
];

function formatLog(fields: LogFields): string {
  let lastAligned = -1;

  logColumns.forEach((col, i) => {
    if (col.width > 0 && fields[col.key]) {
      lastAligned = i;
    }
  });

  const cells: string[] = [];

  logColumns.forEach((col, i) => {
    const value = fields[col.key];

    if (col.width > 0) {
      if (i > lastAligned) {
        return;
      }

      const text = value ? (col.format ? col.format(value) : value) : "";

      cells.push(text.padEnd(col.width));
    } else if (value) {
      cells.push(col.format ? col.format(value) : value);
    }
  });

  return cells.join("  ").trimEnd();
}

const stickyStatuses: ReadonlySet<Status> = new Set([
  Status.Retry,
  Status.Saved,
  Status.SkipUnknown,
  Status.SkipExtra,
  Status.Ok,
  Status.Updated,
  Status.Unchanged,
  Status.Changed,
  Status.Verified,
  Status.SumsMismatch,
  Status.Inconsistent,
  Status.Error,
]);

function workerLog(payload: LogFields & { tag: string }) {
  parentPort?.postMessage({
    type: "log",
    tag: payload.tag,
    line: formatLog(payload),
    sticky: stickyStatuses.has(payload.status),
  });
}

// Worker thread code.

async function runWorker() {
  const { releases, owner, repo, outDir } = workerData as UpdateWorkerData;

  for (const release of releases) {
    parentPort?.postMessage({ type: "start", tag: release.tag_name });

    const safeName = release.tag_name.replace(/[^\w._]+/g, "-");
    const filePath = path.join(outDir, `${safeName}.json`);

    let oldRelease: V2Release | undefined;

    try {
      oldRelease = JSON.parse(await fs.readFile(filePath, "utf8"));
    } catch {
      // File missing is fine.
    }

    const oldVariants = oldRelease?.variants;

    /** Map of variant to platform to V2ReleaseAsset */
    const variants: V2ReleaseVariants = {};
    const assetUrlsByName = Object.fromEntries<string>(
      release.assets.map((asset) => [asset.name, asset.url]),
    );

    // Pull SHA512-SUMS once.
    let sums: Record<string, string> = {};
    const sumsFile = Object.keys(assetUrlsByName).find((name) =>
      /^sha512-sums.txt$/i.test(name),
    );

    if (sumsFile) {
      workerLog({
        status: Status.Download,
        tag: release.tag_name,
        note: "sha512-sums",
      });

      const sumsData = await retryOperation(
        () => fetchText(assetUrlsByName[sumsFile]),
        3,
        (attempt: number, error: unknown) => {
          const msg = error instanceof Error ? error.message : String(error);

          workerLog({
            status: Status.Error,
            tag: release.tag_name,
            note: `failed to download sums, attempt ${attempt}: ${msg}`,
          });
          workerLog({
            status: Status.Retry,
            tag: release.tag_name,
            note: `downloading sums, attempt ${attempt}`,
          });
        },
      );

      for (const line of sumsData.split("\n")) {
        const match = line.trim().match(/^([a-f0-9]{128})\s+(\S+)$/);

        if (match) {
          sums[match[2]] = match[1];
        }
      }
    }

    async function verifyAndRecord({
      prev,
      recordedSha,
      asset,
      assetKey,
      fileName,
    }: {
      prev?: { sha512: string };
      recordedSha?: string;
      asset: ReleaseAsset;
      assetKey: ReleaseAssetKey;
      fileName: string;
    }) {
      // Always download at least once for verification.
      const downloadOnce = async (purpose: string) => {
        workerLog({
          status: Status.Download,
          tag: release.tag_name,
          variant: assetKey.variant,
          platform: assetKey.platform,
          file: fileName,
          note: purpose,
        });
        return retryOperation(
          () => sha512ForUrl(asset.url),
          3,
          (attempt: number, error: unknown) => {
            const msg = error instanceof Error ? error.message : String(error);

            workerLog({
              status: Status.Error,
              tag: release.tag_name,
              variant: assetKey.variant,
              platform: assetKey.platform,
              file: fileName,
              note: `${purpose || "download"} attempt ${attempt}: ${msg}`,
            });
            workerLog({
              status: Status.Retry,
              tag: release.tag_name,
              variant: assetKey.variant,
              platform: assetKey.platform,
              file: fileName,
              note: `${purpose || "download"} attempt ${attempt}`,
            });
          },
        );
      };

      const sha1 = await downloadOnce("verify");
      let finalSha = sha1;
      let status: Status = Status.Ok;
      let note = "";

      if (recordedSha) {
        if (sha1 === recordedSha) {
          // Matches sums file. Warn if previous sum mismatched.
          if (prev && prev.sha512 !== recordedSha) {
            status = Status.Updated;
            note = `replaced previous ${prev.sha512.slice(0, 8)}…`;
          }
          finalSha = recordedSha;
        } else {
          // Mismatch. Download again to rule out network corruption.
          const sha2 = await downloadOnce("recheck");
          if (sha2 === recordedSha) {
            status = Status.Verified;
            finalSha = recordedSha;
          } else if (sha2 === sha1) {
            status = Status.SumsMismatch;
            note = `sums.txt=${recordedSha.slice(0, 8)}… dl=${sha1.slice(
              0,
              8,
            )}…`; // Keep downloaded consistent SHA.
            finalSha = sha1;
          } else {
            status = Status.Inconsistent;
            note = `sums.txt=${recordedSha.slice(0, 8)}… dl1=${sha1.slice(
              0,
              8,
            )}… dl2=${sha2.slice(0, 8)}…`;
            // For security, do not trust either mismatching SHA. Instead,
            // use the recorded SHA from sums.txt.
            finalSha = recordedSha;
          }
        }
      } else if (prev && prev.sha512 === sha1) {
        status = Status.Unchanged;
      } else if (prev && prev.sha512 !== sha1) {
        status = Status.Changed;
        note = `prev ${prev.sha512.slice(0, 8)}… -> ${sha1.slice(0, 8)}…`;
      }

      variants[assetKey.variant] ??= {};
      variants[assetKey.variant][assetKey.platform] = {
        sha512: finalSha,
        urls: [asset.url],
      };

      workerLog({
        status,
        tag: release.tag_name,
        variant: assetKey.variant,
        platform: assetKey.platform,
        file: fileName,
        sha: finalSha,
        note,
      });
    }

    for (const asset of sortAssets(release.assets)) {
      const assetKey = assetKeyFor(asset.name);

      if (!assetKey) {
        workerLog({
          status: Status.SkipUnknown,
          tag: release.tag_name,
          file: fileNameForUrl(asset.url),
        });
        continue;
      }

      // If a binary has already been picked, skip.
      if (variants[assetKey.variant]?.[assetKey.platform]) {
        workerLog({
          status: Status.SkipExtra,
          tag: release.tag_name,
          variant: assetKey.variant,
          platform: assetKey.platform,
          file: fileNameForUrl(asset.url),
        });
        continue;
      }

      variants[assetKey.variant] ??= {};

      const prev = oldVariants?.[assetKey.variant]?.[assetKey.platform];
      const urls = [asset.url];

      const fileName = fileNameForUrl(asset.url);

      // Verify even if unchanged to catch bad sums.
      if (prev && sameUrls(prev.urls, urls)) {
        await verifyAndRecord({
          prev,
          recordedSha: sums[asset.name],
          asset,
          assetKey: assetKey,
          fileName,
        });
        continue;
      }

      // Need new SHA.
      await verifyAndRecord({
        prev: undefined,
        recordedSha: sums[asset.name],
        asset,
        assetKey: assetKey,
        fileName,
      });
    }

    const sortedVariants = sortObjectDeep(variants);
    const changed =
      !oldRelease ||
      JSON.stringify(oldRelease.variants) !== JSON.stringify(sortedVariants);

    if (changed) {
      const outJson: V2Release = {
        schema: 2,
        updated_at: new Date().toISOString(),
        version: release.tag_name,
        variants: sortedVariants,
      };

      await fs.writeFile(filePath, JSON.stringify(outJson, null, "\t") + "\n");

      workerLog({ status: Status.Saved, tag: release.tag_name });
    } else {
      workerLog({ status: Status.Unchanged, tag: release.tag_name });
    }

    parentPort?.postMessage({ type: "done", tag: release.tag_name, changed });
  }
  parentPort?.close();
}

class LiveProgress {
  readonly #out: NodeJS.WriteStream;
  readonly #tty: boolean;
  readonly #total: number;
  readonly #throttleMs: number;
  #completed = 0;
  #active = new Map<string, string>();
  #pending: string[] = [];
  #lastRows = 0;
  #dirty = false;
  #timer: NodeJS.Timeout | undefined;
  readonly #onResize = () => this.#render();

  constructor(
    total: number,
    out: NodeJS.WriteStream = process.stdout,
    throttleMs = 33,
  ) {
    this.#total = total;
    this.#out = out;
    this.#throttleMs = throttleMs;
    this.#tty = Boolean(out.isTTY);

    if (this.#tty) {
      out.on("resize", this.#onResize);
    }
  }

  start(tag: string): void {
    if (!this.#tty) {
      return;
    }
    this.#active.set(tag, `[start] ${tag}`);
    this.#schedule();
  }

  log(tag: string, line: string, sticky: boolean): void {
    if (!this.#tty) {
      this.#out.write(line + "\n"); // Non-interactive, print the full stream.
      return;
    }
    if (sticky) {
      this.#pending.push(line);
    }
    if (this.#active.has(tag)) {
      this.#active.set(tag, line);
    }
    this.#schedule();
  }

  error(line: string): void {
    if (!this.#tty) {
      this.#out.write(line + "\n");
      return;
    }
    this.#pending.push(line);
    this.#schedule();
  }

  finish(tag: string, completed: number): void {
    this.#completed = completed;
    if (!this.#tty) {
      return;
    }
    this.#active.delete(tag);
    this.#schedule();
  }

  stop(): void {
    if (!this.#tty) {
      this.#out.write(
        `Done. ${this.#completed}/${this.#total} releases processed.\n`,
      );
      return;
    }
    this.#out.off("resize", this.#onResize);
    if (this.#timer) {
      clearTimeout(this.#timer);
      this.#timer = undefined;
    }
    this.#active.clear();
    const scrollback = this.#pending;
    this.#pending = [];
    this.#draw(scrollback, [this.#bar()]); // Flush and freeze final bar.
    this.#out.write("\n");
    this.#lastRows = 0;
  }

  #schedule(): void {
    if (this.#throttleMs <= 0) {
      this.#render();
      return;
    }
    if (this.#timer) {
      this.#dirty = true;
      return;
    }
    this.#render();
    this.#timer = setTimeout(() => {
      this.#timer = undefined;
      if (this.#dirty) {
        this.#dirty = false;
        this.#render();
      }
    }, this.#throttleMs);
  }

  #render(): void {
    if (!this.#tty) {
      return;
    }
    const scrollback = this.#pending;
    this.#pending = [];
    this.#draw(scrollback, this.#footer());
  }

  #draw(scrollback: string[], footer: string[]): void {
    let out = "";

    // Move the cursor to the top of the old footer and clear everything below.
    if (this.#lastRows > 0) {
      out += "\r";
      if (this.#lastRows > 1) {
        out += `\x1b[${this.#lastRows - 1}A`;
      }
      out += "\x1b[0J";
    }

    for (const line of scrollback) {
      out += line + "\n";
    }
    out += footer.join("\n");

    this.#out.write(out);
    this.#lastRows = footer.length;
  }

  #footer(): string[] {
    const cap = Math.max(1, this.#rows - 2);
    const lines = [...this.#active.values()];
    const shown = lines.slice(0, cap).map((l) => this.#fit(l));

    if (lines.length > shown.length) {
      shown.push(this.#fit(`…(+${lines.length - shown.length} more)`));
    }

    shown.push(this.#bar());
    return shown;
  }

  #bar(): string {
    const pct = this.#total ? this.#completed / this.#total : 1;
    const width = Math.min(40, Math.max(10, this.#cols - 24));
    const filled = Math.round(pct * width);
    const bar = "█".repeat(filled) + "░".repeat(width - filled);

    return this.#fit(
      `[${bar}] ${(pct * 100).toFixed(1).padStart(5)}%  ${this.#completed}/${this.#total}`,
    );
  }

  #fit(s: string): string {
    return s.length <= this.#cols
      ? s
      : s.slice(0, Math.max(0, this.#cols - 1)) + "…";
  }

  get #cols(): number {
    return this.#out.columns || 80;
  }

  get #rows(): number {
    return this.#out.rows || 24;
  }
}

// Parent thread code.

async function runParent() {
  const owner = "godotengine";
  const repo = "godot-builds";
  const indexFile = "index.json";
  const outDir = "releases";

  const args = process.argv.slice(2);
  const rebuild = args.includes("-r") || args.includes("--rebuild");

  // Read current index to know where to stop, unless rebuilding.
  let currentIndex: V2ReleaseIndex = { schema: 2, releases: [] };

  if (rebuild) {
    console.warn("Rebuilding index from scratch.");
  } else {
    try {
      currentIndex = JSON.parse(await fs.readFile(indexFile, "utf8"));
    } catch {
      // Missing index means full rebuild.
      currentIndex = { schema: 2, releases: [] };
    }
    try {
      state = JSON.parse(
        await fs.readFile(
          path.join(import.meta.dirname, ".state.json"),
          "utf8",
        ),
      );
    } catch {
      console.warn("No previous state found.");
      state = {};
    }
  }

  // For any releases not in state, we'll need to query the GitHub API to get their IDs. This is needed for incremental updates.
  const missingIds = currentIndex.releases.filter(
    (entry) => !state[entry.version],
  );

  if (missingIds.length > 0) {
    console.log(
      `Found ${missingIds.length} releases in index.json without IDs in state. Fetching IDs from GitHub API…`,
    );

    for (const entry of missingIds) {
      const url = `https://api.github.com/repos/${owner}/${repo}/releases/tags/${entry.version}`;
      const releaseData = await fetchJSON(url);
      state[entry.version] = { id: releaseData.id };
      await saveState();
      console.log(`Fetched ID ${releaseData.id} for release ${entry.version}`);
    }

    await saveState();
  }

  const knownIds = new Set(
    currentIndex.releases
      .map((entry) => state[entry.version]?.id)
      .filter(Boolean),
  );
  const knownTagNames = new Set(
    currentIndex.releases.map((entry) => entry.version),
  );

  // When doing incremental updates, always refetch the last 3 builds to fill in
  // any missing assets that may have been added after initial indexing.
  let refetchIds = new Set();

  // Sort releases by ID descending, newest first.
  currentIndex.releases.sort((a, b) => {
    const aId = state[a.version]?.id ?? 0;
    const bId = state[b.version]?.id ?? 0;
    return bId - aId; // Newest first.
  });

  if (!rebuild && currentIndex.releases.length > 0) {
    const lastBuilds = currentIndex.releases.slice(0, 3);
    refetchIds = new Set(
      lastBuilds.map((entry) => state[entry.version]?.id).filter(Boolean),
    );
    console.log(
      `Will refetch data for the last ${lastBuilds.length} builds: ${lastBuilds
        .map((b) => b.version)
        .join(", ")}`,
    );
  }

  // Fetch releases page by page until we hit the first known ID.
  const releases: ReleaseInfo[] = [];
  const perPage = 100;

  for (let page = 1; page < Infinity; page++) {
    const url = `https://api.github.com/repos/${owner}/${repo}/releases?per_page=${perPage}&page=${page}`;
    console.log(`Fetching page ${page} of releases from GitHub API…`);
    const pageReleases: GitHubRelease[] = await fetchJSON(url);

    if (!Array.isArray(pageReleases) || pageReleases.length === 0) {
      break;
    }

    for (const release of pageReleases) {
      // Include release if:
      // 1. We're rebuilding the index.
      // 2. The release is new, not in the current index.
      // 3. It's one of the last 3 releases, to refetch any missing assets.
      // 4. It's a re-release, i.e. it has the same tag name but a different ID.
      const isRerelease =
        knownTagNames.has(release.tag_name) && !knownIds.has(release.id);

      if (
        rebuild ||
        !knownIds.has(release.id) ||
        refetchIds.has(release.id) ||
        isRerelease
      ) {
        releases.push({
          id: release.id,
          tag_name: release.tag_name,
          assets: release.assets.map((a) => ({
            name: a.name,
            url: a.browser_download_url,
          })),
          isRerelease: isRerelease,
        });

        if (isRerelease) {
          console.log(
            `Found re-release: ${release.tag_name} (new ID: ${release.id})`,
          );
        }
      } else {
        console.log(
          `Found known release ID ${release.id} (${release.tag_name}) on page ${page}. Stopping.`,
        );
        page = Infinity; // Stop outer loop.
        break;
      }
    }

    if (pageReleases.length < perPage) {
      break; // Last page.
    }
  }

  // Catch releases in state but missing on disk.
  if (!rebuild) {
    const queuedIds = new Set(releases.map((r) => r.id));
    const missingOnDisk: string[] = [];

    for (const version of Object.keys(state)) {
      const filePath = path.join(
        outDir,
        `${version.replace(/[^\w._]+/g, "-")}.json`,
      );
      try {
        await fs.access(filePath);
      } catch {
        missingOnDisk.push(version);
      }
    }

    if (missingOnDisk.length > 0) {
      console.log(
        `Found ${missingOnDisk.length} releases in state but missing on disk: ` +
          `${missingOnDisk.join(", ")}. Backfilling from GitHub…`,
      );

      for (const version of missingOnDisk) {
        const id = state[version]?.id;
        if (id && queuedIds.has(id)) {
          continue; // Already queued by the page scan.
        }

        try {
          const url = `https://api.github.com/repos/${owner}/${repo}/releases/tags/${version}`;
          const releaseData = await fetchJSON(url);

          releases.push({
            id: releaseData.id,
            tag_name: releaseData.tag_name,
            assets: releaseData.assets.map(
              (a: { name: string; browser_download_url: string }) => ({
                name: a.name,
                url: a.browser_download_url,
              }),
            ),
          });
          queuedIds.add(releaseData.id);
          state[version] = { id: releaseData.id };
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          console.warn(
            `Warning: failed to backfill release ${version}: ${msg}`,
          );
        }
      }

      await saveState();
    }
  }

  // Releases are API newest to oldest, with the newest at index 0 in index.json.
  releases.sort((a, b) => b.id - a.id);

  if (releases.length === 0) {
    console.log("No new releases.");
    return;
  } else {
    console.log(`Fetched ${releases.length} releases from GitHub API.`);
  }

  await fs.mkdir(outDir, { recursive: true });

  const total = releases.length;
  const workers =
    Number(process.env.WORKERS) ||
    os.availableParallelism?.() ||
    os.cpus().length;
  const chunkSize = Math.ceil(total / workers);

  console.log(`Using ${workers} worker threads with chunk size ${chunkSize}.`);

  let completed = 0;
  let anyChanged = false;

  const progress = new LiveProgress(total);
  const pool = [];

  for (let i = 0; i < workers; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, total);

    if (start >= end) {
      break;
    }

    const worker = new Worker(new URL(import.meta.url), {
      workerData: {
        releases: releases.slice(start, end),
        owner: owner,
        repo: repo,
        outDir: outDir,
      },
    });

    worker.on("message", (msg) => {
      if (msg.type === "start") {
        progress.start(msg.tag);
      } else if (msg.type === "done") {
        completed++;
        if (msg.changed) {
          anyChanged = true;
        }
        progress.finish(msg.tag, completed);
      } else if (msg.type === "log") {
        progress.log(msg.tag, msg.line, msg.sticky);
      }
    });

    worker.on("error", (err) => {
      progress.error(
        `Worker error: ${err instanceof Error ? (err.stack ?? err.message) : String(err)}`,
      );
    });

    pool.push(new Promise((res) => worker.on("exit", res)));
  }

  await Promise.all(pool);
  progress.stop();

  // Write index.json with the release index.

  // Sort by ID descending, newest first.
  let finalIndex: V2ReleaseIndex = { schema: 2, releases: [] };
  await Promise.all(
    Array.from(Object.keys(state)).map(async (version) => {
      const releaseType = releaseTypeFor(version);
      const variants: Record<string, string[]> = {};
      const filePath = path.posix.join(
        outDir,
        `${version.replace(/[^\w._]+/g, "-")}.json`,
      );

      try {
        const releaseData: V2Release = JSON.parse(
          await fs.readFile(filePath, "utf8"),
        );

        for (const [variant, platforms] of Object.entries(
          releaseData.variants,
        )) {
          variants[variant] = Object.keys(platforms);
        }
      } catch {
        // If the release file is missing or invalid, skip it.
        console.warn(
          `Warning: Could not read release data for ${version}. Skipping.`,
        );
        return;
      }

      // relative path for the release JSON file, to be used in index.json
      const relPath = path.posix.relative(
        path.posix.dirname(indexFile),
        filePath,
      );

      finalIndex.releases.push({
        version,
        release_type: releaseType,
        variants,
        path: relPath,
      });
    }),
  );

  finalIndex.releases.sort((a, b) => {
    const aId = state[a.version]?.id ?? 0;
    const bId = state[b.version]?.id ?? 0;
    return bId - aId; // Newest first.
  });

  const indexData = JSON.stringify(finalIndex, null, "\t") + "\n";
  const indexChanged =
    (await fs.readFile(indexFile, "utf8").catch(() => "")) !== indexData;

  if (indexChanged) {
    await fs.writeFile(indexFile, indexData);
    console.log("index.json updated.");
  } else {
    console.log("index.json unchanged.");
  }

  if (anyChanged || indexChanged) {
    await updateRegistryManifest();
    console.log("registry.json updated.");
  } else {
    console.log("registry.json unchanged.");
  }
}

// Entry point.

const handlePipeError = (stream: NodeJS.WriteStream) => {
  stream?.on?.("error", (err: { code: string }) => {
    if (err.code === "EPIPE") {
      try {
        process.exit(0);
      } catch {}
    }
  });
};
handlePipeError(process.stdout);
handlePipeError(process.stderr);

if (isMainThread) {
  runParent().catch((err) => console.error(err));
} else {
  runWorker();
}
