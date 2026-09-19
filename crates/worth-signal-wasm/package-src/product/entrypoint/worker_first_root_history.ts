import { freezeObject } from "../graph_support.js";
import {
  createReplayArtifactProofReport,
  createReplayParityProofReport,
  normalizeWorkerFirstBranchId,
} from "./sessions/support/worker_first_history_proofs.js";

const workerFirstRootHistoryFacadeBySession = new WeakMap();

export function createRootHistoryFacade(rootSession) {
  const listeners = new Set();

  function notifyListeners() {
    for (const listener of Array.from(listeners)) {
      listener();
    }
  }

  function withNotification(result) {
    if (result && typeof result.then === "function") {
      return result.then((value) => {
        notifyListeners();
        return value;
      });
    }
    notifyListeners();
    return result;
  }

  return freezeObject({
    replay_for(id) {
      const context = rootSession.currentImportContext();
      if (!context.replayById.has(id)) {
        throw new TypeError(
          `worker-first root history().replay_for(${JSON.stringify(id)}) requires an id from the active imported graph`,
        );
      }
      return context.replayById.get(id);
    },
    lineage_for(id) {
      const context = rootSession.currentImportContext();
      if (!context.lineageById.has(id)) {
        throw new TypeError(
          `worker-first root history().lineage_for(${JSON.stringify(id)}) requires an id from the active imported graph`,
        );
      }
      return context.lineageById.get(id);
    },
    recentHistory() {
      return rootSession.currentImportContext().recentHistory;
    },
    snapshot() {
      return rootSession.currentImportContext().snapshotEnvelope;
    },
    restore_snapshot(snapshot) {
      if (typeof snapshot?.snapshotEnvelopePortableWire === "string") {
        return withNotification(rootSession.restorePortableHistorySnapshotEnvelope(
          snapshot.snapshotEnvelopePortableWire,
        ));
      }
      return withNotification(rootSession.restoreHistorySnapshotEnvelope(snapshot));
    },
    restore_exact_snapshot(snapshot) {
      const restoreToken = snapshot?.snapshotEnvelopeRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.restore_exact_snapshot expects an artifact returned by history.snapshot() or history.branch_snapshot_envelope()",
        );
      }
      return withNotification(rootSession.restoreExactHistorySnapshotEnvelope(restoreToken));
    },
    discard_exact_artifact(artifact) {
      const restoreToken =
        artifact?.snapshotEnvelopeRestoreToken ?? artifact?.snapshotRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.discard_exact_artifact expects an artifact returned by history.snapshot(), history.branch_snapshot(), or history.branch_snapshot_envelope()",
        );
      }
      return rootSession.bridge().discardRestoreToken(restoreToken);
    },
    current_branch() {
      const branch = rootSession.currentBranchSummary();
      if (branch === null) {
        throw new TypeError(
          "worker-first root history().current_branch() requires current branch evidence from the worker-owned runtime",
        );
      }
      return branch;
    },
    branches() {
      return rootSession.branchesSummary();
    },
    worker_branch_basis(branchId) {
      return rootSession.workerBranchBasis(branchId);
    },
    fork_branch(request) {
      return withNotification(rootSession.forkResourceBranch(request));
    },
    apply_transaction_to_branch(request) {
      return withNotification(rootSession.applyResourceBranchTransaction(request));
    },
    retire_branch(request) {
      return withNotification(rootSession.retireResourceBranch(request));
    },
    retire_branches(request) {
      return withNotification(rootSession.retireResourceBranches(request));
    },
    closeout_effect_branch(request) {
      return withNotification(rootSession.closeoutResourceEffectBranch(request));
    },
    create_branch(name) {
      return withNotification(rootSession.createHistoryBranch(name));
    },
    switch_branch(branchId) {
      return withNotification(rootSession.switchHistoryBranch(branchId));
    },
    replay_for_branch(branchId) {
      return requireWorkerFirstBranchValue(
        rootSession,
        branchId,
        "history.replay_for_branch",
        "replayByBranchId",
      );
    },
    branch_snapshot(branchId) {
      return requireWorkerFirstBranchValue(
        rootSession,
        branchId,
        "history.branch_snapshot",
        "branchSnapshotArtifactByBranchId",
      );
    },
    branch_snapshot_id(branchId) {
      const currentBranch = rootSession.currentBranchSummary();
      if (
        currentBranch !== null
        && normalizeWorkerFirstBranchId(branchId, "history.branch_snapshot_id") === Number(currentBranch.id)
        && currentBranch.head_snapshot_id !== null
      ) {
        return Number(currentBranch.head_snapshot_id);
      }
      return requireWorkerFirstBranchValue(
        rootSession,
        branchId,
        "history.branch_snapshot_id",
        "branchSnapshotIdByBranchId",
      );
    },
    branch_snapshot_envelope(branchId) {
      return requireWorkerFirstBranchValue(
        rootSession,
        branchId,
        "history.branch_snapshot_envelope",
        "branchSnapshotEnvelopeByBranchId",
      );
    },
    restore_branch_snapshot(branchId, snapshot) {
      if (typeof snapshot?.snapshotPortableWire === "string") {
        return withNotification(rootSession.restorePortableHistoryBranchSnapshot(
          branchId,
          snapshot.snapshotPortableWire,
        ));
      }
      return withNotification(rootSession.restoreHistoryBranchSnapshot(branchId, snapshot));
    },
    restore_exact_branch_snapshot(branchId, snapshot) {
      const restoreToken = snapshot?.snapshotRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.restore_exact_branch_snapshot expects an artifact returned by history.branch_snapshot()",
        );
      }
      return withNotification(rootSession.restoreExactHistoryBranchSnapshot(branchId, restoreToken));
    },
    restore_branch_snapshot_by_id(branchId, snapshotId) {
      return withNotification(rootSession.restoreHistoryBranchSnapshotById(branchId, snapshotId));
    },
    merge_branches(sourceBranchId, targetBranchId) {
      return withNotification(rootSession.mergeHistoryBranches(sourceBranchId, targetBranchId));
    },
    merge_branches_with_proof(sourceBranchId, targetBranchId) {
      return withNotification(
        rootSession.mergeHistoryBranchesWithProof(sourceBranchId, targetBranchId),
      );
    },
    plan_merge_branches(sourceBranchId, targetBranchId) {
      return rootSession.bridge().planMergeBranches(sourceBranchId, targetBranchId);
    },
    plan_merge_branches_with_proof(sourceBranchId, targetBranchId) {
      return rootSession.bridge().planMergeBranchesWithProof(sourceBranchId, targetBranchId);
    },
    plan_merge_policy_preview(request) {
      return rootSession.bridge().planMergePolicyPreview(request);
    },
    plan_merge_policy_preview_with_proof(request) {
      return rootSession.bridge().planMergePolicyPreviewWithProof(request);
    },
    merge_branches_policy_preview(request) {
      return rootSession.mergeHistoryBranchesPolicyPreview(request);
    },
    merge_branches_policy_preview_with_proof(request) {
      return rootSession.mergeHistoryBranchesPolicyPreviewWithProof(request);
    },
    branch_state_proof(branchId) {
      return requireWorkerFirstBranchValue(
        rootSession,
        branchId,
        "history.branch_state_proof",
        "branchStateProofByBranchId",
      );
    },
    replay_parity_proof(expectedBranchId, replayedBranchId) {
      const context = rootSession.currentImportContext();
      return createReplayParityProofReport(
        context.runtimeProofReport.proofSchemaVersion,
        requireWorkerFirstBranchValue(
          rootSession,
          expectedBranchId,
          "history.replay_parity_proof",
          "branchStateProofByBranchId",
        ),
        requireWorkerFirstBranchValue(
          rootSession,
          replayedBranchId,
          "history.replay_parity_proof",
          "branchStateProofByBranchId",
        ),
      );
    },
    replay_artifact_proof(expected, replayedBranchId) {
      const context = rootSession.currentImportContext();
      return createReplayArtifactProofReport(
        context.runtimeProofReport.proofSchemaVersion,
        expected,
        freezeObject({
          proofSchemaVersion: context.runtimeProofReport.proofSchemaVersion,
          registryBundleDigest: context.runtimeProofReport.registryBundleDigest,
          loweredStrategyBundleDigest: null,
          mergePlanDigest: null,
          mergeResultDigest: null,
          lineageDigest: null,
          branchStateDigest: requireWorkerFirstBranchValue(
            rootSession,
            replayedBranchId,
            "history.replay_artifact_proof",
            "branchStateProofByBranchId",
          ).stateDigest,
        }),
      );
    },
    subscribe(listener) {
      if (typeof listener !== "function") {
        throw new TypeError("history.subscribe(...) requires a listener function");
      }
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    free() {},
    [Symbol.dispose]() {},
  });
}

export function readRootHistoryFacade(rootSession) {
  let facade = workerFirstRootHistoryFacadeBySession.get(rootSession);
  if (facade === undefined) {
    facade = createRootHistoryFacade(rootSession);
    workerFirstRootHistoryFacadeBySession.set(rootSession, facade);
  }
  return facade;
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

function requireWorkerFirstBranchValue(rootSession, branchId, operation, mapField) {
  const context = rootSession.currentImportContext();
  const branchMap = context[mapField];
  const normalizedBranchId = normalizeWorkerFirstBranchId(branchId, operation);
  if (!branchMap?.has(normalizedBranchId)) {
    throw new TypeError(
      `${operation}(${String(branchId)}) requires a branch from the active worker-first history context`,
    );
  }
  return branchMap.get(normalizedBranchId);
}
