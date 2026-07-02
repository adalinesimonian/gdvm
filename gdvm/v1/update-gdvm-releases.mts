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
import process from "node:process";
import {
  gdvmSchemaVersion,
  type GdvmBinary,
  type GdvmRelease,
  type GdvmReleasesManifest,
  downloadUrl,
  isPrereleaseVersion,
  ownerRepo,
  sha256File,
  sortReleasesDescending,
  targetFromAssetName,
  versionFromTag,
} from "./lib.mts";

const manifestPath = path.join(import.meta.dirname, "releases.json");
const repoRoot = path.join(import.meta.dirname, "..", "..");
const relManifestPath = path.relative(repoRoot, manifestPath);

interface Args {
  tag?: string;
  dist?: string;
  prerelease: boolean;
  backfill: boolean;
}

function parseArgs(argv: string[]): Args {
  const args: Args = { prerelease: false, backfill: false };

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];

    switch (arg) {
      case "--tag":
        args.tag = argv[++i];
        break;
      case "--dist":
        args.dist = argv[++i];
        break;
      case "--prerelease":
        args.prerelease = true;
        break;
      case "--backfill":
        args.backfill = true;
        break;
      default:
        throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

async function loadManifest(): Promise<GdvmReleasesManifest> {
  try {
    const text = await fs.readFile(manifestPath, "utf8");
    const parsed = JSON.parse(text) as GdvmReleasesManifest;

    if (!Array.isArray(parsed.releases)) {
      parsed.releases = [];
    }

    return parsed;
  } catch {
    return { schema: gdvmSchemaVersion, releases: [] };
  }
}

async function writeManifest(manifest: GdvmReleasesManifest): Promise<void> {
  sortReleasesDescending(manifest.releases);

  manifest.schema = gdvmSchemaVersion;
  manifest.updated_at = new Date().toISOString();

  await fs.mkdir(path.dirname(manifestPath), { recursive: true });
  await fs.writeFile(manifestPath, `${JSON.stringify(manifest, null, "\t")}\n`);
}

/** Build a release entry from locally built binaries in `dist`. */
async function releaseFromDist(
  tag: string,
  dist: string,
  prerelease: boolean,
): Promise<GdvmRelease> {
  const version = versionFromTag(tag);
  const repo = ownerRepo();
  const binaries: Record<string, GdvmBinary> = {};
  const entries = await fs.readdir(dist);

  for (const filename of entries.sort()) {
    const target = targetFromAssetName(filename);

    if (!target) {
      continue;
    }

    const filePath = path.join(dist, filename);
    const stat = await fs.stat(filePath);

    binaries[target] = {
      filename,
      size: stat.size,
      sha256: await sha256File(filePath),
      urls: [downloadUrl(repo, tag, filename)],
    };
  }

  if (Object.keys(binaries).length === 0) {
    throw new Error(`No gdvm binaries found in ${dist}`);
  }

  return {
    version,
    tag,
    prerelease: prerelease || isPrereleaseVersion(version),
    binaries,
  };
}

function upsertRelease(
  manifest: GdvmReleasesManifest,
  release: GdvmRelease,
): void {
  const idx = manifest.releases.findIndex((r) => r.version === release.version);

  if (idx >= 0) {
    manifest.releases[idx] = release;
  } else {
    manifest.releases.push(release);
  }
}

interface GitHubAsset {
  name: string;
  size: number;
  digest: string | null;
  browser_download_url: string;
}

interface GitHubRelease {
  tag_name: string;
  draft: boolean;
  prerelease: boolean;
  assets: GitHubAsset[];
}

async function fetchAllReleases(): Promise<GitHubRelease[]> {
  const repo = ownerRepo();
  const headers: Record<string, string> = {
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
    "User-Agent": "gdvm-registry",
  };

  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }

  const releases: GitHubRelease[] = [];

  for (let page = 1; ; page++) {
    const url = `https://api.github.com/repos/${repo}/releases?per_page=100&page=${page}`;
    const resp = await fetch(url, { headers });

    if (!resp.ok) {
      throw new Error(`GitHub API error ${resp.status}: ${await resp.text()}`);
    }

    const batch = (await resp.json()) as GitHubRelease[];

    if (batch.length === 0) {
      break;
    }

    releases.push(...batch);
  }
  return releases;
}

/** Build a release entry from a GitHub Releases API object. */
function releaseFromGitHub(gh: GitHubRelease): GdvmRelease | null {
  const version = versionFromTag(gh.tag_name);
  const binaries: Record<string, GdvmBinary> = {};

  for (const asset of gh.assets) {
    const target = targetFromAssetName(asset.name);

    if (!target) {
      continue;
    }

    const binary: GdvmBinary = {
      filename: asset.name,
      size: asset.size,
      urls: [asset.browser_download_url],
    };

    if (asset.digest?.startsWith("sha256:")) {
      binary.sha256 = asset.digest.slice("sha256:".length);
    }

    binaries[target] = binary;
  }

  if (Object.keys(binaries).length === 0) {
    return null;
  }

  return {
    version,
    tag: gh.tag_name,
    prerelease: gh.prerelease || isPrereleaseVersion(version),
    binaries,
  };
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));

  if (args.backfill) {
    const githubReleases = await fetchAllReleases();
    const releases: GdvmRelease[] = [];

    for (const gh of githubReleases) {
      if (gh.draft) {
        continue;
      }

      const release = releaseFromGitHub(gh);

      if (release) {
        releases.push(release);
      }
    }

    const manifest: GdvmReleasesManifest = {
      schema: gdvmSchemaVersion,
      releases,
    };

    await writeManifest(manifest);

    console.log(
      `Backfilled ${releases.length} release(s) into ${relManifestPath}.`,
    );

    return;
  }

  if (!args.tag || !args.dist) {
    console.error(
      "Usage: update-gdvm-releases.mts --tag <tag> --dist <dir> [--prerelease] | --backfill",
    );
    process.exit(1);
  }

  const manifest = await loadManifest();
  const release = await releaseFromDist(args.tag, args.dist, args.prerelease);

  upsertRelease(manifest, release);
  await writeManifest(manifest);

  console.log(
    `Updated ${relManifestPath} with ${release.version} (${
      Object.keys(release.binaries).length
    } binaries, prerelease=${release.prerelease}).`,
  );
}

await main();
