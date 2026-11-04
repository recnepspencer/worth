import {
  normalizeWorkerRuntimeDefinitionEnvelope,
  normalizeWorkerRuntimeEnvelope,
} from "../../bridge/worker_runtime_envelope_normalization.js";
import {
  createWorkerFirstSnapshotArtifact,
  createWorkerFirstSnapshotEnvelopeArtifact,
} from "./worker_first_history_proofs.js";

export async function buildActiveImportContext(bridge, definition, snapshot, committedState = null) {
  const inputIds = definition.inputDescriptors.map((descriptor) => descriptor.sourceId);
  const outputIds = definition.descriptors.map((descriptor) => descriptor.publishedId);
  const outputSourceIds = definition.descriptors.map((descriptor) => descriptor.sourceId);
  const signalIds = [...new Set([...inputIds, ...outputIds, ...outputSourceIds])];
  const {
    runtimeDefinitionEnvelope, snapshotEnvelopeArtifact, exactRuntimeArtifacts,
    signalReadbackPacket, versionSummaries, health, currentBranch, branches,
    latestFlow, latestObservation, performanceSummary, latestFailure, latestRollback,
    latestInvalidationPlanningEstimate, latestInvalidationTraceRecords, recentHistory,
    runtimeProofReport, whys, replays, lineages, replaysByBranch, branchSnapshotArtifacts,
    branchSnapshotEnvelopes, branchStateProofs, diagnosticsSummary, diagnosticsHistory,
  } = committedState ?? await bridge.readImportState({ signalIds });
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
    diagnosticsSummary,
    diagnosticsHistory,
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
