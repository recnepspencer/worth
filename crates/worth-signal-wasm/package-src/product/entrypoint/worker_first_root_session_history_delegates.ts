/** History / resource-branch / runtime-replacement delegates for the root session. */
export function createWorkerFirstRootSessionHistoryDelegates({
  historyLifecycle,
  resourceBranches,
  runtimeReplacement,
}) {
  return Object.freeze({
    replaceRuntimeEnvelope(envelope) {
      return runtimeReplacement.replaceRuntimeEnvelope(envelope);
    },
    restoreExactRuntimeEnvelope(envelope) {
      return runtimeReplacement.restoreExactRuntimeEnvelope(envelope);
    },
    createHistoryBranch(name) {
      return historyLifecycle.createBranch(name);
    },
    switchHistoryBranch(branchId) {
      return historyLifecycle.switchBranch(branchId);
    },
    restoreHistorySnapshotEnvelope(envelope) {
      return historyLifecycle.restoreSnapshotEnvelope(envelope);
    },
    restoreExactHistorySnapshotEnvelope(token) {
      return historyLifecycle.restoreExactSnapshotEnvelope(token);
    },
    restorePortableHistorySnapshotEnvelope(wire) {
      return historyLifecycle.restorePortableSnapshotEnvelope(wire);
    },
    restoreHistoryBranchSnapshot(branchId, snapshot) {
      return historyLifecycle.restoreBranchSnapshot(branchId, snapshot);
    },
    restoreExactHistoryBranchSnapshot(branchId, token) {
      return historyLifecycle.restoreExactBranchSnapshot(branchId, token);
    },
    restorePortableHistoryBranchSnapshot(branchId, wire) {
      return historyLifecycle.restorePortableBranchSnapshot(branchId, wire);
    },
    restoreHistoryBranchSnapshotById(branchId, snapshotId) {
      return historyLifecycle.restoreBranchSnapshotById(branchId, snapshotId);
    },
    workerBranchBasis(branchId) {
      return resourceBranches.basis(branchId);
    },
    forkResourceBranch(request) {
      return resourceBranches.fork(request);
    },
    applyResourceBranchTransaction(request) {
      return resourceBranches.applyTransaction(request);
    },
    retireResourceBranch(request) {
      return resourceBranches.retire(request);
    },
    retireResourceBranches(request) {
      return resourceBranches.retireBatch(request);
    },
    closeoutResourceEffectBranch(request) {
      return resourceBranches.closeoutEffect(request);
    },
    evaluateDirty() {
      return historyLifecycle.evaluateDirty();
    },
  });
}
