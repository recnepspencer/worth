import { freezeObject } from "../graph_support.js";
import {
  createReplayArtifactProofReport,
  createReplayParityProofReport,
  createWorkerFirstSnapshotArtifact,
  createWorkerFirstSnapshotEnvelopeArtifact,
} from "./sessions/support/worker_first_history_proofs.js";

export function createWorkerFirstHistoryFacade(session) {
  let runtimeProofReportPromise = null;

  return Object.freeze({
    replay_for(id) {
      return session.bridge.replayFor(id);
    },
    lineage_for(id) {
      return session.bridge.lineageFor(id);
    },
    recentHistory() {
      return session.bridge.recentHistory();
    },
    async snapshot() {
      return createWorkerFirstSnapshotEnvelopeArtifact(
        await session.bridge.exportWorkerSnapshotEnvelopeArtifact(),
      );
    },
    restore_snapshot(snapshot) {
      if (typeof snapshot?.snapshotEnvelopePortableWire === "string") {
        return session.bridge.restoreSnapshotEnvelopePortableWire(
          snapshot.snapshotEnvelopePortableWire,
        );
      }
      return session.bridge.restoreSnapshotEnvelope(snapshot);
    },
    restore_exact_snapshot(snapshot) {
      const restoreToken = snapshot?.snapshotEnvelopeRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.restore_exact_snapshot expects an artifact returned by history.snapshot() or history.branch_snapshot_envelope()",
        );
      }
      return session.bridge.restoreSnapshotEnvelopeWire(restoreToken);
    },
    discard_exact_artifact(artifact) {
      return session.bridge.discardRestoreToken(
        exactArtifactRestoreToken(artifact, "history.discard_exact_artifact"),
      );
    },
    current_branch() {
      return session.bridge.currentBranch();
    },
    branches() {
      return session.bridge.branches();
    },
    create_branch(name) {
      return session.bridge.createBranch(name);
    },
    worker_branch_basis(branchId) {
      return session.bridge.workerBranchBasis(branchId);
    },
    fork_branch(request) {
      return session.bridge.forkBranch(request);
    },
    apply_transaction_to_branch(request) {
      return session.bridge.applyTransactionToBranch(request);
    },
    retire_branch(request) {
      return session.bridge.retireBranch(request);
    },
    retire_branches(request) {
      return session.bridge.retireBranches(request);
    },
    closeout_effect_branch(request) {
      return session.bridge.closeoutEffectBranch(request);
    },
    switch_branch(branchId) {
      return session.bridge.switchBranch(branchId);
    },
    replay_for_branch(branchId) {
      return session.bridge.replayForBranch(branchId);
    },
    branch_snapshot(branchId) {
      return session.bridge.branchSnapshotArtifact(branchId)
        .then((artifact) => createWorkerFirstSnapshotArtifact(artifact));
    },
    branch_snapshot_id(branchId) {
      return session.bridge.branchSnapshotArtifact(branchId)
        .then((artifact) => artifact.snapshot.meta.snapshot_id);
    },
    async branch_snapshot_envelope(branchId) {
      return createWorkerFirstSnapshotEnvelopeArtifact(
        await session.bridge.branchSnapshotEnvelopeArtifact(branchId),
      );
    },
    restore_branch_snapshot(branchId, snapshot) {
      if (typeof snapshot?.snapshotPortableWire === "string") {
        return session.bridge.restoreBranchSnapshotPortableWire(
          branchId,
          snapshot.snapshotPortableWire,
        );
      }
      return session.bridge.restoreBranchSnapshotArtifact(branchId, snapshot);
    },
    restore_exact_branch_snapshot(branchId, snapshot) {
      const restoreToken = snapshot?.snapshotRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.restore_exact_branch_snapshot expects an artifact returned by history.branch_snapshot()",
        );
      }
      return session.bridge.restoreBranchSnapshotWire(branchId, restoreToken);
    },
    restore_branch_snapshot_by_id(branchId, snapshotId) {
      return session.bridge.restoreBranchSnapshotById(branchId, snapshotId);
    },
    merge_branches(sourceBranchId, targetBranchId) {
      return session.bridge.mergeBranches(sourceBranchId, targetBranchId);
    },
    merge_branches_with_proof(sourceBranchId, targetBranchId) {
      return session.bridge.mergeBranchesWithProof(sourceBranchId, targetBranchId);
    },
    plan_merge_branches(sourceBranchId, targetBranchId) {
      return session.bridge.planMergeBranches(sourceBranchId, targetBranchId);
    },
    plan_merge_branches_with_proof(sourceBranchId, targetBranchId) {
      return session.bridge.planMergeBranchesWithProof(sourceBranchId, targetBranchId);
    },
    plan_merge_policy_preview(request) {
      return session.bridge.planMergePolicyPreview(request);
    },
    plan_merge_policy_preview_with_proof(request) {
      return session.bridge.planMergePolicyPreviewWithProof(request);
    },
    merge_branches_policy_preview(request) {
      return session.bridge.mergeBranchesPolicyPreview(request);
    },
    merge_branches_policy_preview_with_proof(request) {
      return session.bridge.mergeBranchesPolicyPreviewWithProof(request);
    },
    branch_state_proof(branchId) {
      return session.bridge.branchStateProof(branchId);
    },
    replay_parity_proof(expectedBranchId, replayedBranchId) {
      return Promise.all([
        runtimeProofReport(),
        session.bridge.branchStateProof(expectedBranchId),
        session.bridge.branchStateProof(replayedBranchId),
      ]).then(([runtimeProofReportValue, expected, replayed]) => createReplayParityProofReport(
        runtimeProofReportValue.proofSchemaVersion,
        expected,
        replayed,
      ));
    },
    replay_artifact_proof(expected, replayedBranchId) {
      return Promise.all([
        runtimeProofReport(),
        session.bridge.branchStateProof(replayedBranchId),
      ]).then(([runtimeProofReportValue, replayedState]) => createReplayArtifactProofReport(
        runtimeProofReportValue.proofSchemaVersion,
        expected,
        freezeObject({
          proofSchemaVersion: runtimeProofReportValue.proofSchemaVersion,
          registryBundleDigest: runtimeProofReportValue.registryBundleDigest,
          loweredStrategyBundleDigest: null,
          mergePlanDigest: null,
          mergeResultDigest: null,
          lineageDigest: null,
          branchStateDigest: replayedState.stateDigest,
        }),
      ));
    },
    free() {},
    [Symbol.dispose]() {},
  });

  function runtimeProofReport() {
    runtimeProofReportPromise ??= session.bridge.runtimeProofReport();
    return runtimeProofReportPromise;
  }
}

function exactArtifactRestoreToken(artifact, operation) {
  const restoreToken = artifact?.snapshotEnvelopeRestoreToken ?? artifact?.snapshotRestoreToken;
  if (typeof restoreToken !== "string") {
    throw new TypeError(
      `${operation} expects an artifact returned by history.snapshot(), history.branch_snapshot(), or history.branch_snapshot_envelope()`,
    );
  }
  return restoreToken;
}

function throwWorkerFirstHistoryUnavailable(operation) {
  const error = new Error(
    `${operation} is unavailable on the current worker-first history facade; use deployment: "mainThreadCompatibility" for branch and snapshot history operations`,
  );
  error.name = "WorkerFirstHistoryUnavailable";
  error.code = "workerFirstHistoryUnavailable";
  error.compatibilityRecovery = Object.freeze({
    deployment: "mainThreadCompatibility",
    message:
      'Retry with deployment: "mainThreadCompatibility" to use full branch and snapshot history operations.',
  });
  throw error;
}
