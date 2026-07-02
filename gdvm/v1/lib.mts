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

import crypto from "node:crypto";
import fs from "node:fs/promises";

/** Current schema version of the gdvm release manifest. */
export const gdvmSchemaVersion = 1 as const;

/** Rust targets gdvm supports. */
export const knownTargets: readonly string[] = [
  "x86_64-unknown-linux-gnu",
  "i686-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
  "i686-pc-windows-msvc",
  "aarch64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
];

/** Build provenance metadata. */
export interface GdvmProvenance {
  attestation_url?: string;
}

/** Binary information. */
export interface GdvmBinary {
  filename?: string;
  size?: number;
  sha256?: string;
  urls: string[];
  provenance?: GdvmProvenance;
}

/** A gdvm release. */
export interface GdvmRelease {
  version: string;
  tag?: string;
  prerelease: boolean;
  binaries: Record<string, GdvmBinary>;
}

/** Parsed `releases.json`. */
export interface GdvmReleasesManifest {
  schema: typeof gdvmSchemaVersion;
  updated_at?: string;
  releases: GdvmRelease[];
}

/** Strip a leading `v` from a tag to get the bare version. */
export function versionFromTag(tag: string): string {
  return tag.replace(/^v/, "");
}

/** True when a version carries a pre-release component. */
export function isPrereleaseVersion(version: string): boolean {
  return version.includes("-");
}

/**
 * Get a Rust target from a release file name.
 */
export function targetFromAssetName(name: string): string | null {
  if (!name.startsWith("gdvm-")) {
    return null;
  }

  const target = name.slice("gdvm-".length).replace(/\.exe$/, "");

  return knownTargets.includes(target) ? target : null;
}

/** Get the GitHub release download URL for an asset. */
export function downloadUrl(
  ownerRepo: string,
  tag: string,
  filename: string,
): string {
  return `https://github.com/${ownerRepo}/releases/download/${tag}/${filename}`;
}

/** Get the lower-case hex SHA 256 sum of a file. */
export async function sha256File(path: string): Promise<string> {
  const hash = crypto.createHash("sha256");

  hash.update(await fs.readFile(path));

  return hash.digest("hex");
}

/**
 * Compare two semantic versions. Returns a negative number when `a < b`, a
 * positive number when `a > b`, and zero when equal.
 */
export function compareVersions(a: string, b: string): number {
  const parse = (v: string) => {
    const [core, pre] = v.split("-", 2);
    const nums = core.split(".").map((n) => Number.parseInt(n, 10) || 0);

    while (nums.length < 3) {
      nums.push(0);
    }

    return { nums, pre: pre ?? "" };
  };

  const pa = parse(a);
  const pb = parse(b);

  for (let i = 0; i < 3; i++) {
    if (pa.nums[i] !== pb.nums[i]) {
      return pa.nums[i] - pb.nums[i];
    }
  }

  // A version without a pre-release outranks one with a pre-release.
  if (pa.pre === "" && pb.pre !== "") {
    return 1;
  }

  if (pa.pre !== "" && pb.pre === "") {
    return -1;
  }

  if (pa.pre === pb.pre) {
    return 0;
  }

  // Compare pre-release identifiers field by field.
  const fa = pa.pre.split(".");
  const fb = pb.pre.split(".");
  const len = Math.max(fa.length, fb.length);

  for (let i = 0; i < len; i++) {
    const ia = fa[i];
    const ib = fb[i];

    if (ia === undefined) {
      return -1;
    }

    if (ib === undefined) {
      return 1;
    }

    const na = /^\d+$/.test(ia) ? Number.parseInt(ia, 10) : null;
    const nb = /^\d+$/.test(ib) ? Number.parseInt(ib, 10) : null;

    if (na !== null && nb !== null) {
      if (na !== nb) return na - nb;
    } else if (na !== null) {
      return -1; // Numeric identifiers have lower precedence than non-numeric.
    } else if (nb !== null) {
      return 1;
    } else if (ia !== ib) {
      return ia < ib ? -1 : 1;
    }
  }
  return 0;
}

/** Sort releases in place, newest first. */
export function sortReleasesDescending(releases: GdvmRelease[]): void {
  releases.sort((a, b) => compareVersions(b.version, a.version));
}

/** The GitHub `owner/repo` slug, from the environment or a default. */
export function ownerRepo(): string {
  return process.env.GITHUB_REPOSITORY || "adalinesimonian/gdvm";
}

export class GitHubClient {
  readonly headers: Record<string, string>;
  readonly #concurrency: number;
  readonly #queue: (() => void)[] = [];
  #active = 0;
  #rateLimitRemaining = Infinity;
  #rateLimitResetMs = 0;

  constructor(options: { token?: string; concurrency?: number } = {}) {
    this.headers = {
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      "User-Agent": "gdvm-registry",
    };

    if (options.token) {
      this.headers.Authorization = `Bearer ${options.token}`;
    }

    this.#concurrency = options.concurrency ?? 8;
  }

  async fetch(url: string): Promise<Response> {
    await this.acquireSlot();

    try {
      if (this.#rateLimitRemaining <= 0) {
        const waitMs = Math.max(0, this.#rateLimitResetMs - Date.now()) + 1000;

        await new Promise<void>((resolve) => setTimeout(resolve, waitMs));

        this.#rateLimitRemaining = Infinity;
      }

      const resp = await fetch(url, { headers: this.headers });

      this.applyRateLimitHeaders(resp.headers);

      return resp;
    } finally {
      this.releaseSlot();
    }
  }

  private applyRateLimitHeaders(headers: Headers): void {
    const remaining = headers.get("x-ratelimit-remaining");
    const reset = headers.get("x-ratelimit-reset");

    if (remaining !== null) {
      this.#rateLimitRemaining = Number(remaining);
    }

    if (reset !== null) {
      this.#rateLimitResetMs = Number(reset) * 1000;
    }
  }

  private acquireSlot(): Promise<void> {
    if (this.#active < this.#concurrency) {
      this.#active++;

      return Promise.resolve();
    }

    return new Promise<void>((resolve) => {
      this.#queue.push(resolve);
    });
  }

  private releaseSlot(): void {
    const next = this.#queue.shift();

    if (next) {
      next();
    } else {
      this.#active--;
    }
  }
}
