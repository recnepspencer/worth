import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";
import {
  comparableBranchStateProof,
  comparableGraphSummary,
  comparableHistorySurfaceSummary,
  comparableMergePlan,
  comparableMergeResult,
  comparableMergeResultProof,
  comparablePerformanceSummary,
  comparableRecentHistory,
  comparableReplayParityProof,
  comparableRuntimeBranch,
  comparableSnapshotArtifact,
  comparableSnapshotEnvelope,
} from "./support/worker_first_root_surface_comparators.mjs";

test("default worker-first root exposes synchronous cached diagnostics, history, and adapters for the active imported graph", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });

  const compatibilitySignals = await createSignals({ deployment: "mainThreadCompatibility" });
  const count = compatibilitySignals.input(4, { debugName: "count" });
  const graph = compatibilitySignals.graph("workerFirstRootSurfaces", {
    inputs: { count },
    outputs: {
      doubled: compatibilitySignals.computedSpec("worker:first:root:surfaces:doubled", {
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
  graph.writeInput("count", 9);
  const definition = graph.exportDefinition();
  const snapshot = graph.exportSnapshot();
  const outputId = graph.output("doubled").id;
  const compatibilityImportedSignals = await createSignals({
    deployment: "mainThreadCompatibility",
  });
  const compatibilityImportedGraph = compatibilityImportedSignals.importGraph(definition, snapshot);
  await compatibilityImportedGraph.ready();

  try {
    const workerSignals = await createSignals();
    const importedGraph = workerSignals.importGraph(definition, snapshot);
    await importedGraph.ready();

    assert.deepEqual(
      workerSignals.diagnostics().why(outputId),
      compatibilityImportedSignals.diagnostics().why(outputId),
    );
    assert.deepEqual(
      workerSignals.diagnostics().health(),
      compatibilityImportedSignals.diagnostics().health(),
    );
    assert.deepEqual(
      comparableGraphSummary(workerSignals.diagnostics().summaryNow()),
      comparableGraphSummary(compatibilityImportedSignals.diagnostics().summaryNow()),
    );
    assert.deepEqual(
      comparableHistorySurfaceSummary(workerSignals.diagnostics().historyNow()),
      comparableHistorySurfaceSummary(compatibilityImportedSignals.diagnostics().historyNow()),
    );
    assert.equal(workerSignals.diagnostics().latestHostCapabilityEvent(), null);
    assert.deepEqual(workerSignals.diagnostics().recentHostCapabilityEvents(), []);
    assert.equal(
      workerSignals.diagnostics().hostCapabilityReport().totals.registrationCount,
      0,
    );
    assert.equal(
      workerSignals.diagnostics().hostCapabilityReport().totals.retainedEventCount,
      0,
    );
    assert.deepEqual(
      comparablePerformanceSummary(workerSignals.diagnostics().performanceSummary()),
      comparablePerformanceSummary(compatibilityImportedSignals.diagnostics().performanceSummary()),
    );
    assert.equal(
      workerSignals.read(outputId),
      compatibilityImportedSignals.read(outputId),
    );
    assert.equal(
      workerSignals.read(importedGraph.output("doubled")),
      compatibilityImportedSignals.read(outputId),
    );

    assert.deepEqual(
      workerSignals.history().replay_for(outputId),
      compatibilityImportedSignals.history().replay_for(outputId),
    );
    assert.deepEqual(
      workerSignals.history().lineage_for(outputId),
      compatibilityImportedSignals.history().lineage_for(outputId),
    );
    assert.deepEqual(
      comparableRecentHistory(workerSignals.history().recentHistory()),
      comparableRecentHistory(compatibilityImportedSignals.diagnostics().recentHistory()),
    );
    assert.deepEqual(
      comparableSnapshotEnvelope(workerSignals.history().snapshot()),
      comparableSnapshotEnvelope(compatibilityImportedSignals.history().snapshot()),
    );
    assert.deepEqual(
      comparableRuntimeBranch(workerSignals.history().current_branch()),
      comparableRuntimeBranch(compatibilityImportedSignals.history().current_branch()),
    );
    assert.deepEqual(
      workerSignals.history().branches().map(comparableRuntimeBranch),
      compatibilityImportedSignals.history().branches().map(comparableRuntimeBranch),
    );
    const workerBranchReplay = workerSignals.history().replay_for_branch(0);
    assert.ok(workerBranchReplay.frames.length >= 1);
    assert.equal(
      workerBranchReplay.frames[workerBranchReplay.frames.length - 1].branchId,
      0,
    );
    const workerBranchSnapshot = workerSignals.history().branch_snapshot(0);
    assert.equal(
      workerSignals.history().branch_snapshot_id(0),
      workerSignals.history().branch_snapshot_envelope(0).snapshot.meta.snapshot_id,
    );
    assert.equal(typeof workerBranchSnapshot.snapshotRestoreToken, "string");
    assert.deepEqual(
      comparableSnapshotEnvelope(workerSignals.history().branch_snapshot_envelope(0)),
      comparableSnapshotEnvelope(compatibilityImportedSignals.history().branch_snapshot_envelope(0)),
    );
    assert.deepEqual(
      comparableSnapshotArtifact(workerBranchSnapshot),
      comparableSnapshotArtifact(compatibilityImportedSignals.history().branch_snapshot(0)),
    );
    assert.deepEqual(
      comparableBranchStateProof(workerSignals.history().branch_state_proof(0)),
      comparableBranchStateProof(compatibilityImportedSignals.history().branch_state_proof(0)),
    );
    assert.deepEqual(
      comparableReplayParityProof(workerSignals.history().replay_parity_proof(0, 0)),
      comparableReplayParityProof(compatibilityImportedSignals.history().replay_parity_proof(0, 0)),
    );
    const replayArtifactInput = {
      proofSchemaVersion: compatibilityImportedSignals.adapters().runtimeProofReport().proofSchemaVersion,
      registryBundleDigest: compatibilityImportedSignals.adapters().runtimeProofReport().registryBundleDigest,
      loweredStrategyBundleDigest: null,
      mergePlanDigest: null,
      mergeResultDigest: null,
      lineageDigest: null,
      branchStateDigest: compatibilityImportedSignals.history().branch_state_proof(0).stateDigest,
    };
    assert.deepEqual(
      workerSignals.history().replay_artifact_proof(replayArtifactInput, 0),
      compatibilityImportedSignals.history().replay_artifact_proof(replayArtifactInput, 0),
    );
    const workerHistorySnapshot = workerSignals.history().snapshot();
    const compatibilityHistorySnapshot = compatibilityImportedSignals.history().snapshot();
    const workerFeatureBranch = await workerSignals.history().create_branch("feature");
    const compatibilityFeatureBranch = compatibilityImportedSignals.history().create_branch("feature");
    assert.deepEqual(
      comparableRuntimeBranch(workerFeatureBranch),
      comparableRuntimeBranch(compatibilityFeatureBranch),
    );
    await workerSignals.history().switch_branch(workerFeatureBranch.id);
    compatibilityImportedSignals.history().switch_branch(compatibilityFeatureBranch.id);
    await importedGraph.writeInput("count", 11);
    await compatibilityImportedGraph.writeInput("count", 11);
    assert.equal(workerSignals.read(outputId), compatibilityImportedSignals.read(outputId));
    const workerFeatureSnapshot = workerSignals.history().branch_snapshot(workerFeatureBranch.id);
    const compatibilityFeatureSnapshot = compatibilityImportedSignals.history().branch_snapshot(
      compatibilityFeatureBranch.id,
    );
    assert.deepEqual(
      comparableSnapshotArtifact(workerFeatureSnapshot),
      comparableSnapshotArtifact(compatibilityFeatureSnapshot),
    );
    await workerSignals.history().restore_exact_snapshot(workerHistorySnapshot);
    compatibilityImportedSignals.history().restore_exact_snapshot(compatibilityHistorySnapshot);
    assert.equal(workerSignals.read(outputId), compatibilityImportedSignals.read(outputId));
    await workerSignals.history().switch_branch(workerFeatureBranch.id);
    compatibilityImportedSignals.history().switch_branch(compatibilityFeatureBranch.id);
    await workerSignals.history().restore_exact_branch_snapshot(
      workerFeatureBranch.id,
      workerFeatureSnapshot,
    );
    compatibilityImportedSignals.history().restore_exact_branch_snapshot(
      compatibilityFeatureBranch.id,
      compatibilityFeatureSnapshot,
    );
    await workerSignals.history().restore_branch_snapshot_by_id(
      workerFeatureBranch.id,
      workerSignals.history().branch_snapshot_id(workerFeatureBranch.id),
    );
    compatibilityImportedSignals.history().restore_branch_snapshot_by_id(
      compatibilityFeatureBranch.id,
      compatibilityImportedSignals.history().branch_snapshot_id(compatibilityFeatureBranch.id),
    );
    assert.equal(workerSignals.read(outputId), compatibilityImportedSignals.read(outputId));
    const workerMergePlan = await workerSignals.history().plan_merge_branches(
      workerFeatureBranch.id,
      0,
    );
    assert.equal(workerMergePlan.source_branch_id, workerFeatureBranch.id);
    assert.equal(workerMergePlan.target_branch_id, 0);
    assert.equal(typeof workerMergePlan.merge_kind, "string");
    const workerMergePlanProof = await workerSignals.history().plan_merge_policy_preview_with_proof({
      source_branch_id: workerFeatureBranch.id,
      target_branch_id: 0,
    });
    assert.deepEqual(
      comparableMergePlan(workerMergePlanProof.plan),
      comparableMergePlan(workerMergePlan),
    );
    assert.equal(typeof workerMergePlanProof.proof?.proofSchemaVersion, "string");
    const workerMergePreview = await workerSignals.history().merge_branches_policy_preview({
      source_branch_id: workerFeatureBranch.id,
      target_branch_id: 0,
    });
    assert.equal(typeof workerMergePreview.merge_kind, "string");
    const workerMergePreviewProof = await workerSignals.history().merge_branches_policy_preview_with_proof({
      source_branch_id: workerFeatureBranch.id,
      target_branch_id: 0,
    });
    assert.deepEqual(
      comparableMergeResult(workerMergePreviewProof.result),
      comparableMergeResult(workerMergePreview),
    );
    assert.equal(typeof workerMergePreviewProof.proof?.proofSchemaVersion, "string");
    const workerAppliedMerge = await workerSignals.history().merge_branches(workerFeatureBranch.id, 0);
    assert.deepEqual(
      comparableMergeResult(workerAppliedMerge),
      comparableMergeResult(workerMergePreview),
    );
    compatibilityImportedSignals.history().merge_branches(compatibilityFeatureBranch.id, 0);
    assert.equal(workerSignals.read(outputId), compatibilityImportedSignals.read(outputId));
    assert.deepEqual(
      comparableRuntimeBranch(workerSignals.history().current_branch()),
      comparableRuntimeBranch(compatibilityImportedSignals.history().current_branch()),
    );

    const workerProofBranch = await workerSignals.history().create_branch("feature-proof");
    const compatibilityProofBranch = compatibilityImportedSignals.history().create_branch("feature-proof");
    await workerSignals.history().switch_branch(workerProofBranch.id);
    compatibilityImportedSignals.history().switch_branch(compatibilityProofBranch.id);
    await importedGraph.writeInput("count", 17);
    await compatibilityImportedGraph.writeInput("count", 17);
    assert.deepEqual(
      comparableMergeResultProof(
        await workerSignals.history().merge_branches_with_proof(workerProofBranch.id, 0),
      ),
      comparableMergeResultProof(
        compatibilityImportedSignals.history().merge_branches_with_proof(compatibilityProofBranch.id, 0),
      ),
    );
    assert.equal(workerSignals.read(outputId), compatibilityImportedSignals.read(outputId));

    // Exact runtime restore artifacts are root-branch evidence on both deployments: while the
    // proof branch is still active, every exact export refuses with the same denial, and
    // definitions stay readable.
    assert.deepEqual(
      comparableRuntimeBranch(workerSignals.history().current_branch()),
      comparableRuntimeBranch(compatibilityImportedSignals.history().current_branch()),
    );
    assert.notEqual(workerSignals.history().current_branch().parent_branch_id, null);
    assert.throws(() => workerSignals.adapters().exportRuntimeEnvelope(), { code: "invalidInput" });
    assert.throws(
      () => compatibilityImportedSignals.adapters().exportRuntimeEnvelope(),
      { code: "invalidInput" },
    );
    assert.throws(() => importedGraph.exportSnapshot(), { code: "invalidInput" });
    assert.throws(() => compatibilityImportedGraph.exportSnapshot(), { code: "invalidInput" });
    assert.deepEqual(
      workerSignals.adapters().exportDefinitions(),
      compatibilityImportedSignals.adapters().exportDefinitions(),
    );
    await workerSignals.history().switch_branch(0);
    compatibilityImportedSignals.history().switch_branch(0);
    assert.equal(workerSignals.history().current_branch().parent_branch_id, null);
    assert.equal(workerSignals.read(outputId), compatibilityImportedSignals.read(outputId));

    assert.deepEqual(
      workerSignals.adapters().exportDefinitions(),
      compatibilityImportedSignals.adapters().exportDefinitions(),
    );
    assert.deepEqual(
      workerSignals.adapters().exportRuntimeEnvelope().definitions,
      compatibilityImportedSignals.adapters().exportDefinitions(),
    );
    assert.ok(workerSignals.adapters().exportRuntimeEnvelope().snapshot);
    assert.equal(
      typeof workerSignals.adapters().exportRuntimeEnvelope().runtimeEnvelopeRestoreToken,
      "string",
    );
    assert.equal(
      typeof workerSignals.adapters().exportRuntimeEnvelope().runtimeEnvelopePortableWire,
      "string",
    );
    assert.equal(
      workerSignals.adapters().exportRuntimeEnvelope().runtimeEnvelopeRestoreMode,
      "SameRuntimeExact",
    );
    assert.deepEqual(
      workerSignals.adapters().runtimeProofReport(),
      compatibilityImportedSignals.adapters().runtimeProofReport(),
    );
    assert.deepEqual(
      workerSignals.adapters().hostCapabilityTransportReport(),
      compatibilityImportedSignals.adapters().hostCapabilityTransportReport(
        compatibilityImportedSignals.adapters().exportRuntimeEnvelope(),
      ),
    );
    assert.deepEqual(
      comparableGraphSummary(workerSignals.specialist().graphSummary()),
      comparableGraphSummary(compatibilityImportedSignals.specialist().graphSummary()),
    );
    assert.deepEqual(
      comparableGraphSummary(workerSignals.specialist().graph_summary()),
      comparableGraphSummary(compatibilityImportedSignals.specialist().graph_summary()),
    );
    assert.deepEqual(
      workerSignals.specialist().readVersions([outputId]),
      compatibilityImportedSignals.specialist().readVersions([outputId]),
    );
    assert.deepEqual(
      workerSignals.specialist().read_versions([outputId]),
      compatibilityImportedSignals.specialist().read_versions([outputId]),
    );
    assert.deepEqual(
      await workerSignals.specialist().evaluateDirty(),
      compatibilityImportedSignals.specialist().evaluateDirty(),
    );
    assert.deepEqual(
      await workerSignals.specialist().evaluate_dirty(),
      compatibilityImportedSignals.specialist().evaluate_dirty(),
    );
    assert.throws(
      () => workerSignals.diagnostics().why("not-an-active-id"),
      /active imported graph/,
    );
    assert.throws(
      () => workerSignals.read("not-an-active-id"),
      /currently available worker-first signal/,
    );
    assert.throws(
      () => workerSignals.specialist().readVersions(["not-an-active-id"]),
      /active imported graph/,
    );

    const notices = [];
    let effectCount = 0;
    const watchHandle = workerSignals.watch(outputId, (notice) => {
      notices.push(notice);
    });
    const effectHandle = workerSignals.effect(importedGraph.output("doubled"), () => {
      effectCount += 1;
    });

    graph.writeInput("count", 13);
    const changedSnapshot = graph.exportSnapshot();
    const reimportedGraph = workerSignals.importGraph(definition, changedSnapshot);
    await reimportedGraph.ready();
    assert.equal(workerSignals.read(outputId), 26);
    assert.equal(notices.length, 1);
    assert.equal(notices[0].signalId, outputId);
    assert.equal(notices[0].meaningfulChange, true);
    assert.equal(effectCount, 1);
    assert.equal(workerSignals.nuke(watchHandle), true);
    assert.equal(workerSignals.nuke(watchHandle), false);
    assert.equal(workerSignals.nuke(effectHandle), true);

    const unchangedGraph = workerSignals.importGraph(definition, changedSnapshot);
    await unchangedGraph.ready();
    assert.equal(notices.length, 1);
    assert.equal(effectCount, 1);

    await unchangedGraph.terminate();
    await reimportedGraph.terminate();
    await importedGraph.terminate();
    await workerSignals.terminate();
  } finally {
    await compatibilityImportedGraph.terminate();
    compatibilityImportedSignals.free();
    compatibilitySignals.free();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});
