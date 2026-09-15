export function comparablePerformanceSummary(summary) {
  if (Array.isArray(summary)) {
    return summary.map(comparablePerformanceSummary);
  }
  if (!summary || typeof summary !== "object") {
    return summary;
  }
  const comparable = {};
  for (const [key, value] of Object.entries(summary)) {
    if (
      key.endsWith("_nanos") ||
      key.startsWith("hostCapability") ||
      key === "activeCallbackCount" ||
      key === "activeComputeCallbackCount" ||
      key.startsWith("computeCallback") ||
      // Executor usage counts how many execution runs this runtime has
      // performed. An exact restore carries the source runtime's count; a
      // portable rebuild starts at zero and adds its own refresh reads. Like
      // timings, that describes how the runtime got here, not the graph it
      // holds, so it is not part of cross-deployment surface parity.
      key === "serialExecutorUsageCount" ||
      key === "parallelExecutorUsageCount"
    ) {
      continue;
    }
    comparable[key] = comparablePerformanceSummary(value);
  }
  return comparable;
}

export function comparableGraphSummary(summary) {
  if (Array.isArray(summary)) {
    return summary.map(comparableGraphSummary);
  }
  if (!summary || typeof summary !== "object") {
    return summary;
  }
  const comparable = {};
  for (const [key, value] of Object.entries(summary)) {
    if (
      key.endsWith("_nanos")
      || key === "nodes_with_execution_record"
      || key === "sample_nodes_with_execution_record"
      || key === "patch_application_breadth"
      || key === "shared_snapshot_replacement_count"
      || key === "snapshot_batch_size"
      || key === "structural_replace_batch_commit_count"
      || key === "gc_epoch_count"
      || key === "graph_storage_compaction_count"
      || key === "graph_storage_snapshot_rewrites"
      || key === "graph_storage_subscriber_segments_rewritten"
      // Rewritten dependency segments are counted only by storage compaction, like
      // the compaction count and subscriber segments above: when compaction ran
      // depends on the run history, not on the graph it holds.
      || key === "graph_storage_dependency_segments_rewritten"
      // Executor usage and plans built count execution runs, not graph truth: a
      // worker-first root runs the executor and planner for every cached artifact it
      // refreshes, a compatibility runtime runs them per direct read. Same exclusion
      // as the performance summary. Stages built and tasks scheduled still compare.
      || key === "serial_executor_usage_count"
      || key === "parallel_executor_usage_count"
      || key === "plans_built"
    ) {
      continue;
    }
    comparable[key] = comparableGraphSummary(value);
  }
  return comparable;
}

export function comparableHistorySurfaceSummary(summary) {
  if (Array.isArray(summary)) {
    return summary.map(comparableHistorySurfaceSummary);
  }
  if (!summary || typeof summary !== "object") {
    return summary;
  }
  const comparable = {};
  for (const [key, value] of Object.entries(summary)) {
    if (key === "execution_record_count" || key === "latest_execution_record_id") {
      continue;
    }
    comparable[key] = comparableHistorySurfaceSummary(value);
  }
  return comparable;
}

export function comparableRecentHistory(history) {
  return history.filter(
    (entry) => !(entry.execution_record_count === 0 && entry.latest_execution_record_id == null),
  );
}

export function comparableSnapshotEnvelope(envelope) {
  return {
    state: envelope.state,
  };
}

export function comparableSnapshotArtifact(snapshot) {
  return {
    state: snapshot.state,
  };
}

export function comparableRuntimeBranch(branch) {
  return {
    id: branch.id,
    name: branch.name,
    parent_branch_id: branch.parent_branch_id,
  };
}

export function comparableBranchStateProof(proof) {
  return {
    proofSchemaVersion: proof.proofSchemaVersion,
    branchId: proof.branchId,
    branchName: proof.branchName,
    stateDigest: proof.stateDigest,
  };
}

export function comparableReplayParityProof(proof) {
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

export function comparableMergePlan(plan) {
  return {
    source_branch_id: plan.source_branch_id,
    target_branch_id: plan.target_branch_id,
    merge_kind: plan.merge_kind,
    selected_semantics: plan.selected_semantics,
    counters: plan.counters,
  };
}

export function comparableMergeResult(result) {
  return {
    source_branch: result.source_branch,
    target_branch: result.target_branch,
    merge_kind: result.merge_kind,
    selected_semantics: result.selected_semantics,
    counters: result.counters,
  };
}

export function comparableMergeResultProof(envelope) {
  return {
    result: {
      source_branch: envelope.result.source_branch,
      target_branch: envelope.result.target_branch,
      selected_semantics: envelope.result.selected_semantics,
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
