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
import snappy from "snappy";
import {
  GitHubClient,
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

async function writeManifest(manifest: GdvmReleasesManifest): Promise<boolean> {
  sortReleasesDescending(manifest.releases);

  try {
    const existing = JSON.parse(
      await fs.readFile(manifestPath, "utf8"),
    ) as GdvmReleasesManifest;

    if (Array.isArray(existing.releases)) {
      sortReleasesDescending(existing.releases);

      if (
        JSON.stringify(manifest.releases) === JSON.stringify(existing.releases)
      ) {
        return false;
      }
    }
  } catch {
    // Something wrong with file.
  }

  manifest.schema = gdvmSchemaVersion;
  manifest.updated_at = new Date().toISOString();

  await fs.mkdir(path.dirname(manifestPath), { recursive: true });
  await fs.writeFile(manifestPath, `${JSON.stringify(manifest, null, "\t")}\n`);

  return true;
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

interface GitHubAttestation {
  bundle_url: string;
}

async function fetchAndCacheAttestations(
  repo: string,
  digest: string,
  tag: string,
  client: GitHubClient,
): Promise<string | undefined> {
  const apiUrl = `https://api.github.com/repos/${repo}/attestations/${digest}`;
  const resp = await client.fetch(apiUrl);

  if (!resp.ok) {
    return undefined;
  }

  const data = (await resp.json()) as { attestations?: GitHubAttestation[] };

  if (!Array.isArray(data.attestations) || data.attestations.length === 0) {
    return undefined;
  }

  const lines: string[] = [];

  for (const attestation of data.attestations) {
    const bundleResp = await fetch(attestation.bundle_url);

    if (!bundleResp.ok) {
      continue;
    }

    const compressed = Buffer.from(await bundleResp.arrayBuffer());
    const raw = (await snappy.uncompress(compressed, {
      asBuffer: false,
    })) as string;

    lines.push(JSON.stringify(JSON.parse(raw)));
  }

  if (lines.length === 0) {
    return undefined;
  }

  const sha256hex = digest.slice("sha256:".length);
  const outDir = path.join(import.meta.dirname, tag);
  const outFile = path.join(outDir, `${sha256hex}.jsonl`);

  await fs.mkdir(outDir, { recursive: true });
  await fs.writeFile(outFile, lines.join("\n") + "\n");

  return `${tag}/${sha256hex}.jsonl`;
}

async function fetchAllReleases(
  client: GitHubClient,
): Promise<GitHubRelease[]> {
  const repo = ownerRepo();
  const releases: GitHubRelease[] = [];

  for (let page = 1; ; page++) {
    const url = `https://api.github.com/repos/${repo}/releases?per_page=100&page=${page}`;
    const resp = await client.fetch(url);

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
async function releaseFromGitHub(
  gh: GitHubRelease,
  client: GitHubClient,
): Promise<GdvmRelease | null> {
  const version = versionFromTag(gh.tag_name);
  const repo = ownerRepo();
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

      const attestationUrl = await fetchAndCacheAttestations(
        repo,
        asset.digest,
        gh.tag_name,
        client,
      );

      if (attestationUrl) {
        binary.provenance = { attestation_url: attestationUrl };
      }
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
    const client = new GitHubClient({
      token: process.env.GITHUB_TOKEN,
      concurrency: 8,
    });
    const allReleases = await fetchAllReleases(client);
    const publishedReleases: GitHubRelease[] = [];
    let maxTagLength = 0;

    for (const gh of allReleases) {
      if (!gh.draft) {
        publishedReleases.push(gh);

        if (gh.tag_name.length > maxTagLength) {
          maxTagLength = gh.tag_name.length;
        }
      }
    }

    const total = publishedReleases.length;
    const countWidth = `[${total}/${total}]`.length + 1;
    const tagWidth = maxTagLength + 2;

    console.log(
      `Fetched ${allReleases.length} release(s) from GitHub (${total} published).`,
    );

    const releases: GdvmRelease[] = [];
    let completed = 0;

    await Promise.all(
      publishedReleases.map(async (gh) => {
        const release = await releaseFromGitHub(gh, client);
        const n = ++completed;

        if (release) {
          releases.push(release);
          const withAttestation = Object.values(release.binaries).filter(
            (b) => b.provenance,
          ).length;
          const binaryCount = Object.keys(release.binaries).length;
          const attestationNote =
            withAttestation > 0
              ? `, ${withAttestation}/${binaryCount} attested`
              : "";
          console.log(
            `  ${`[${n}/${total}]`.padEnd(countWidth)}${gh.tag_name.padEnd(tagWidth)}${binaryCount} binaries${attestationNote}`,
          );
        } else {
          console.log(
            `  ${`[${n}/${total}]`.padEnd(countWidth)}${gh.tag_name.padEnd(tagWidth)}skipped, no matching assets`,
          );
        }
      }),
    );

    const manifest: GdvmReleasesManifest = {
      schema: gdvmSchemaVersion,
      releases,
    };

    const written = await writeManifest(manifest);

    if (written) {
      console.log(
        `Backfilled ${releases.length} release(s) into ${relManifestPath}.`,
      );
    } else {
      console.log(`No changes detected. ${relManifestPath} not updated.`);
    }

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

  const written = await writeManifest(manifest);

  if (written) {
    console.log(
      `Updated ${relManifestPath} with ${release.version} (${Object.keys(release.binaries).length} binaries, prerelease=${release.prerelease}).`,
    );
  } else {
    console.log(
      `No changes detected for ${release.version}. ${relManifestPath} not updated.`,
    );
  }
}

await main();
