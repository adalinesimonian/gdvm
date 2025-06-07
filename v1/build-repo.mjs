#!/usr/bin/env node

/*
  build-repo.js - Builds a JSON-based repository index for Godot releases
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

// Worker thread code.

async function runWorker() {
  const { releases, owner, repo, outDir } = workerData;

  for (const release of releases) {
    parentPort.postMessage({ type: "start", tag: release.tag_name });

    const tagNameCol = release.tag_name.padEnd(21);

    const safeName = release.tag_name.replace(/[^\w.-]+/g, "_");
    const filePath = path.join(outDir, `${release.id}_${safeName}.json`);

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
      /sha512.*\.txt$/i.test(name)
    );

    if (sumsFile) {
      parentPort.postMessage({
        type: "log",
        msg: `[download]    ${tagNameCol}  Downloading SHA512-SUMS file…`,
      });

      const sumsData = await retryOperation(
        () => fetchText(byName[sumsFile]),
        3,
        (attempt, error) => {
          parentPort.postMessage({
            type: "log",
            msg: `[error]       ${tagNameCol}  Downloading SHA512-SUMS file failed on attempt ${attempt}.\nError: ${
              error?.message || String(error)
            }`,
          });
          parentPort.postMessage({
            type: "log",
            msg: `[dl try ${attempt}]    ${tagNameCol}  Downloading SHA512-SUMS file…`,
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

    for (const asset of sortAssets(release.assets)) {
      const slot = slotFor(asset.name);

      if (!slot) {
        parentPort.postMessage({
          type: "log",
          msg:
            `[skip-unkwn]  ${release.tag_name.padEnd(21)}  ` +
            `${" ".repeat(24)}  ` +
            `${fileNameForUrl(asset.url)}`,
        });
        continue;
      }

      // If a binary has already been picked, skip.
      if (binaries[slot.os]?.[slot.arch]) {
        parentPort.postMessage({
          type: "log",
          msg:
            `[skip-extra]  ${release.tag_name.padEnd(21)}  ` +
            `${`${slot.os}/${slot.arch}`.padEnd(24)}  ` +
            `${fileNameForUrl(asset.url)}`,
        });

        continue;
      }

      binaries[slot.os] ??= {};

      const prev = oldBinaries?.[slot.os]?.[slot.arch];
      const urls = [asset.url];

      const osArchCol = `${slot.os}/${slot.arch}`.padEnd(24);
      const fileNameCol = fileNameForUrl(asset.url).padEnd(50);

      // Copy and skip if unchanged.
      if (prev && sameUrls(prev.urls, urls)) {
        binaries[slot.os][slot.arch] = prev;
        parentPort.postMessage({
          type: "log",
          msg: `[unchanged]   ${tagNameCol}  ${osArchCol}  ${fileNameCol}  ${prev.sha512.slice(
            0,
            8
          )}…`,
        });

        continue;
      }

      // Need new SHA.
      let sha;
      if (sums[asset.name]) {
        sha = sums[asset.name];

        parentPort.postMessage({
          type: "log",
          msg: `[cached-sha]   ${tagNameCol}  ${osArchCol}  ${fileNameCol}  ${sha.slice(
            0,
            8
          )}…`,
        });
      } else {
        parentPort.postMessage({
          type: "log",
          msg: `[download]    ${tagNameCol}  ${osArchCol}  ${fileNameCol}`,
        });

        sha = await retryOperation(
          () => sha512ForUrl(asset.url),
          3,
          (attempt, error) => {
            parentPort.postMessage({
              type: "log",
              msg: `[error]       ${tagNameCol}  ${osArchCol}  ${fileNameCol}\nError: ${
                error?.message || String(error)
              }`,
            });
            parentPort.postMessage({
              type: "log",
              msg: `[dl try ${attempt}]    ${tagNameCol}  ${osArchCol}  ${fileNameCol}`,
            });
          }
        );

        parentPort.postMessage({
          type: "log",
          msg: `[sha512]      ${tagNameCol}  ${osArchCol}  ${fileNameCol}  ${sha.slice(
            0,
            8
          )}…`,
        });
      }

      binaries[slot.os][slot.arch] = { sha512: sha, urls };
    }

    const outJson = {
      id: release.id,
      name: release.tag_name,
      url: `https://github.com/${owner}/${repo}/releases/tag/${release.tag_name}`,
      binaries: sortObjectDeep(binaries),
    };

    await fs.writeFile(filePath, JSON.stringify(outJson, null, "\t") + "\n");

    parentPort.postMessage({
      type: "log",
      msg: `[saved]       ${release.tag_name}`,
    });
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
      if (!rebuild && knownIds.has(release.id)) {
        console.log(
          `Found known release ID ${release.id} (${release.tag_name}) on page ${page}. Stopping.`
        );
        page = Infinity; // Stop outer loop.
        break;
      }
      releases.push({
        id: release.id,
        tag_name: release.tag_name,
        assets: release.assets.map((a) => ({
          name: a.name,
          url: a.browser_download_url,
        })),
      });
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

  const newEntries = releases.map((r) => ({ id: r.id, name: r.tag_name }));
  const finalIndex = rebuild
    ? newEntries
    : [...newEntries, ...currentIndex].sort((a, b) => b.id - a.id);

  await fs.writeFile(indexFile, JSON.stringify(finalIndex, null, "\t") + "\n");
  console.log("index.json updated.");
}

// Entry point.

if (isMainThread) {
  runParent().catch((err) => console.error(err));
} else {
  runWorker();
}
