import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { localTruthCertificationManifest } from "./local_truth_certification_manifest.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../../../..");

test("certification manifest closes every required evidence lane", () => {
  assert.deepEqual(Object.keys(localTruthCertificationManifest), [
    "authority",
    "derivation",
    "deployment",
    "boundedness",
    "recovery",
    "uiNoShortcut",
  ]);
  assert.deepEqual(localTruthCertificationManifest.boundedness, {
    branches: 32,
    entities: 128,
    aspects: 64,
  });
});

test("docs state both one-way authority lanes and process-local limits", async () => {
  const boundaries = await read("crates/worth-signal-wasm/docs/local-truth/authority-boundaries.md");
  assert.match(boundaries, /Query -> Relational -> Bridge -> Signal/u);
  assert.match(boundaries, /TypeScript Local Truth -> Signal/u);
  assert.match(boundaries, /does not provide MVCC, persistence, replication/u);
  assert.match(boundaries, /Do not call process-local history durable or restart-stable/u);
});

test("Rust Signal and the live gear UI contain no local-truth or object-merge shortcut", async () => {
  const rust = await readTree("crates/worth-signal-wasm/src", ".rs");
  assert.doesNotMatch(rust, /LocalTruthAuthority|LocalTruthCommit|local_truth_journal/u);
  const liveGear = [
    await read("apps/WORTH-signal-demo/src/local-truth-gear/gear_scenario.ts"),
    await read("apps/WORTH-signal-demo/src/ui/CompositionSection.tsx"),
  ].join("\n");
  assert.doesNotMatch(liveGear, /composeManualMergedState|manualMergedObject/u);
  assert.match(liveGear, /forkBranch/u);
  assert.match(liveGear, /inspection\.values/u);
});

async function read(relativePath) {
  return readFile(path.join(repoRoot, relativePath), "utf8");
}

async function readTree(relativePath, extension) {
  const root = path.join(repoRoot, relativePath);
  const chunks = [];
  await visit(root);
  return chunks.join("\n");

  async function visit(directory) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) await visit(entryPath);
      else if (entry.name.endsWith(extension)) chunks.push(await readFile(entryPath, "utf8"));
    }
  }
}
