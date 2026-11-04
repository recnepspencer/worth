/** Capture one committed worker turn without yielding between component reads. */
function readWorkerDiagnostics(runtime, committedSummary = null) {
  return {
    health: runtime.health(),
    latestFlow: runtime.latestFlow(),
    latestObservation: runtime.latestObservation(),
    performanceSummary: runtime.performanceSummary(),
    latestFailure: runtime.latestFailure(),
    latestRollback: runtime.latestRollback(),
    latestInvalidationPlanningEstimate: runtime.latestInvalidationPlanningEstimate(),
    latestInvalidationTraceRecords: runtime.latestInvalidationTraceRecords(),
    recentHistory: runtime.recentHistory(),
    diagnosticsSummary: committedSummary?.diagnosticsSummary ?? runtime.readDiagnosticsSummary().summary,
    diagnosticsHistory: committedSummary?.diagnosticsHistory ?? runtime.readDiagnosticsHistory().history,
  };
}

export function applyWorkerAcknowledgedTransaction(runtime, request) {
  const projection = request.importOutputIds?.length
    ? runtime.applyTransactionProjection({ transactionOps: request.transactionOps, outputIds: request.importOutputIds })
    : null;
  const transaction = projection?.transaction ?? runtime.applyTransaction(request.transactionOps);
  const committedSummary = projection === null ? null : {
    diagnosticsSummary: projection.diagnosticsSummary.summary,
    diagnosticsHistory: projection.diagnosticsHistory.history,
  };
  return readWorkerAcknowledgementState(runtime, request, { transaction, committedSummary });
}

export function readWorkerAcknowledgementState(runtime, request, committed = null) {
  const signalIds = [...new Set([...request.signalIds, ...(request.importSignalIds ?? [])])];
  const signalReadbackPacket = signalIds.length === 0
    ? { signals: [] }
    : runtime.readSignals({ signalIds });
  const currentBranch = runtime.currentBranch();
  const branches = runtime.branches();
  let deliveryPacket = null;
  try {
    if (request.deliverObservation) deliveryPacket = runtime.deliverLatestObservation();
  } catch (error) {
    if (!String(error?.message).includes("requires an active lifecycle subscription")) throw error;
  }
  const diagnostics = readWorkerDiagnostics(runtime, committed?.committedSummary);
  return {
    transaction: committed?.transaction ?? null, signalReadbackPacket, currentBranch, branches, deliveryPacket, diagnostics,
    importState: request.importSignalIds === null ? null : readWorkerImportState(runtime, {
      signalIds: request.importSignalIds,
    }, { currentBranch, branches, diagnostics, signalReadbackPacket }),
  };
}

/** Synchronous public inspection/export facades require current retained artifacts. */
export function readWorkerImportState(runtime, { signalIds }, committed = null) {
  const committedSummary = committed?.diagnostics ?? {
    diagnosticsSummary: runtime.readDiagnosticsSummary().summary,
    diagnosticsHistory: runtime.readDiagnosticsHistory().history,
  };
  const currentBranch = committed?.currentBranch ?? runtime.currentBranch();
  const branches = committed?.branches ?? runtime.branches();
  const branchIds = branches.map(branch => BigInt(branch.id));
  const importIds = new Set(signalIds);
  const entries = (ids, method) => ids.map(id => [id, runtime[method](id)]);
  return {
    runtimeDefinitionEnvelope: runtime.exportDefinitions(),
    snapshotEnvelopeArtifact: runtime.exportWorkerSnapshotEnvelopeArtifact(),
    exactRuntimeArtifacts: currentBranch.parent_branch_id === null ? [
      runtime.exportWorkerRuntimeEnvelope(), runtime.exportWorkerRuntimeEnvelopeWire(),
      runtime.exportWorkerRuntimeEnvelopePortableWire(),
    ] : null,
    signalReadbackPacket: committed?.signalReadbackPacket
      ? { signals: committed.signalReadbackPacket.signals.filter(signal => importIds.has(signal.id)) }
      : runtime.readSignals({ signalIds }),
    versionSummaries: runtime.readVersions(signalIds),
    currentBranch, branches,
    ...(committed?.diagnostics ?? readWorkerDiagnostics(runtime, committedSummary)),
    runtimeProofReport: runtime.runtimeProofReport(),
    whys: entries(signalIds, "why"),
    replays: entries(signalIds, "replayFor"),
    lineages: entries(signalIds, "lineageFor"),
    replaysByBranch: entries(branchIds, "replayForBranch"),
    branchSnapshotArtifacts: entries(branchIds, "branchSnapshotArtifact"),
    branchSnapshotEnvelopes: entries(branchIds, "branchSnapshotEnvelopeArtifact"),
    branchStateProofs: entries(branchIds, "branchStateProof"),
  };
}
