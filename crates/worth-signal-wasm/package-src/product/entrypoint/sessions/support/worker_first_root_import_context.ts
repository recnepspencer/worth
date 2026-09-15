import {
  normalizeWorkerRuntimeDefinitionEnvelope,
  normalizeWorkerRuntimeEnvelope,
} from "../../bridge/worker_runtime_envelope_normalization.js";
import {
  createWorkerFirstSnapshotArtifact,
  createWorkerFirstSnapshotEnvelopeArtifact,
} from "./worker_first_history_proofs.js";

export async function buildActiveImportContext(bridge, definition, snapshot) {
  const inputIds = definition.inputDescriptors.map((descriptor) => descriptor.sourceId);
  const outputIds = definition.descriptors.map((descriptor) => descriptor.publishedId);
  const outputSourceIds = definition.descriptors.map((descriptor) => descriptor.sourceId);
  const signalIds = [...new Set([...inputIds, ...outputIds, ...outputSourceIds])];
  const branchesPromise = bridge.branches();
  const currentBranchPromise = bridge.currentBranch();
  // Exact runtime restore artifacts are root-branch evidence: the runtime refuses to mint
  // them while a child branch is active, so the context only asks for them on the root branch
  // and records the denial otherwise. Definitions have no branch preflight and stay cached.
  const exactRuntimeArtifactsPromise = currentBranchPromise.then((branch) => (
    branch.parent_branch_id === null
      ? Promise.all([
        bridge.exportWorkerRuntimeEnvelope(),
        bridge.exportWorkerRuntimeEnvelopeWire(),
        bridge.exportWorkerRuntimeEnvelopePortableWire(),
      ])
      : null
  ));
  const [
    runtimeDefinitionEnvelope,
    snapshotEnvelopeArtifact,
    exactRuntimeArtifacts,
    diagnosticsSummaryPacket,
    diagnosticsHistoryPacket,
    signalReadbackPacket,
    versionSummaries,
    health,
    currentBranch,
    branches,
    latestFlow,
    latestObservation,
    performanceSummary,
    latestFailure,
    latestRollback,
    latestInvalidationPlanningEstimate,
    latestInvalidationTraceRecords,
    recentHistory,
    runtimeProofReport,
    whys,
    replays,
    lineages,
    replaysByBranch,
    branchSnapshotArtifacts,
    branchSnapshotEnvelopes,
    branchStateProofs,
  ] = await Promise.all([
    bridge.exportDefinitions(),
    bridge.exportWorkerSnapshotEnvelopeArtifact(),
    exactRuntimeArtifactsPromise,
    bridge.readDiagnosticsSummary(),
    bridge.readDiagnosticsHistory(),
    bridge.readSignals({ signalIds }),
    bridge.readVersions(signalIds),
    bridge.health(),
    currentBranchPromise,
    branchesPromise,
    bridge.latestFlow(),
    bridge.latestObservation(),
    bridge.performanceSummary(),
    bridge.latestFailure(),
    bridge.latestRollback(),
    bridge.latestInvalidationPlanningEstimate(),
    bridge.latestInvalidationTraceRecords(),
    bridge.recentHistory(),
    bridge.runtimeProofReport(),
    Promise.all(signalIds.map(async (id) => [id, await bridge.why(id)])),
    Promise.all(signalIds.map(async (id) => [id, await bridge.replayFor(id)])),
    Promise.all(signalIds.map(async (id) => [id, await bridge.lineageFor(id)])),
    branchesPromise.then((workerBranches) => Promise.all(
      workerBranches.map(async (branch) => [branch.id, await bridge.replayForBranch(branch.id)]),
    )),
    branchesPromise.then((workerBranches) => Promise.all(
      workerBranches.map(async (branch) => [branch.id, await bridge.branchSnapshotArtifact(branch.id)]),
    )),
    branchesPromise.then((workerBranches) => Promise.all(
      workerBranches.map(async (branch) => [
        branch.id,
        await bridge.branchSnapshotEnvelopeArtifact(branch.id),
      ]),
    )),
    branchesPromise.then((workerBranches) => Promise.all(
      workerBranches.map(async (branch) => [branch.id, await bridge.branchStateProof(branch.id)]),
    )),
  ]);
  const definitions = normalizeWorkerRuntimeDefinitionEnvelope(runtimeDefinitionEnvelope);
  const runtimeEnvelopeArtifact = exactRuntimeArtifacts === null
    ? null
    : Object.freeze({
      ...normalizeWorkerRuntimeEnvelope(exactRuntimeArtifacts[0]),
      runtimeEnvelopeRestoreToken: exactRuntimeArtifacts[1],
      runtimeEnvelopeRestoreMode: "SameRuntimeExact",
      runtimeEnvelopePortableWire: exactRuntimeArtifacts[2],
    });
  return Object.freeze({
    definition,
    snapshot,
    inputDescriptorBySourceId: new Map(
      definition.inputDescriptors.map((descriptor) => [descriptor.sourceId, descriptor]),
    ),
    outputDescriptorBySourceId: new Map(
      definition.descriptors.map((descriptor) => [descriptor.sourceId, descriptor]),
    ),
    sourceDefinitionById: new Map(
      (definitions.sources ?? []).map((source) => [source.id, source]),
    ),
    recipeDefinitionById: new Map(
      (definitions.recipes ?? []).map((recipe) => [recipe.id, recipe]),
    ),
    runtimeDefinitionEnvelope: definitions,
    runtimeEnvelopeArtifact,
    requireRuntimeEnvelopeArtifact() {
      if (runtimeEnvelopeArtifact === null) {
        throw workerFirstRootBranchExactExportDenial(currentBranch);
      }
      return runtimeEnvelopeArtifact;
    },
    snapshotEnvelope: createWorkerFirstSnapshotEnvelopeArtifact(snapshotEnvelopeArtifact),
    diagnosticsSummary: diagnosticsSummaryPacket.summary,
    diagnosticsHistory: diagnosticsHistoryPacket.history,
    signalValueById: new Map(
      signalReadbackPacket.signals.map((signal) => [signal.id, signal.value]),
    ),
    publishedOutputIds: new Set(outputIds),
    versionById: new Map(versionSummaries.map((summary) => [summary.id, summary])),
    health,
    currentBranch,
    branches,
    latestFlow,
    latestObservation,
    performanceSummary,
    latestFailure,
    latestRollback,
    latestInvalidationPlanningEstimate,
    latestInvalidationTraceRecords,
    recentHistory,
    runtimeProofReport,
    whyById: new Map(whys),
    replayById: new Map(replays),
    lineageById: new Map(lineages),
    replayByBranchId: workerFirstBranchMap(replaysByBranch),
    branchSnapshotArtifactByBranchId: workerFirstBranchMap(
      branchSnapshotArtifacts.map(([branchId, artifact]) => [
        branchId,
        createWorkerFirstSnapshotArtifact(artifact),
      ]),
    ),
    branchSnapshotIdByBranchId: workerFirstBranchMap(
      branchSnapshotEnvelopes.map(([branchId, artifact]) => [
        branchId,
        artifact.snapshotEnvelope.snapshot.meta.snapshot_id,
      ]),
    ),
    branchSnapshotEnvelopeByBranchId: workerFirstBranchMap(
      branchSnapshotEnvelopes.map(([branchId, artifact]) => [
        branchId,
        createWorkerFirstSnapshotEnvelopeArtifact(artifact),
      ]),
    ),
    branchStateProofByBranchId: workerFirstBranchMap(branchStateProofs),
  });
}

// Mirrors the denial the worker runtime raises for exact runtime envelope exports on a child
// branch, so callers see the same error whether the artifact is cached or minted on demand.
function workerFirstRootBranchExactExportDenial(currentBranch) {
  const error = new Error(
    "exact runtime restore artifacts require the root branch to be active"
    + ` (branch ${currentBranch.id} "${currentBranch.name}" is active; switch to its root branch first)`,
  );
  error.name = "WorkerRuntimeBridgeError";
  error.code = "invalidInput";
  return error;
}

function workerFirstBranchMap(entries) {
  return new Map(entries.map(([branchId, value]) => [BigInt(branchId), value]));
}
