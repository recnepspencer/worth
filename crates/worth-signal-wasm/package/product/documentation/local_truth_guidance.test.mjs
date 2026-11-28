import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { loadSignalsModule } from "../signals_runtime/module_loading/load_signals_module.mjs";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../../..",
);
const docsRoot = "crates/worth-signal-wasm/docs/local-truth";

test("Local Truth docs preserve the process-local and platform authority boundary", async () => {
  const [landing, boundaries, history] = await Promise.all([
    readDoc("README.md"),
    readDoc("authority-boundaries.md"),
    readDoc("history-and-rebuild.md"),
  ]);

  assert.match(landing, /browser-local application-value authority/u);
  assert.match(landing, /Worth Query over Worth Relational/u);
  assert.match(boundaries, /Query -> Relational -> Bridge -> Signal/u);
  assert.match(boundaries, /TypeScript Local Truth -> Signal/u);
  assert.match(boundaries, /does not provide MVCC, persistence, replication/u);
  assert.match(boundaries, /Do not call process-local history durable or restart-stable/u);
  assert.match(history, /There is no public export-and-restore API/u);
  assert.match(history, /derived Signal state/u);
});

test("Local Truth docs cover the executable lifecycle for branching and historical readback", async () => {
  const [start, branches, history, reference] = await Promise.all([
    readDoc("getting-started.md"),
    readDoc("branches-and-snapshots.md"),
    readDoc("history-and-rebuild.md"),
    readDoc("api-reference.md"),
  ]);

  assert.match(start, /localTruthSchema/u);
  assert.match(start, /signals\.localTruth/u);
  assert.match(branches, /forkBranch/u);
  assert.match(branches, /historicalSnapshot/u);
  assert.match(history, /rebuildDerivation/u);
  assert.match(reference, /LocalTruthOutcome/u);
  assert.match(reference, /There is currently no public delete operation/u);
});

test("documented public Local Truth branch edits stay independent", async () => {
  const { createSignals, importProductModule, cleanup } = await loadSignalsModule({
    rawSurface: "real",
  });
  const { localTruthSchema } = await importProductModule("local_truth/facade.js");
  const signals = await createSignals({ deployment: "mainThreadCompatibility" });
  const initial = { teeth: 18, thickness: 0.58 };
  const input = signals.input(initial, { producesAspects: [0, 1] });
  const schema = localTruthSchema({
    id: "documentation.gear",
    aspects: [
      { id: "teeth", field: "teeth", valueType: "number",
        equivalence: { kind: "exact" }, costClass: "constant" },
      { id: "thickness", field: "thickness", valueType: "number",
        equivalence: { kind: "numberEpsilon", epsilon: 0.001 },
        costClass: "constant" },
    ],
  });
  const truth = signals.localTruth({
    authorityId: "documentation-gear",
    schema,
    initialEntities: { gear: initial },
    bindings: [{
      entityId: "gear",
      input,
      aspectMap: { teeth: 0, thickness: 1 },
    }],
  });

  try {
    await truth.ready?.();
    const main = admitted(await truth.branch());
    const source = admitted(await truth.forkBranch({
      parentBranchId: main.id,
      expectedParentBasis: main.basis,
      name: "source",
    }));
    const target = admitted(await truth.forkBranch({
      parentBranchId: main.id,
      expectedParentBasis: main.basis,
      name: "target",
    }));

    admitted(await truth.commit({
      requestId: "documentation-source-teeth",
      branchId: source.id,
      expectedBasis: source.basis,
      operations: [{ entityId: "gear", aspectId: "teeth", value: 20 }],
    }));
    admitted(await truth.commit({
      requestId: "documentation-target-thickness",
      branchId: target.id,
      expectedBasis: target.basis,
      operations: [{ entityId: "gear", aspectId: "thickness", value: 0.62 }],
    }));

    assert.deepEqual((await truth.inspect()).values[target.id].gear, {
      teeth: 18,
      thickness: 0.62,
    });
    assert.equal((await truth.inspect()).values[source.id].gear.teeth, 20);
  } finally {
    await truth.terminate();
    signals.free();
    await cleanup();
  }
});

function admitted(outcome) {
  assert.ok(
    outcome.posture === "success" || outcome.posture === "advisory",
    outcome.message ?? `Expected admitted outcome, received ${outcome.posture}`,
  );
  return outcome.value;
}

function readDoc(name) {
  return readFile(path.join(repoRoot, docsRoot, name), "utf8");
}
