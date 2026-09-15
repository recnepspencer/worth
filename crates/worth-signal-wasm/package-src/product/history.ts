function attachSerializedField(value, field, serialized) {
  if (!value || typeof value !== "object" || typeof serialized !== "string") {
    return value;
  }
  Object.defineProperty(value, field, {
    value: serialized,
    enumerable: true,
    configurable: false,
    writable: false,
  });
  return value;
}

function attachLiteralField(value, field, literal) {
  if (!value || typeof value !== "object") {
    return value;
  }
  Object.defineProperty(value, field, {
    value: literal,
    enumerable: true,
    configurable: false,
    writable: false,
  });
  return value;
}

function notifyHistoryListeners(listeners) {
  for (const listener of Array.from(listeners)) {
    listener();
  }
}

function withHistoryMutationNotification(result, listeners) {
  if (result && typeof result.then === "function") {
    return result.then((value) => {
      notifyHistoryListeners(listeners);
      return value;
    });
  }
  notifyHistoryListeners(listeners);
  return result;
}

function snapshotEnvelopeRestoreToken(snapshot) {
  return snapshot?.snapshotEnvelopeRestoreToken;
}

function snapshotRestoreToken(snapshot) {
  return snapshot?.snapshotRestoreToken;
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

function normalizeBranchId(value, operation) {
  if (typeof value === "bigint") {
    if (value < 0n) {
      throw new RangeError(`${operation} expects a non-negative branch id`);
    }
    return value;
  }
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${operation} expects a non-negative safe integer branch id`);
  }
  return BigInt(value);
}

function normalizeSnapshotId(value, operation) {
  if (typeof value === "bigint") {
    if (value < 0n) {
      throw new RangeError(`${operation} expects a non-negative snapshot id`);
    }
    return value;
  }
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${operation} expects a non-negative safe integer snapshot id`);
  }
  return BigInt(value);
}

function normalizePreviewBranchId(value, operation) {
  if (typeof value === "bigint") {
    if (value < 0n) {
      throw new RangeError(`${operation} expects a non-negative branch id`);
    }
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
      throw new RangeError(`${operation} exceeds the safe integer range supported by merge preview requests`);
    }
    return Number(value);
  }
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${operation} expects a non-negative safe integer branch id`);
  }
  return value;
}

function normalizeMergePreviewRequest(request, operation) {
  if (!request || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError(`${operation} expects a merge preview request object`);
  }
  return {
    ...request,
    source_branch_id: normalizePreviewBranchId(
      request.source_branch_id,
      `${operation}.source_branch_id`,
    ),
    target_branch_id: normalizePreviewBranchId(
      request.target_branch_id,
      `${operation}.target_branch_id`,
    ),
  };
}

export function wrapHistory(rawHistory) {
  const listeners = new Set();
  return Object.freeze({
    replay_for(id) {
      return rawHistory.replay_for(id);
    },
    lineage_for(id) {
      return rawHistory.lineage_for(id);
    },
    snapshot() {
      const snapshot = attachSerializedField(
        rawHistory.snapshot(),
        "snapshotEnvelopeRestoreToken",
        rawHistory.snapshot_wire(),
      );
      attachSerializedField(
        snapshot,
        "snapshotEnvelopePortableWire",
        rawHistory.snapshot_portable_wire(),
      );
      return attachLiteralField(snapshot, "snapshotEnvelopeRestoreMode", "SameRuntimeExact");
    },
    restore_snapshot(snapshot) {
      if (typeof snapshot?.snapshotEnvelopePortableWire === "string") {
        return withHistoryMutationNotification(
          rawHistory.restore_snapshot_portable_wire(snapshot.snapshotEnvelopePortableWire),
          listeners,
        );
      }
      return withHistoryMutationNotification(
        rawHistory.restore_snapshot(snapshot),
        listeners,
      );
    },
    restore_exact_snapshot(snapshot) {
      const restoreToken = snapshotEnvelopeRestoreToken(snapshot);
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.restore_exact_snapshot expects an artifact returned by history.snapshot() or history.branch_snapshot_envelope()",
        );
      }
      return withHistoryMutationNotification(
        rawHistory.restore_snapshot_wire(restoreToken),
        listeners,
      );
    },
    discard_exact_artifact(artifact) {
      return rawHistory.discard_restore_token(
        exactArtifactRestoreToken(artifact, "history.discard_exact_artifact"),
      );
    },
    current_branch() {
      return rawHistory.current_branch();
    },
    branches() {
      return rawHistory.branches();
    },
    create_branch(name) {
      return withHistoryMutationNotification(rawHistory.create_branch(name), listeners);
    },
    worker_branch_basis(branchId) {
      return rawHistory.worker_branch_basis(
        normalizeBranchId(branchId, "history.worker_branch_basis"),
      );
    },
    fork_branch(request) {
      return withHistoryMutationNotification(rawHistory.fork_branch(request), listeners);
    },
    apply_transaction_to_branch(request) {
      return withHistoryMutationNotification(
        rawHistory.apply_transaction_to_branch(request),
        listeners,
      );
    },
    retire_branch(request) {
      return withHistoryMutationNotification(rawHistory.retire_branch(request), listeners);
    },
    retire_branches(request) {
      return withHistoryMutationNotification(rawHistory.retire_branches(request), listeners);
    },
    closeout_effect_branch(request) {
      return withHistoryMutationNotification(
        rawHistory.closeout_effect_branch(request),
        listeners,
      );
    },
    switch_branch(branchId) {
      return withHistoryMutationNotification(
        rawHistory.switch_branch(normalizeBranchId(branchId, "history.switch_branch")),
        listeners,
      );
    },
    replay_for_branch(branchId) {
      return rawHistory.replay_for_branch(normalizeBranchId(branchId, "history.replay_for_branch"));
    },
    branch_snapshot(branchId) {
      const normalizedBranchId = normalizeBranchId(branchId, "history.branch_snapshot");
      const snapshot = attachSerializedField(
        rawHistory.branch_snapshot(normalizedBranchId),
        "snapshotRestoreToken",
        rawHistory.branch_snapshot_wire(normalizedBranchId),
      );
      attachSerializedField(
        snapshot,
        "snapshotPortableWire",
        rawHistory.branch_snapshot_portable_wire(normalizedBranchId),
      );
      return attachLiteralField(snapshot, "snapshotRestoreMode", "SameRuntimeExact");
    },
    branch_snapshot_id(branchId) {
      return rawHistory.branch_snapshot_id(normalizeBranchId(branchId, "history.branch_snapshot_id"));
    },
    branch_snapshot_envelope(branchId) {
      const normalizedBranchId = normalizeBranchId(branchId, "history.branch_snapshot_envelope");
      const envelope = attachSerializedField(
        rawHistory.branch_snapshot_envelope(normalizedBranchId),
        "snapshotEnvelopeRestoreToken",
        rawHistory.branch_snapshot_envelope_wire(normalizedBranchId),
      );
      attachSerializedField(
        envelope,
        "snapshotEnvelopePortableWire",
        rawHistory.branch_snapshot_envelope_portable_wire(normalizedBranchId),
      );
      return attachLiteralField(envelope, "snapshotEnvelopeRestoreMode", "SameRuntimeExact");
    },
    restore_branch_snapshot(branchId, snapshot) {
      const normalizedBranchId = normalizeBranchId(branchId, "history.restore_branch_snapshot");
      if (typeof snapshot?.snapshotPortableWire === "string") {
        return withHistoryMutationNotification(
          rawHistory.restore_branch_snapshot_portable_wire(
            normalizedBranchId,
            snapshot.snapshotPortableWire,
          ),
          listeners,
        );
      }
      return withHistoryMutationNotification(
        rawHistory.restore_branch_snapshot(
          normalizedBranchId,
          snapshot,
        ),
        listeners,
      );
    },
    restore_exact_branch_snapshot(branchId, snapshot) {
      const normalizedBranchId = normalizeBranchId(branchId, "history.restore_exact_branch_snapshot");
      const restoreToken = snapshotRestoreToken(snapshot);
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "history.restore_exact_branch_snapshot expects an artifact returned by history.branch_snapshot()",
        );
      }
      return withHistoryMutationNotification(
        rawHistory.restore_branch_snapshot_wire(normalizedBranchId, restoreToken),
        listeners,
      );
    },
    restore_branch_snapshot_by_id(branchId, snapshotId) {
      return withHistoryMutationNotification(
        rawHistory.restore_branch_snapshot_by_id(
          normalizeBranchId(branchId, "history.restore_branch_snapshot_by_id"),
          normalizeSnapshotId(snapshotId, "history.restore_branch_snapshot_by_id"),
        ),
        listeners,
      );
    },
    merge_branches(sourceBranchId, targetBranchId) {
      return withHistoryMutationNotification(
        rawHistory.merge_branches(
          normalizeBranchId(sourceBranchId, "history.merge_branches"),
          normalizeBranchId(targetBranchId, "history.merge_branches"),
        ),
        listeners,
      );
    },
    merge_branches_with_proof(sourceBranchId, targetBranchId) {
      return withHistoryMutationNotification(
        rawHistory.merge_branches_with_proof(
          normalizeBranchId(sourceBranchId, "history.merge_branches_with_proof"),
          normalizeBranchId(targetBranchId, "history.merge_branches_with_proof"),
        ),
        listeners,
      );
    },
    plan_merge_branches(sourceBranchId, targetBranchId) {
      return rawHistory.plan_merge_branches(
        normalizeBranchId(sourceBranchId, "history.plan_merge_branches"),
        normalizeBranchId(targetBranchId, "history.plan_merge_branches"),
      );
    },
    plan_merge_branches_with_proof(sourceBranchId, targetBranchId) {
      return rawHistory.plan_merge_branches_with_proof(
        normalizeBranchId(sourceBranchId, "history.plan_merge_branches_with_proof"),
        normalizeBranchId(targetBranchId, "history.plan_merge_branches_with_proof"),
      );
    },
    plan_merge_policy_preview(request) {
      return rawHistory.plan_merge_policy_preview(
        normalizeMergePreviewRequest(request, "history.plan_merge_policy_preview"),
      );
    },
    plan_merge_policy_preview_with_proof(request) {
      return rawHistory.plan_merge_policy_preview_with_proof(
        normalizeMergePreviewRequest(request, "history.plan_merge_policy_preview_with_proof"),
      );
    },
    merge_branches_policy_preview(request) {
      return rawHistory.merge_branches_policy_preview(
        normalizeMergePreviewRequest(request, "history.merge_branches_policy_preview"),
      );
    },
    merge_branches_policy_preview_with_proof(request) {
      return rawHistory.merge_branches_policy_preview_with_proof(
        normalizeMergePreviewRequest(request, "history.merge_branches_policy_preview_with_proof"),
      );
    },
    branch_state_proof(branchId) {
      return rawHistory.branch_state_proof(normalizeBranchId(branchId, "history.branch_state_proof"));
    },
    replay_parity_proof(expectedBranchId, replayedBranchId) {
      return rawHistory.replay_parity_proof(
        normalizeBranchId(expectedBranchId, "history.replay_parity_proof"),
        normalizeBranchId(replayedBranchId, "history.replay_parity_proof"),
      );
    },
    replay_artifact_proof(expected, replayedBranchId) {
      return rawHistory.replay_artifact_proof(
        expected,
        normalizeBranchId(replayedBranchId, "history.replay_artifact_proof"),
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
    free() {
      rawHistory.free();
    },
    [Symbol.dispose]() {
      if (typeof rawHistory[Symbol.dispose] === "function") {
        rawHistory[Symbol.dispose]();
        return;
      }
      rawHistory.free();
    },
  });
}
