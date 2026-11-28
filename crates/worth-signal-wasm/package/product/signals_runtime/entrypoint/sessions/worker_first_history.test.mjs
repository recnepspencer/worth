import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

function comparableSnapshotEnvelope(envelope) {
  return {
    state: envelope.state,
  };
}

function comparableSnapshotArtifact(snapshot) {
  return {
    state: snapshot.state,
  };
}

function comparableRuntimeBranch(branch) {
  return {
    id: branch.id,
    name: branch.name,
    parent_branch_id: branch.parent_branch_id,
  };
}

function comparableBranchStateProof(proof) {
  return {
    proofSchemaVersion: proof.proofSchemaVersion,
    branchId: proof.branchId,
    branchName: proof.branchName,
    stateDigest: proof.stateDigest,
  };
}

function comparableReplayParityProof(proof) {
  return {
    proofSchemaVersion: proof.proofSchemaVersion,
    expectedBranchId: proof.expectedBranchId,
    expectedBranchName: proof.expectedBranchName,
    expectedStateDigest: proof.expectedStateDigest,
    replayedBranchId: proof.replayedBranchId,
    replayedBranchName: proof.replayedBranchName,
    replayedStateDigest: proof.replayedStateDigest,
    parity: proof.parity,
    mismatchClasses: proof.mismatchClasses,
  };
}

function comparableMergePlan(plan) {
  return {
    source_branch_id: plan.source_branch_id,
    target_branch_id: plan.target_branch_id,
    merge_kind: plan.merge_kind,
    selected_semantics: plan.selected_semantics,
    counters: plan.counters,
  };
}

function comparableMergePlanProof(envelope) {
  return {
    plan: comparableMergePlan(envelope.plan),
    proof: {
      proofSchemaVersion: envelope.proof.proofSchemaVersion,
      registryBundleDigest: envelope.proof.registryBundleDigest,
      semanticsDigest: envelope.proof.semanticsDigest,
      selectedStrategyDigest: envelope.proof.selectedStrategyDigest,
      selectedMergeBaseDigest: envelope.proof.selectedMergeBaseDigest,
      selectedConflictPolicyDigest: envelope.proof.selectedConflictPolicyDigest,
      selectedConflictIsolationDigest: envelope.proof.selectedConflictIsolationDigest,
      selectedIdentityMatcherDigest: envelope.proof.selectedIdentityMatcherDigest,
      selectedSourceOnlyPolicyDigest: envelope.proof.selectedSourceOnlyPolicyDigest,
      selectedDeletionPolicyDigest: envelope.proof.selectedDeletionPolicyDigest,
    },
  };
}

function comparableMergeResult(result) {
  return {
    source_branch: result.source_branch,
    target_branch: result.target_branch,
    merge_kind: result.merge_kind,
    selected_semantics: result.selected_semantics,
    counters: result.counters,
  };
}

function comparableMergeResultProof(envelope) {
  return {
    result: {
      source_branch: envelope.result.source_branch,
      target_branch: envelope.result.target_branch,
      selected_semantics: envelope.result.selected_semantics,
      counters: envelope.result.counters,
    },
    proof: {
      proofSchemaVersion: envelope.proof.proofSchemaVersion,
      registryBundleDigest: envelope.proof.registryBundleDigest,
      semanticsDigest: envelope.proof.semanticsDigest,
      selectedStrategyDigest: envelope.proof.selectedStrategyDigest,
      selectedMergeBaseDigest: envelope.proof.selectedMergeBaseDigest,
      selectedConflictPolicyDigest: envelope.proof.selectedConflictPolicyDigest,
      selectedConflictIsolationDigest: envelope.proof.selectedConflictIsolationDigest,
      selectedIdentityMatcherDigest: envelope.proof.selectedIdentityMatcherDigest,
      selectedSourceOnlyPolicyDigest: envelope.proof.selectedSourceOnlyPolicyDigest,
      selectedDeletionPolicyDigest: envelope.proof.selectedDeletionPolicyDigest,
    },
  };
}

function comparableBranchReplay(summary) {
  return {
    frames: summary.frames.filter(
      (frame) => frame.kind !== "SnapshotCaptured" || frame.snapshotId === 0,
    ),
  };
}

test("worker-first history facade preserves branch lifecycle and exact snapshot restore parity on worker-owned truth", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const mod = await loadSignalsModule({ rawSurface: "real" });
  const { createSignals, importProductModule, cleanup } = mod;
  const { createWorkerRuntimeBridge } = await importProductModule(
    "entrypoint/bridge/worker_runtime_bridge.js",
  );
  const { createWorkerFirstHistoryFacade } = await importProductModule(
    "entrypoint/worker_first_history.js",
  );

  const compatibilitySignals = await createSignals({
    deployment: "mainThreadCompatibility",
  });
  const count = compatibilitySignals.input(2, { debugName: "count" });
  const graph = compatibilitySignals.graph("workerFirstHistory", {
    inputs: { count },
    outputs: {
      doubled: compatibilitySignals.computedSpec("worker:first:history:doubled", {
        reads: [count.id],
        expr: {
          kind: "sum",
          args: [
            { kind: "read", id: count.id },
            { kind: "read", id: count.id },
          ],
        },
        identity: { kind: "exact" },
      }),
    },
  });
  graph.writeInput("count", 5);
  const outputId = graph.output("doubled").id;
  const bridge = createWorkerRuntimeBridge();

  try {
    await bridge.bootstrapRecord();
    await bridge.workerRuntimeShellLock();
    await bridge.publishPortableGraph({
      ...compatibilitySignals.adapters().exportDefinitions(),
      outputIds: [outputId],
    });
    await bridge.applyTransaction([
      {
        kind: "set",
        id: count.id,
        value: 5,
      },
    ]);
    const history = createWorkerFirstHistoryFacade({ bridge });
    const baselineSnapshot = await history.snapshot();
    const compatibilityBaselineSnapshot = compatibilitySignals.history().snapshot();

    assert.deepEqual(
      await history.replay_for(outputId),
      compatibilitySignals.history().replay_for(outputId),
    );
    assert.deepEqual(
      await history.lineage_for(outputId),
      compatibilitySignals.history().lineage_for(outputId),
    );
    assert.deepEqual(
      await history.recentHistory(),
      compatibilitySignals.diagnostics().recentHistory(),
    );
    assert.deepEqual(
      comparableSnapshotEnvelope(await history.snapshot()),
      comparableSnapshotEnvelope(compatibilitySignals.history().snapshot()),
    );
    assert.deepEqual(
      comparableRuntimeBranch(await history.current_branch()),
      comparableRuntimeBranch(compatibilitySignals.history().current_branch()),
    );
    assert.deepEqual(
      (await history.branches()).map(comparableRuntimeBranch),
      compatibilitySignals.history().branches().map(comparableRuntimeBranch),
    );
    assert.deepEqual(
      comparableBranchReplay(await history.replay_for_branch(0)),
      comparableBranchReplay(compatibilitySignals.history().replay_for_branch(0)),
    );
    const workerBranchEnvelope = await history.branch_snapshot_envelope(0);
    const workerBranchSnapshot = await history.branch_snapshot(0);
    assert.equal(typeof await history.branch_snapshot_id(0), "number");
    assert.equal(
      workerBranchSnapshot.snapshotRestoreToken.length > 0,
      true,
    );
    assert.deepEqual(
      comparableSnapshotEnvelope(workerBranchEnvelope),
      comparableSnapshotEnvelope(compatibilitySignals.history().branch_snapshot_envelope(0)),
    );
    assert.deepEqual(
      comparableSnapshotArtifact(workerBranchSnapshot),
      comparableSnapshotArtifact(compatibilitySignals.history().branch_snapshot(0)),
    );
    assert.deepEqual(
      comparableBranchStateProof(await history.branch_state_proof(0)),
      comparableBranchStateProof(compatibilitySignals.history().branch_state_proof(0)),
    );
    assert.deepEqual(
      comparableReplayParityProof(await history.replay_parity_proof(0, 0)),
      comparableReplayParityProof(compatibilitySignals.history().replay_parity_proof(0, 0)),
    );
    const replayArtifactInput = {
      proofSchemaVersion: compatibilitySignals.adapters().runtimeProofReport().proofSchemaVersion,
      registryBundleDigest: compatibilitySignals.adapters().runtimeProofReport().registryBundleDigest,
      loweredStrategyBundleDigest: null,
      mergePlanDigest: null,
      mergeResultDigest: null,
      lineageDigest: null,
      branchStateDigest: compatibilitySignals.history().branch_state_proof(0).stateDigest,
    };
    assert.deepEqual(
      await history.replay_artifact_proof(replayArtifactInput, 0),
      compatibilitySignals.history().replay_artifact_proof(replayArtifactInput, 0),
    );

    const workerFeature = await history.create_branch("feature");
    const compatibilityFeature = compatibilitySignals.history().create_branch("feature");
    assert.deepEqual(
      comparableRuntimeBranch(workerFeature),
      comparableRuntimeBranch(compatibilityFeature),
    );
    await history.switch_branch(workerFeature.id);
    compatibilitySignals.history().switch_branch(compatibilityFeature.id);
    await bridge.applyTransaction([{ kind: "set", id: count.id, value: 11 }]);
    count.set(11);
    assert.deepEqual(
      comparableRuntimeBranch(await history.current_branch()),
      comparableRuntimeBranch(compatibilitySignals.history().current_branch()),
    );
    assert.equal(
      (await bridge.readSignals({ signalIds: [outputId] })).signals[0]?.value,
      compatibilitySignals.read(outputId),
    );

    const workerFeatureSnapshot = await history.branch_snapshot(workerFeature.id);
    const compatibilityFeatureSnapshot = compatibilitySignals.history().branch_snapshot(
      compatibilityFeature.id,
    );
    assert.deepEqual(
      comparableSnapshotArtifact(workerFeatureSnapshot),
      comparableSnapshotArtifact(compatibilityFeatureSnapshot),
    );
    await bridge.applyTransaction([{ kind: "set", id: count.id, value: 13 }]);
    count.set(13);
    const workerFeatureSnapshotUpdated = await history.branch_snapshot(workerFeature.id);
    const compatibilityFeatureSnapshotUpdated = compatibilitySignals.history().branch_snapshot(
      compatibilityFeature.id,
    );
    assert.notEqual(
      workerFeatureSnapshotUpdated.snapshotRestoreToken,
      workerFeatureSnapshot.snapshotRestoreToken,
    );
    assert.deepEqual(
      comparableSnapshotArtifact(workerFeatureSnapshotUpdated),
      comparableSnapshotArtifact(compatibilityFeatureSnapshotUpdated),
    );
    await history.restore_exact_snapshot(baselineSnapshot);
    compatibilitySignals.history().restore_exact_snapshot(compatibilityBaselineSnapshot);
    assert.equal(
      (await bridge.readSignals({ signalIds: [outputId] })).signals[0]?.value,
      compatibilitySignals.read(outputId),
    );

    await history.switch_branch(workerFeature.id);
    compatibilitySignals.history().switch_branch(compatibilityFeature.id);
    await history.restore_exact_branch_snapshot(workerFeature.id, workerFeatureSnapshot);
    compatibilitySignals.history().restore_exact_branch_snapshot(
      compatibilityFeature.id,
      compatibilityFeatureSnapshot,
    );
    assert.equal(
      (await bridge.readSignals({ signalIds: [outputId] })).signals[0]?.value,
      compatibilitySignals.read(outputId),
    );
    await history.restore_branch_snapshot_by_id(
      workerFeature.id,
      await history.branch_snapshot_id(workerFeature.id),
    );
    compatibilitySignals.history().restore_branch_snapshot_by_id(
      compatibilityFeature.id,
      compatibilitySignals.history().branch_snapshot_id(compatibilityFeature.id),
    );
    assert.equal(
      (await bridge.readSignals({ signalIds: [outputId] })).signals[0]?.value,
      compatibilitySignals.read(outputId),
    );
  } finally {
    await bridge.terminate();
    compatibilitySignals.free();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});
