import { buildActiveImportContext } from "./sessions/support/worker_first_root_import_context.js";
import {
  historyOperationMayDropAuthoredCatalog,
  tipBranchId,
} from "./sessions/support/authored/worker_first_authored_tip_catalog.js";

export function createWorkerFirstRootHistoryLifecycle(deps) {
  return Object.freeze({
    async createBranch(name) {
      await deps.ready();
      deps.requireActive("history.create_branch");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const branch = await deps.bridge.createBranch(name);
      await deps.refreshAfterHistoryMutation("history.create_branch", deps.activeImportContext());
      return branch;
    },
    async switchBranch(branchId) {
      await deps.ready();
      deps.requireActive("history.switch_branch");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      await deps.bridge.switchBranch(branchId);
      await deps.refreshAfterHistoryMutation("history.switch_branch", deps.activeImportContext());
    },
    async restoreSnapshotEnvelope(snapshotEnvelope) {
      await deps.ready();
      deps.requireActive("history.restore_snapshot");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreSnapshotEnvelope(snapshotEnvelope);
      await deps.refreshAfterHistoryMutation("history.restore_snapshot", activeImportContext);
    },
    async restoreExactSnapshotEnvelope(restoreToken) {
      await deps.ready();
      deps.requireActive("history.restore_exact_snapshot");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreSnapshotEnvelopeWire(restoreToken);
      await deps.refreshAfterHistoryMutation("history.restore_exact_snapshot", activeImportContext);
    },
    async restorePortableSnapshotEnvelope(portableWire) {
      await deps.ready();
      deps.requireActive("history.restore_snapshot");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreSnapshotEnvelopePortableWire(portableWire);
      await deps.refreshAfterHistoryMutation("history.restore_snapshot", activeImportContext);
    },
    async restoreBranchSnapshot(branchId, snapshot) {
      await deps.ready();
      deps.requireActive("history.restore_branch_snapshot");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreBranchSnapshotArtifact(branchId, snapshot);
      await deps.refreshAfterHistoryMutation("history.restore_branch_snapshot", activeImportContext);
    },
    async restoreExactBranchSnapshot(branchId, restoreToken) {
      await deps.ready();
      deps.requireActive("history.restore_exact_branch_snapshot");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreBranchSnapshotWire(branchId, restoreToken);
      await deps.refreshAfterHistoryMutation(
        "history.restore_exact_branch_snapshot",
        activeImportContext,
      );
    },
    async restorePortableBranchSnapshot(branchId, portableWire) {
      await deps.ready();
      deps.requireActive("history.restore_branch_snapshot");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreBranchSnapshotPortableWire(branchId, portableWire);
      await deps.refreshAfterHistoryMutation("history.restore_branch_snapshot", activeImportContext);
    },
    async restoreBranchSnapshotById(branchId, snapshotId) {
      await deps.ready();
      deps.requireActive("history.restore_branch_snapshot_by_id");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const activeImportContext = deps.activeImportContext();
      await deps.bridge.restoreBranchSnapshotById(branchId, snapshotId);
      await deps.refreshAfterHistoryMutation(
        "history.restore_branch_snapshot_by_id",
        activeImportContext,
      );
    },
    async evaluateDirty() {
      await deps.ready();
      deps.requireActive("specialist.evaluateDirty");
      await deps.authoredRuntime.settlePendingPublications();
      await deps.settlePendingMutations();
      const runSummary = await deps.bridge.evaluateDirty();
      await deps.refreshBranchCache();
      if (deps.activeImportContext() !== null) {
        await deps.refreshActiveImportContext();
      }
      await deps.authoredRuntime.refreshAllReadables();
      return runSummary;
    },
  });
}

export async function refreshWorkerFirstRootAfterHistoryMutation(deps, operation, activeImportContext) {
  const activeImportController = deps.activeImportController();
  if (activeImportController === null || activeImportContext === null) {
    const previousTipBranchId = tipBranchId(
      typeof deps.readCachedCurrentBranch === "function"
        ? deps.readCachedCurrentBranch()
        : null,
    );
    await deps.refreshBranchCache();
    const nextTipBranchId = tipBranchId(
      typeof deps.readCachedCurrentBranch === "function"
        ? deps.readCachedCurrentBranch()
        : null,
    );
    // Re-admit only when tip identity changes, or restore/merge rewrites catalog
    // contents in place. Avoid O(authored) probes on create_branch no-ops.
    if (
      historyOperationMayDropAuthoredCatalog(operation)
      || previousTipBranchId !== nextTipBranchId
    ) {
      deps.authoredRuntime.markActiveTipCatalogChanged();
      await deps.authoredRuntime.readmitReadyAuthoredOntoActiveTip();
    }
    await deps.authoredRuntime.refreshAllReadables();
    if (typeof deps.publishDiagnosticsChanged === "function") {
      await deps.publishDiagnosticsChanged();
    }
    return;
  }
  deps.authoredRuntime.invalidate(
    `worker-first root ${operation}(...) replaced the worker-owned runtime truth`,
  );
  deps.setActiveImportContext(
    await buildActiveImportContext(
      deps.bridge,
      activeImportContext.definition,
      activeImportContext.snapshot,
    ),
  );
  await deps.observations.replaceContext(deps.bridge, deps.activeImportContext());
  if (typeof activeImportController.refreshFromRootRuntime === "function") {
    await activeImportController.refreshFromRootRuntime();
  }
  deps.requireControllerActive(activeImportController, operation);
  if (typeof deps.publishDiagnosticsChanged === "function") {
    await deps.publishDiagnosticsChanged();
  }
}
