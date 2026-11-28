import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  DEMO_FIVE_CODE,
  DEMO_FOUR_CODE,
  DEMO_ONE_CODE,
  DEMO_THREE_CODE,
  DEMO_TWO_CODE,
} from "../src/state/demoCodeSamples.ts";
import { demoRegistry } from "../src/state/demoData.ts";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const appDir = path.resolve(testDir, "..");
const manifestPath = path.resolve(
  appDir,
  "../../crates/worth-signal-wasm/docs/metadata/public-documentation.json",
);
const expectedGuides = new Map([
  [1, "core/diagnostics"],
  [2, "resources/debugging/README"],
  [3, "forms/collaboration/README"],
  [4, "router/admission/admit"],
  [5, "resources/effects/README"],
  [6, "local-truth/branches-and-snapshots"],
]);
const sharedSamples = new Map([
  [1, DEMO_ONE_CODE],
  [2, DEMO_TWO_CODE],
  [3, DEMO_THREE_CODE],
  [4, DEMO_FOUR_CODE],
  [5, DEMO_FIVE_CODE],
]);
const mojibake = /\u00c2|\u00c3|\u00e2(?:\u0080|\u20ac|\u201a|\u2020|\u02c6)/u;

test("each demo teaches through one canonical public guide", async () => {
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const publicPaths = new Set(
    manifest.sections.flatMap((section) => section.items.map((item) => item.path)),
  );

  assert.equal(demoRegistry.length, 6);
  for (const demo of demoRegistry) {
    assert.equal(demo.relatedDocsPath, expectedGuides.get(demo.id));
    assert.equal(publicPaths.has(demo.relatedDocsPath), true, `demo ${demo.id} guide is not public`);
    assert.doesNotMatch(JSON.stringify(demo), mojibake, `demo ${demo.id} contains mojibake`);
  }
});

test("displayed code and comparison code share one registry-owned source", () => {
  for (const [demoId, code] of sharedSamples) {
    const demo = demoRegistry.find((candidate) => candidate.id === demoId);
    assert.ok(demo, `missing demo ${demoId}`);
    assert.equal(demo.WORTHCode, code);
  }
  assert.match(DEMO_ONE_CODE, /await signals\.transaction/u);
  assert.match(DEMO_ONE_CODE, /await signals\.diagnostics\(\)\.why/u);
  assert.match(DEMO_FIVE_CODE, /if \(!\("effectId" in admission\)\)/u);
});

test("demo one exposes only the active diagnostics lesson", async () => {
  const source = await readFile(path.join(appDir, "src/ui/SignalsSection.tsx"), "utf8");
  const demo = demoRegistry.find((candidate) => candidate.id === 1);
  assert.ok(demo);
  assert.doesNotMatch(source, /SignalsAspectWorkbench|createSignalsAspectGraph/u);
  assert.match(source, /SignalsTransferWorkbench/u);
  assert.doesNotMatch(`${demo.title}\n${demo.purpose}\n${demo.preface}`, /part 1|part 2|aspect/iu);
});
