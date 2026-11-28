function readMutationResponseRollbackProof(effect) {
  const rollback = effect.optimistic.rollback;
  return Object.freeze({
    kind: rollback.kind,
    mode: "mode" in rollback ? rollback.mode : null,
    branchId: "branchId" in rollback ? rollback.branchId : null,
    snapshotId: "snapshotId" in rollback ? rollback.snapshotId : null,
    inverseKind:
      "inverse" in rollback && rollback.inverse !== null
        ? rollback.inverse.kind
        : null,
    detail: rollback.detail,
  });
}

function readMutationResponseMergeRebaseProof(effect) {
  return Object.freeze({
    kind: "unavailable",
    granularity: readMergeRebaseGranularity(effect),
    locusKind: effect.locus.kind,
    locusProofDigest: effect.locusProof?.effectLocusDigest ?? null,
    detail: "mutation response target has no native merge/rebase profile",
  });
}

function readMergeRebaseGranularity(effect) {
  switch (effect.locus.kind) {
    case "detailField":
      return `field:${effect.locus.field}`;
    case "detailRegion":
      return `region:${effect.locus.region}:${readRegionMergeGranularity(effect)}`;
    case "detailJsonPath":
      return `jsonPath:${effect.locus.path}`;
    case "membership":
    case "entityStore":
    case "connection":
    case "discriminatedTuple":
    case "groupedCollection":
    case "mapCollection":
    case "namedCollection":
    case "recursiveTree":
    case "sparsePage":
      return `item:${effect.locus.itemId}`;
    case "summary":
      return `summary:${effect.locus.summary}`;
    default:
      return effect.locus.kind;
  }
}

function readRegionMergeGranularity(effect) {
  const mergeGranularity = effect.patch.region?.mergeGranularity;
  if (typeof mergeGranularity !== "string" || mergeGranularity.length === 0) {
    throw new TypeError(
      "mutation response lifecycle proof requires detail region mergeGranularity",
    );
  }
  return mergeGranularity;
}

function readIdentityMigrationGranularity(target) {
  if (target.execution.kind === "exactDetailChildRegion") {
    return `detailChildRegion:${target.scope.region}`;
  }
  return [
    "identityMigration",
    target.execution.previousCanonicalKey,
    target.execution.nextCanonicalKey,
  ].join(":");
}

function createTargetEffectProofDigest(effect, artifact, rollback, mergeRebase) {
  return [
    artifact.targetId,
    effect.effectId,
    rollback.kind,
    rollback.mode ?? "none",
    mergeRebase.kind,
    mergeRebase.granularity,
    effect.authority.envelopeDigest,
  ].join("|");
}

function createIdentityMigrationLifecycleDigest(
  target,
  authorityDigest,
  rollbackKind,
  mergeKind,
) {
  return [
    "identityMigration",
    target.targetId,
    rollbackKind,
    mergeKind,
    readIdentityMigrationGranularity(target),
    authorityDigest,
  ].join("|");
}

function createRollbackDigest(entries) {
  if (entries.length === 0) {
    return "mutation-response-rollback|none";
  }
  return `mutation-response-rollback|${entries.map((entry) =>
    [
      entry.entryKind,
      entry.targetId,
      entry.effectId ?? "none",
      entry.rollback.kind,
      entry.rollback.mode ?? "none",
      entry.rollback.branchId ?? "none",
      entry.rollback.snapshotId ?? "none",
      entry.rollback.inverseKind ?? "none",
    ].join(":")).join(",")}`;
}

function createMergeRebaseDigest(entries) {
  if (entries.length === 0) {
    return "mutation-response-merge-rebase|none";
  }
  return `mutation-response-merge-rebase|${entries.map((entry) =>
    [
      entry.entryKind,
      entry.targetId,
      entry.effectId ?? "none",
      entry.mergeRebase.kind,
      entry.mergeRebase.granularity,
      entry.mergeRebase.locusKind ?? "none",
      entry.mergeRebase.locusProofDigest ?? "none",
    ].join(":")).join(",")}`;
}

export {
  createIdentityMigrationLifecycleDigest,
  createMergeRebaseDigest,
  createRollbackDigest,
  createTargetEffectProofDigest,
  readIdentityMigrationGranularity,
  readMutationResponseMergeRebaseProof,
  readMutationResponseRollbackProof,
};
