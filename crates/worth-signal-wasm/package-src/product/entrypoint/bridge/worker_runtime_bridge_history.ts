import {
  normalizeWorkerBranchId,
} from "./worker_runtime_bridge_support.js";

function createWorkerRuntimeBridgeHistory(request) {
  const branchId = (value, operation) =>
    normalizeWorkerBranchId(value, operation);
  const branchRequest = (method, value, operation) =>
    request(method, branchId(value, operation));

  return Object.freeze({
    currentBranch: () => request("currentBranch"),
    branches: () => request("branches"),
    createBranch: (name) => request("createBranch", name),
    workerBranchBasis: (id) =>
      branchRequest("workerBranchBasis", id, "workerBranchBasis"),
    forkBranch(value) {
      return request("forkBranch", {
        ...value,
        parentBranchId: branchId(
          value?.parentBranchId,
          "forkBranch.parentBranchId",
        ),
      });
    },
    applyTransactionToBranch(value) {
      return request("applyTransactionToBranch", {
        ...value,
        branchId: branchId(
          value?.branchId,
          "applyTransactionToBranch.branchId",
        ),
      });
    },
    retireBranch(value) {
      return request("retireBranch", {
        ...value,
        branchId: branchId(value?.branchId, "retireBranch.branchId"),
      });
    },
    retireBranches(value) {
      return request("retireBranches", {
        ...value,
        retirements: value?.retirements?.map((retirement, index) => ({
          ...retirement,
          branchId: branchId(
            retirement?.branchId,
            `retireBranches.retirements[${index}].branchId`,
          ),
        })),
      });
    },
    closeoutEffectBranch(value) {
      return request("closeoutEffectBranch", normalizeCloseoutRequest(value, branchId));
    },
    switchBranch: (id) => branchRequest("switchBranch", id, "switchBranch"),
    replayForBranch: (id) =>
      branchRequest("replayForBranch", id, "replayForBranch"),
    branchSnapshotId: (id) =>
      branchRequest("branchSnapshotId", id, "branchSnapshotId"),
    branchSnapshotEnvelope: (id) =>
      branchRequest("branchSnapshotEnvelope", id, "branchSnapshotEnvelope"),
    branchSnapshotArtifact: (id) =>
      branchRequest("branchSnapshotArtifact", id, "branchSnapshotArtifact"),
    branchSnapshotEnvelopeArtifact: (id) => branchRequest(
      "branchSnapshotEnvelopeArtifact",
      id,
      "branchSnapshotEnvelopeArtifact",
    ),
    branchSnapshotEnvelopeWire: (id) => branchRequest(
      "branchSnapshotEnvelopeWire",
      id,
      "branchSnapshotEnvelopeWire",
    ),
    branchSnapshotEnvelopePortableWire: (id) => branchRequest(
      "branchSnapshotEnvelopePortableWire",
      id,
      "branchSnapshotEnvelopePortableWire",
    ),
    restoreBranchSnapshotArtifact: (id, snapshot) => request(
      "restoreBranchSnapshotArtifact",
      branchId(id, "restoreBranchSnapshotArtifact"),
      snapshot,
    ),
    restoreBranchSnapshotWire: (id, snapshot) => request(
      "restoreBranchSnapshotWire",
      branchId(id, "restoreBranchSnapshotWire"),
      snapshot,
    ),
    restoreBranchSnapshotPortableWire: (id, snapshot) => request(
      "restoreBranchSnapshotPortableWire",
      branchId(id, "restoreBranchSnapshotPortableWire"),
      snapshot,
    ),
    restoreBranchSnapshotById: (id, snapshotId) => request(
      "restoreBranchSnapshotById",
      branchId(id, "restoreBranchSnapshotById"),
      branchId(snapshotId, "restoreBranchSnapshotById.snapshotId"),
    ),
    branchStateProof: (id) =>
      branchRequest("branchStateProof", id, "branchStateProof"),
  });
}

function normalizeCloseoutRequest(value, branchId) {
  const normalizeRetirement = (retirement, operation) => retirement == null
    ? null
    : {
        ...retirement,
        branchId: branchId(retirement.branchId, `${operation}.branchId`),
      };
  return {
    ...value,
    canonicalTransaction: {
      ...value?.canonicalTransaction,
      branchId: branchId(
        value?.canonicalTransaction?.branchId,
        "closeoutEffectBranch.canonicalTransaction.branchId",
      ),
    },
    effectRetirement: normalizeRetirement(
      value?.effectRetirement,
      "closeoutEffectBranch.effectRetirement",
    ),
    dependencyBasisRetirement: normalizeRetirement(
      value?.dependencyBasisRetirement,
      "closeoutEffectBranch.dependencyBasisRetirement",
    ),
  };
}

export { createWorkerRuntimeBridgeHistory };
