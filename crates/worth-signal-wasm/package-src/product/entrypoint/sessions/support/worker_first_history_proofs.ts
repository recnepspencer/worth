import { freezeObject } from "../../../graph_support.js";

export function normalizeWorkerFirstBranchId(branchId, operation) {
  if (typeof branchId === "bigint") {
    if (branchId < 0n) {
      throw new RangeError(`${operation} expects a non-negative branch id`);
    }
    return branchId;
  }
  if (!Number.isSafeInteger(branchId) || branchId < 0) {
    throw new TypeError(`${operation} expects a non-negative safe integer branch id`);
  }
  return BigInt(branchId);
}

export function createWorkerFirstSnapshotArtifact(snapshotArtifact) {
  return freezeObject({
    ...snapshotArtifact.snapshot,
    snapshotRestoreToken: snapshotArtifact.snapshotRestoreToken,
    snapshotPortableWire: snapshotArtifact.snapshotPortableWire,
    snapshotRestoreMode: "SameRuntimeExact",
  });
}

export function createWorkerFirstSnapshotEnvelopeArtifact(snapshotArtifact) {
  return freezeObject({
    ...snapshotArtifact.snapshotEnvelope,
    snapshotEnvelopeRestoreToken: snapshotArtifact.snapshotEnvelopeRestoreToken,
    snapshotEnvelopePortableWire: snapshotArtifact.snapshotEnvelopePortableWire,
    snapshotEnvelopeRestoreMode: "SameRuntimeExact",
  });
}

export function createReplayParityProofReport(
  proofSchemaVersion,
  expected,
  replayed,
) {
  const mismatchClasses = expected.stateDigest === replayed.stateDigest
    ? []
    : ["BranchStateDigestMismatch"];
  return freezeObject({
    proofSchemaVersion,
    expectedBranchId: expected.branchId,
    expectedBranchName: expected.branchName,
    expectedSnapshotId: expected.snapshotId,
    expectedStateDigest: expected.stateDigest,
    replayedBranchId: replayed.branchId,
    replayedBranchName: replayed.branchName,
    replayedSnapshotId: replayed.snapshotId,
    replayedStateDigest: replayed.stateDigest,
    parity: mismatchClasses.length === 0,
    mismatchClasses: freezeObject(mismatchClasses),
  });
}

export function createReplayArtifactProofReport(
  proofSchemaVersion,
  expected,
  replayed,
) {
  const frozenExpected = freezeArtifactProofInput(expected);
  const frozenReplayed = freezeArtifactProofInput(replayed);
  const mismatchClasses = [];
  if (frozenExpected.proofSchemaVersion !== proofSchemaVersion
    || frozenReplayed.proofSchemaVersion !== proofSchemaVersion) {
    mismatchClasses.push("ProofSchemaVersionMismatch");
  }
  compareOptionalDigest(
    frozenExpected.registryBundleDigest,
    frozenReplayed.registryBundleDigest,
    "MissingRegistryBundleDigest",
    "RegistryBundleDigestMismatch",
    mismatchClasses,
  );
  if (frozenExpected.branchStateDigest !== frozenReplayed.branchStateDigest) {
    mismatchClasses.push("BranchStateDigestMismatch");
  }
  return freezeObject({
    proofSchemaVersion,
    expected: frozenExpected,
    replayed: frozenReplayed,
    parity: mismatchClasses.length === 0,
    mismatchClasses: freezeObject(mismatchClasses),
  });
}

function freezeArtifactProofInput(input) {
  const fields = ["proofSchemaVersion", "registryBundleDigest", "branchStateDigest"];
  if (Object.keys(input).some((field) => !fields.includes(field))) {
    throw new TypeError("replay artifact proof contains retired or unsupported fields");
  }
  return freezeObject({
    proofSchemaVersion: input.proofSchemaVersion,
    registryBundleDigest: input.registryBundleDigest ?? null,
    branchStateDigest: input.branchStateDigest,
  });
}

function compareOptionalDigest(expected, replayed, missingClass, mismatchClass, output) {
  if (expected == null && replayed == null) {
    return;
  }
  if (expected == null || replayed == null) {
    output.push(missingClass);
    return;
  }
  if (expected !== replayed) {
    output.push(mismatchClass);
  }
}
