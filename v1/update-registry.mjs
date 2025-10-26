#!/usr/bin/env node

/*
  update-registry.mjs - Builds a JSON-based registry for Godot releases
  ───────────────────────────────────────────────────────────────────────
  Copyright © 2025 Adaline Simonian <adalinesimonian@pm.me>
  ───────────────────────────────────────────────────────────────────────
  - Fetches release data from the GitHub API.
  - Incremental by default; full rebuild with -r / --rebuild.
  - Parallel worker pool using worker threads.
*/

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

// Shared helpers.

const githubToken =
  (await fs.readFile(".github_token", "utf8").catch(() => "")).trim() ||
  process.env.GITHUB_TOKEN ||
  "";

const fetchOpts = {
  headers: {
    "User-Agent": "gdvm-indexer",
    ...(githubToken ? { Authorization: `token ${githubToken}` } : {}),
  },
};

if (!githubToken) {
  console.warn(
    "Warning: no GitHub token found. Unauthenticated requests are limited to 60/hour."
  );
}

async function fetchJSON(url) {
  const response = await fetch(url, fetchOpts);

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} - ${url}`);
  }

  return response.json();
}

async function fetchText(url) {
  const response = await fetch(url, fetchOpts);

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} - ${url}`);
  }

  return response.text();
}

async function sha512ForUrl(url) {
  const response = await fetch(url, fetchOpts);

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} - ${url}`);
  }

  const hash = crypto.createHash("sha512");

  for await (const chunk of response.body) {
    hash.update(chunk);
  }

  return hash.digest("hex");
}

const fileNameForUrl = (url) => path.basename(new URL(url).pathname);

const isX11 = (n) => /x11/i.test(n);
const hasExtraInfo = (n) =>
  /(portable|headless|server|dedicated|symbols|debug|pdb)/i.test(n);

function slotFor(filename) {
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

  if (/mono/.test(name)) {
    os = `${os}-csharp`;
  }

  return { os, arch };
}

const sortAssets = (array) =>
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

function sortObjectDeep(obj) {
  return Object.fromEntries(
    Object.keys(obj)
      .sort()
      .map((key) => [
        key,
        obj[key] && typeof obj[key] === "object" && !Array.isArray(obj[key])
          ? sortObjectDeep(obj[key])
          : obj[key],
      ])
  );
}

const retryOperation = async (fn, attempts, onRetry) => {
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

const sameUrls = (a = [], b = []) =>
  a.length === b.length && a.every((u) => b.includes(u));

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
function statusCell(s) {
  return `[${s}]` + " ".repeat(statusWidth - s.length + 1);
}

function formatLog({ status, tag, osArch, file, sha, note }) {
  const cols = [
    statusCell(status),
    tag ? tag.padEnd(21) : "".padEnd(21),
    osArch ? osArch.padEnd(24) : "".padEnd(24),
    file ? file.padEnd(50) : "",
  ];

  if (sha) {
    cols.push(sha.slice(0, 8) + "…");
  }
  if (note) {
    cols.push(note);
  }

  return cols.filter(Boolean).join("  ");
}

function workerLog(payload) {
  parentPort.postMessage({ type: "log", msg: formatLog(payload) });
}

// Worker thread code.

async function runWorker() {
  const { releases, owner, repo, outDir } = workerData;

  for (const release of releases) {
    parentPort.postMessage({ type: "start", tag: release.tag_name });

    const safeName = release.tag_name.replace(/[^\w.-]+/g, "_");
    const filePath = path.join(outDir, `${release.id}_${safeName}.json`);

    // If this is a re-release, find and delete the old file.
    if (release.isRerelease) {
      try {
        const files = await fs.readdir(outDir);
        const oldFilePattern = new RegExp(
          `^\\d+_${safeName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\.json$`
        );

        for (const file of files) {
          if (oldFilePattern.test(file) && !file.startsWith(`${release.id}_`)) {
            const oldFilePath = path.join(outDir, file);
            await fs.unlink(oldFilePath);
            workerLog({
              status: Status.Updated,
              tag: release.tag_name,
              note: `removed old file ${file}`,
            });
            break; // Should only be one old file with this tag name.
          }
        }
      } catch (error) {
        workerLog({
          status: Status.Error,
          tag: release.tag_name,
          note: `failed to delete old file: ${error.message}`,
        });
      }
    }

    let oldBinaries = {};

    try {
      oldBinaries =
        JSON.parse(await fs.readFile(filePath, "utf8")).binaries || {};
    } catch {
      // File missing is fine.
    }

    const binaries = {};
    const byName = Object.fromEntries(
      release.assets.map((asset) => [asset.name, asset.url])
    );

    // Pull SHA512-SUMS once.
    let sums = {};
    const sumsFile = Object.keys(byName).find((name) =>
      /^sha512-sums.txt$/i.test(name)
    );

    if (sumsFile) {
      workerLog({
        status: Status.Download,
        tag: release.tag_name,
        note: "sha512-sums",
      });

      const sumsData = await retryOperation(
        () => fetchText(byName[sumsFile]),
        3,
        (attempt, error) => {
          workerLog({
            status: Status.Error,
            tag: release.tag_name,
            note: `failed to download sums, attempt ${attempt}: ${
              error?.message || String(error)
            }`,
          });
          workerLog({
            status: Status.Retry,
            tag: release.tag_name,
            note: `downloading sums, attempt ${attempt}`,
          });
        }
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
      slot,
      fileName,
    }) {
      // Always download at least once for verification.
      const downloadOnce = async (purpose) => {
        workerLog({
          status: Status.Download,
          tag: release.tag_name,
          osArch: `${slot.os}/${slot.arch}`,
          file: fileName,
          note: purpose,
        });
        return retryOperation(
          () => sha512ForUrl(asset.url),
          3,
          (attempt, error) => {
            workerLog({
              status: Status.Error,
              tag: release.tag_name,
              osArch: `${slot.os}/${slot.arch}`,
              file: fileName,
              note: `${purpose || "download"} attempt ${attempt}: ${
                error?.message || String(error)
              }`,
            });
            workerLog({
              status: Status.Retry,
              tag: release.tag_name,
              osArch: `${slot.os}/${slot.arch}`,
              file: fileName,
              note: `${purpose || "download"} attempt ${attempt}`,
            });
          }
        );
      };

      const sha1 = await downloadOnce("verify");
      let finalSha = sha1;
      let status = Status.Ok;
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
              8
            )}…`; // Keep downloaded consistent SHA.
            finalSha = sha1;
          } else {
            status = Status.Inconsistent;
            note = `sums.txt=${recordedSha.slice(0, 8)}… dl1=${sha1.slice(
              0,
              8
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

      binaries[slot.os][slot.arch] = { sha512: finalSha, urls: [asset.url] };
      workerLog({
        status,
        tag: release.tag_name,
        osArch: `${slot.os}/${slot.arch}`,
        file: fileName,
        sha: finalSha,
        note,
      });
    }

    for (const asset of sortAssets(release.assets)) {
      const slot = slotFor(asset.name);

      if (!slot) {
        workerLog({
          status: Status.SkipUnknown,
          tag: release.tag_name,
          file: fileNameForUrl(asset.url),
        });
        continue;
      }

      // If a binary has already been picked, skip.
      if (binaries[slot.os]?.[slot.arch]) {
        workerLog({
          status: Status.SkipExtra,
          tag: release.tag_name,
          osArch: `${slot.os}/${slot.arch}`,
          file: fileNameForUrl(asset.url),
        });
        continue;
      }

      binaries[slot.os] ??= {};

      const prev = oldBinaries?.[slot.os]?.[slot.arch];
      const urls = [asset.url];

      const fileName = fileNameForUrl(asset.url);

      // Verify even if unchanged to catch bad sums.
      if (prev && sameUrls(prev.urls, urls)) {
        await verifyAndRecord({
          prev,
          recordedSha: sums[asset.name],
          asset,
          slot,
          fileName,
        });
        continue;
      }

      // Need new SHA.
      await verifyAndRecord({
        prev: null,
        recordedSha: sums[asset.name],
        asset,
        slot,
        fileName,
      });
    }

    const outJson = {
      id: release.id,
      name: release.tag_name,
      url: `https://github.com/${owner}/${repo}/releases/tag/${release.tag_name}`,
      binaries: sortObjectDeep(binaries),
    };

    await fs.writeFile(filePath, JSON.stringify(outJson, null, "\t") + "\n");

    workerLog({ status: Status.Saved, tag: release.tag_name });
    parentPort.postMessage({ type: "done", tag: release.tag_name });
  }
  parentPort.close();
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
  let currentIndex = [];

  if (rebuild) {
    console.warn("Rebuilding index from scratch.");
  } else {
    try {
      currentIndex = JSON.parse(await fs.readFile(indexFile, "utf8"));
    } catch {
      // Missing index means full rebuild.
      currentIndex = [];
    }
  }

  const knownIds = new Set(currentIndex.map((entry) => entry.id));
  const knownTagNames = new Set(currentIndex.map((entry) => entry.name));

  // When doing incremental updates, always refetch the last 3 builds to fill in
  // any missing assets that may have been added after initial indexing.
  let refetchIds = new Set();

  if (!rebuild && currentIndex.length > 0) {
    const lastBuilds = currentIndex.slice(0, 3);
    refetchIds = new Set(lastBuilds.map((entry) => entry.id));
    console.log(
      `Will refetch data for the last ${lastBuilds.length} builds: ${lastBuilds
        .map((b) => b.name)
        .join(", ")}`
    );
  }

  // Fetch releases page by page until we hit the first known ID.
  const releases = [];
  const perPage = 100;

  for (let page = 1; page < Infinity; page++) {
    const url = `https://api.github.com/repos/${owner}/${repo}/releases?per_page=${perPage}&page=${page}`;
    console.log(`Fetching page ${page} of releases from GitHub API…`);
    const pageReleases = await fetchJSON(url);

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
            `Found re-release: ${release.tag_name} (new ID: ${release.id})`
          );
        }
      } else {
        console.log(
          `Found known release ID ${release.id} (${release.tag_name}) on page ${page}. Stopping.`
        );
        page = Infinity; // Stop outer loop.
        break;
      }
    }

    if (pageReleases.length < perPage) {
      break; // Last page.
    }
  }

  // Releases are API newest → oldest, with the newest at index 0 in index.json.
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
  const active = new Set();

  const isTTY = process.stdout.isTTY;

  const clearLine = () => {
    if (!isTTY) {
      return;
    }
    process.stdout.clearLine(0);
    process.stdout.cursorTo(0);
  };

  function printProgress() {
    if (!isTTY) {
      return;
    }

    const pct = completed / total;
    const width = 28;
    const filled = Math.round(pct * width);
    const bar = "█".repeat(filled) + "░".repeat(width - filled);
    const tags = [...active].slice(0, 3).join(", ");
    const more = active.size > 3 ? `, …(+${active.size - 3})` : "";

    clearLine();
    process.stdout.write(
      `[${bar}] ${(pct * 100).toFixed(1).padStart(5)}%  ` +
        `${completed}/${total} done  ` +
        `${active.size} active  ` +
        (active.size ? `{ ${tags}${more} }` : "")
    );
  }

  printProgress();

  const pool = [];

  for (let i = 0; i < workers; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, total);

    if (start >= end) {
      break;
    }

    const worker = new Worker(new URL(import.meta.url), {
      type: "module",
      workerData: {
        releases: releases.slice(start, end),
        owner: owner,
        repo: repo,
        outDir: outDir,
      },
    });

    worker.on("message", (msg) => {
      if (msg.type === "start") {
        active.add(msg.tag);
      } else if (msg.type === "done") {
        active.delete(msg.tag);
        completed++;
      } else if (msg.type === "log") {
        clearLine();
        console.log(msg.msg);
      }
      printProgress();
    });

    worker.on("error", (err) => {
      clearLine();
      console.error("Worker error:", err);
    });

    pool.push(new Promise((res) => worker.on("exit", res)));
  }

  await Promise.all(pool);
  clearLine();
  console.log("[████████████████████████████████] 100.0%  finished");

  // Write index.json with the release index.
  const indexMap = new Map();
  const tagNameToIdMap = new Map();

  if (rebuild) {
    // For rebuild, just use the fetched releases.
    releases.forEach((r) => {
      indexMap.set(r.id, { id: r.id, name: r.tag_name });
      tagNameToIdMap.set(r.tag_name, r.id);
    });
  } else {
    // For incremental, start with existing index, then overlay new releases to
    // update refetched ones and add new ones.
    currentIndex.forEach((entry) => {
      indexMap.set(entry.id, entry);
      tagNameToIdMap.set(entry.name, entry.id);
    });
    releases.forEach((r) => {
      if (r.isRerelease) {
        // Remove the old entry with the same tag name.
        const oldId = tagNameToIdMap.get(r.tag_name);
        if (oldId) {
          indexMap.delete(oldId);
          console.log(
            `Removed old index entry for re-release: ${r.tag_name} (old ID: ${oldId})`
          );
        }
      }

      // Add the new entry.
      indexMap.set(r.id, { id: r.id, name: r.tag_name });
      tagNameToIdMap.set(r.tag_name, r.id);
    });
  }

  // Sort by ID descending, newest first.
  const finalIndex = Array.from(indexMap.values()).sort((a, b) => b.id - a.id);

  await fs.writeFile(indexFile, JSON.stringify(finalIndex, null, "\t") + "\n");
  console.log("index.json updated.");
}

// Entry point.

const handlePipeError = (stream) => {
  stream?.on?.("error", (err) => {
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
