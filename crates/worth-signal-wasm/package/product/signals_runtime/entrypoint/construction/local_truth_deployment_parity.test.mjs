import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

const initialGear = Object.freeze({ teeth: 16, material: "steel", rotation: 0, label: "Drive" });
const aspectIds = Object.freeze(["label", "material", "rotation", "teeth"]);
const signalAspects = Object.freeze([0, 1, 2, 3]);

async function runScenario(deployment) {
  const loaded = await loadSignalsModule({ rawSurface: "real" });
  const { declareLocalTruthSchema } = await loaded.importProductModule(
    "local_truth/schema/schema_declaration.js",
  );
  const signals = await loaded.createSignals({ deployment });
  const schema = declareLocalTruthSchema({
    id: "deployment-parity.gear",
    aspects: aspectIds.map((field) => ({
      id: field,
      field,
      valueType: field === "teeth" || field === "rotation" ? "number" : "string",
      equivalence: { kind: "exact" },
      costClass: "constant",
    })),
  });
  const gear = signals.input(initialGear, {
    debugName: "gear",
    producesAspects: signalAspects,
  });
  const truth = signals.localTruth({
    authorityId: "deployment-parity",
    schema,
    initialEntities: { gear: initialGear },
    bindings: [{
      entityId: "gear",
      input: gear,
      aspectMap: { label: 0, material: 1, rotation: 2, teeth: 3 },
    }],
  });
  await truth.ready?.();
  const main = (await truth.branch()).value;
  const source = (await truth.forkBranch({
    parentBranchId: main.id,
    expectedParentBasis: main.basis,
    name: "source",
  })).value;
  const target = (await truth.forkBranch({
    parentBranchId: main.id,
    expectedParentBasis: main.basis,
    name: "target",
  })).value;
  const sourceCommit = await truth.commit({
    requestId: "source-teeth",
    branchId: source.id,
    expectedBasis: source.basis,
    operations: [{ entityId: "gear", aspectId: "teeth", value: 24 }],
  });
  const targetCommit = await truth.commit({
    requestId: "target-label",
    branchId: target.id,
    expectedBasis: target.basis,
    operations: [{ entityId: "gear", aspectId: "label", value: "Final drive" }],
  });
  const inspection = await truth.inspect();
  const destroyed = await truth.destroyDerivation(target.id);
  const afterDestroy = await truth.inspect();
  const rebuilt = await truth.rebuildDerivation(target.id);
  const afterRebuild = await truth.inspect();
  if (deployment === "workerFirst") {
    assert.ok(inspection.bridgeCounters.roundTrips >= 8);
    assert.ok(inspection.bridgeCounters.serializedBreadth > 0);
  } else {
    assert.equal(inspection.bridgeCounters, undefined);
  }
  await truth.terminate();
  await signals.terminate();
  return {
    sourceCommitId: sourceCommit.value.commit.id,
    sourceForkDerivation: source.derivation.posture,
    sourceForkDigest: source.derivation.digest,
    targetForkDerivation: target.derivation.posture,
    targetForkDigest: target.derivation.digest,
    targetCommitId: targetCommit.value.commit.id,
    sourceDerivation: sourceCommit.value.derivation.posture,
    sourceProjectionDigest: sourceCommit.value.derivation.digest,
    sourceDerivationReason: sourceCommit.value.derivation.reason ?? null,
    targetDerivation: targetCommit.value.derivation.posture,
    targetProjectionDigest: targetCommit.value.derivation.digest,
    targetDerivationReason: targetCommit.value.derivation.reason ?? null,
    destroyedPosture: destroyed.posture,
    rebuiltPosture: rebuilt.posture,
    truthSurvivedDestroy: afterDestroy.digest === inspection.digest,
    truthSurvivedRebuild: afterRebuild.digest === inspection.digest,
    truthDigest: inspection.digest,
    targetValue: inspection.values[target.id].gear,
    sourceValue: inspection.values[source.id].gear,
    counters: inspection.counters,
  };
}

test("worker-first and compatibility local truth converge on the same semantic history", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  try {
    const compatibility = await runScenario("mainThreadCompatibility");
    const workerFirst = await runScenario("workerFirst");
    assert.deepEqual(workerFirst, compatibility);
    assert.deepEqual(workerFirst.targetValue, {
      teeth: 16,
      material: "steel",
      rotation: 0,
      label: "Final drive",
    });
    assert.equal(workerFirst.sourceValue.teeth, 24);
    assert.equal(workerFirst.destroyedPosture, "RebuildRequired");
    assert.equal(workerFirst.rebuiltPosture, "Current");
    assert.equal(workerFirst.truthSurvivedDestroy, true);
    assert.equal(workerFirst.truthSurvivedRebuild, true);
  } finally {
    globalThis.Worker = previousWorker;
  }
});
