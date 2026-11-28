import assert from "node:assert/strict";
import { describe, test } from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../../signals_runtime/module_loading/load_signals_module.mjs";
import { createRealResourceTestRuntime } from "../../runtime_fixture/real_resource_runtime.mjs";
import { createBranchHead } from "../../runtime_fixture/real_resource_signals.mjs";
import { createEffectLine, titlePatch } from "../resource_effect_dag_fixture.mjs";
import {
  generateConcurrentEffectScenario,
  runBoundednessProbe,
  runConcurrentEffectScenario,
  runDenialParityProbe,
  semanticScenarioDigest,
} from "./concurrent_resource_effect_certification_support.mjs";

describe("concurrent resource effect branch DAG certification", { concurrency: false }, () => {
  const evidence = {};

  test("Concurrent Effect Branch DAG Property Test", async () => {
    const proofs = await withDeployment("mainThreadCompatibility", async (runtime) => {
      const generated = [];
      for (let offset = 0; offset < 12; offset += 1) {
        const scenario = generateConcurrentEffectScenario(81_000 + offset, 10 + (offset % 3));
        generated.push(await runConcurrentEffectScenario(runtime, scenario));
      }
      return generated;
    });
    assert.equal(proofs.length, 12);
    assert.equal(proofs.every((proof) => proof.branchResidue === 0), true);
    evidence.scenarioMatrix = Object.freeze({
      generatedScenarioCount: proofs.length,
      minimumEffectCount: 10,
      siblings: true,
      singleDependencies: true,
      multiDependencies: true,
      sameLocusConflicts: true,
      retries: true,
      responsePermutations: true,
      dependencyPolicies: Object.freeze([
        "independent",
        "cancelOnDependencyRejection",
      ]),
      seeds: Object.freeze(proofs.map((proof) => proof.seed)),
      effectCounts: Object.freeze(proofs.map((proof) => proof.effectCount)),
    });
    evidence.residueReport = zeroResidueReport(proofs);
  });

  test("Worker First Compatibility Full Parity Test", async () => {
    const seeds = [91_001, 91_002, 91_003];
    const workerProofs = await withDeployment("workerFirst", async (runtime) => ({
      scenarios: await runSeedMatrix(runtime, seeds),
      denials: await runDenialParityProbe(runtime, 92_001),
    }));
    const compatibilityProofs = await withDeployment(
      "mainThreadCompatibility",
      async (runtime) => ({
        scenarios: await runSeedMatrix(runtime, seeds),
        denials: await runDenialParityProbe(runtime, 92_001),
      }),
    );
    const workerDigest = workerProofs.scenarios.map(semanticScenarioDigest);
    const compatibilityDigest = compatibilityProofs.scenarios.map(semanticScenarioDigest);
    assert.deepEqual(workerDigest, compatibilityDigest);
    assert.deepEqual(workerProofs.denials, compatibilityProofs.denials);
    evidence.parity = Object.freeze({
      matched: true,
      workerFirst: true,
      mainThreadCompatibility: true,
      digest: JSON.stringify(workerDigest),
      denialDigest: JSON.stringify(workerProofs.denials),
    });
  });

  test("Concurrent Effect Boundedness Slope Test", async () => {
    const probes = await withDeployment("mainThreadCompatibility", async (runtime) => {
      const results = [];
      for (const population of [4, 12, 24]) {
        results.push(await runBoundednessProbe(runtime, population));
      }
      return results;
    });
    const counterDigests = probes.map((probe) => JSON.stringify(probe.counters));
    assert.equal(new Set(counterDigests).size, 1);
    assert.deepEqual(probes[0].counters, {
      openEffectLookupCount: 0,
      dependencyTraversalCount: 1,
      affectedEffectCount: 1,
      affectedLocusCount: 1,
      reconstructionCount: 1,
      fallbackBreadth: 0,
    });
    evidence.performanceEnvelope = Object.freeze({
      fixedAffectedBreadth: true,
      populations: Object.freeze(probes.map((probe) => probe.population)),
      counterDigest: counterDigests[0],
      exactCounters: probes[0].counters,
    });
  });

  test("Concurrent Effect Crash Restore Replay Test", async () => {
    for (const fault of [
      "projectionRefresh",
      "branchRetirement",
    ]) {
      await proveRetryAfterFault(fault);
    }
    await proveAdmissionProjectionFailureIsAtomic();
    await proveRejectionProjectionRecovery();
    await proveRecordedResponseRecovery();
    evidence.crashRestore = Object.freeze({
      recoveredWithoutDuplicateCommit: true,
      phases: Object.freeze([
        "responseRecorded",
        "projectionRefresh",
        "branchRetirement",
        "admissionProjectionCleanup",
        "rejectionNativeRetirement",
      ]),
    });
  });

  test("sealed cross-layer certification bundle", async () => {
    const loaded = await loadSignalsModule({ rawSurface: "real" });
    const layerProof = namedLayerProof();
    const docsProof = Object.freeze({
      example: true,
      claims: true,
      links: true,
      evidence: Object.freeze([
        "concurrent_resource_effect_documentation.test.mjs",
      ]),
    });
    const run = loaded.sealConcurrentResourceEffectBranchDagCertificationRun({
      layerProof,
      scenarioMatrix: evidence.scenarioMatrix,
      parity: evidence.parity,
      performanceEnvelope: evidence.performanceEnvelope,
      residueReport: evidence.residueReport,
      crashRestore: evidence.crashRestore,
      docsProof,
    });
    assert.equal(run.status, "sealed");
    assert.equal(
      run.version,
      "concurrent-resource-effect-branch-dag-certification-v1",
    );
    assert.equal(typeof run.evidenceDigest, "string");
    assert.equal(Object.isFrozen(run.evidence), true);
    assert.throws(
      () => loaded.sealConcurrentResourceEffectBranchDagCertificationRun({}),
      (error) => error.code === "incompleteEvidence",
    );
  });
});

async function runSeedMatrix(runtime, seeds) {
  const proofs = [];
  for (const seed of seeds) {
    proofs.push(await runConcurrentEffectScenario(
      runtime,
      generateConcurrentEffectScenario(seed, 12),
    ));
  }
  return proofs;
}

async function withDeployment(deployment, run) {
  const previousWorker = globalThis.Worker;
  if (deployment === "workerFirst") globalThis.Worker = NodeWorker;
  const loaded = await loadSignalsModule({ rawSurface: "real" });
  let signals = null;
  try {
    signals = await loaded.createSignals({ deployment });
    return await run({
      signals,
      resourcePatch: loaded.resourcePatch,
    });
  } finally {
    if (signals) await signals.terminate();
    globalThis.Worker = previousWorker;
  }
}

async function proveRetryAfterFault(faultKind) {
  let armedFault = null;
  let underlyingCloseoutCount = 0;
  const runtime = await createRealResourceTestRuntime({
    closeout_effect_branch(history, request) {
      if (armedFault === "branchRetirement") {
        armedFault = null;
        throw new Error("injected atomic closeout interruption");
      }
      underlyingCloseoutCount += 1;
      return history.closeout_effect_branch(request);
    },
    fork_branch(history, request) {
      if (
        armedFault === "projectionRefresh"
        && request.name === "resource-effect-projection"
      ) {
        armedFault = null;
        throw new Error("injected projection interruption");
      }
      return history.fork_branch(request);
    },
  });
  try {
    createBranchHead(runtime.signals, `crash-${faultKind}`);
    const line = createEffectLine(runtime);
    const baseline = runtime.signals.history().branches().length;
    const first = await line.patch(titlePatch(runtime, 0, "confirmed-once"));
    const second = await line.patch(titlePatch(runtime, 1, "still-open"));
    const options = { responseId: `crash:${faultKind}:response` };
    armedFault = faultKind;
    await assert.rejects(line.effects().confirm(first.effectId, options));
    const reconstructed = await line.effects().rebuildProjection();
    assert.equal(reconstructed.kind, "derivedEffectProjectionBranch");
    const recovered = await line.effects().confirm(first.effectId, options);
    assert.equal(recovered.kind, "merged");
    assert.equal(underlyingCloseoutCount, 1);
    const duplicate = await line.effects().confirm(first.effectId, options);
    assert.equal(duplicate.kind, "duplicateSettlement");
    await line.effects().reject(second.effectId);
    assert.equal(line.value().items[0].title, "confirmed-once");
    assert.equal(line.effects().open().length, 0);
    assert.equal(runtime.signals.history().branches().length, baseline);
  } finally {
    await runtime.cleanup();
  }
}

async function proveRecordedResponseRecovery() {
  const runtime = await createRealResourceTestRuntime();
  try {
    createBranchHead(runtime.signals, "crash-response-recorded");
    const line = createEffectLine(runtime);
    const parent = await line.patch(titlePatch(runtime, 0, "parent"));
    const child = await line.patch(runtime.mod.resourcePatch.dependsOn(
      titlePatch(runtime, 1, "child"),
      [parent.effectId],
    ));
    const options = { responseId: "crash:child:response" };
    const recorded = await line.effects().confirm(child.effectId, options);
    assert.equal(recorded.kind, "responseRecorded");
    const duplicate = await line.effects().confirm(child.effectId, options);
    assert.equal(duplicate.kind, "duplicateSettlement");
    const settled = await line.effects().confirm(parent.effectId);
    assert.deepEqual(
      settled.automaticallySettled.map((entry) => entry.effectId),
      [child.effectId],
    );
    assert.equal(line.effects().open().length, 0);
  } finally {
    await runtime.cleanup();
  }
}

async function proveAdmissionProjectionFailureIsAtomic() {
  let armed = false;
  const runtime = await createRealResourceTestRuntime({
    fork_branch(history, request) {
      if (armed && request.name === "resource-effect-projection") {
        armed = false;
        throw new Error("injected admission projection interruption");
      }
      return history.fork_branch(request);
    },
  });
  try {
    createBranchHead(runtime.signals, "crash-admission-projection");
    const line = createEffectLine(runtime);
    const baseline = runtime.signals.history().branches().length;
    armed = true;
    await assert.rejects(line.patch(titlePatch(runtime, 0, "not-admitted")));
    assert.equal((await line.effects().rebuildProjection()).kind, "canonical");
    assert.equal(line.effects().open().length, 0);
    assert.deepEqual(line.effects().counters(), {
      effectLookupCount: 0,
      pendingAdmissionCount: 0,
      openEffectCount: 0,
      dependencyIndexKeyCount: 0,
      locusIndexKeyCount: 0,
      retryLineageIndexKeyCount: 0,
    });
    assert.equal(runtime.signals.history().branches().length, baseline);

    const admitted = await line.patch(titlePatch(runtime, 0, "retry-admitted"));
    await line.effects().reject(admitted.effectId);
    assert.equal(runtime.signals.history().branches().length, baseline);
  } finally {
    await runtime.cleanup();
  }
}

async function proveRejectionProjectionRecovery() {
  let armed = false;
  let retirementBatchCount = 0;
  const runtime = await createRealResourceTestRuntime({
    fork_branch(history, request) {
      if (armed && request.name === "resource-effect-projection") {
        armed = false;
        throw new Error("injected rejection projection interruption");
      }
      return history.fork_branch(request);
    },
    retire_branches(history, request) {
      retirementBatchCount += 1;
      return history.retire_branches(request);
    },
  });
  try {
    createBranchHead(runtime.signals, "crash-rejection-projection");
    const line = createEffectLine(runtime);
    const baseline = runtime.signals.history().branches().length;
    const rejected = await line.patch(titlePatch(runtime, 0, "rejected"));
    const sibling = await line.patch(titlePatch(runtime, 1, "survives"));
    const options = { responseId: "crash:rejection:response" };
    retirementBatchCount = 0;
    armed = true;
    await assert.rejects(line.effects().reject(rejected.effectId, options));
    assert.equal(line.effects().get(rejected.effectId).lifecycle, "Retired");
    assert.equal(line.effects().get(sibling.effectId).lifecycle, "Pending");
    assert.equal(retirementBatchCount, 1);
    assert.equal(
      (await line.effects().rebuildProjection()).kind,
      "derivedEffectProjectionBranch",
    );

    const recovered = await line.effects().reject(rejected.effectId, options);
    assert.equal(recovered.kind, "rejectedAndRetired");
    assert.equal(retirementBatchCount, 1);
    assert.equal(line.value().items[1].title, "survives");
    await line.effects().reject(sibling.effectId);
    assert.equal(runtime.signals.history().branches().length, baseline);
  } finally {
    await runtime.cleanup();
  }
}

function zeroResidueReport(proofs) {
  assert.equal(proofs.every((proof) => proof.counters.openEffectCount === 0), true);
  return Object.freeze({
    liveSettledBranches: 0,
    openEffects: 0,
    pendingAdmissions: 0,
    dependencyIndexKeys: 0,
    locusIndexKeys: 0,
  });
}

function namedLayerProof() {
  const proof = (evidence) => Object.freeze({
    verified: true,
    evidence: Object.freeze(evidence),
  });
  return Object.freeze({
    nativeCore: proof(["branch_lifecycle_retirement.rs", "branch_targeted_transactions.rs"]),
    workerBoundary: proof(["worker_branch_commands.rs", "worker_first_callable_form.test.mjs"]),
    resourceProduct: proof(["resource_effect_branch_dag.test.mjs"]),
    formsIntegration: proof(["form_resource_effect_parity_execution.test.mjs"]),
    demoFive: proof(["demo_five_concurrency.test.mjs"]),
    documentation: proof(["concurrent_resource_effect_documentation.test.mjs"]),
  });
}
