import {
  cloneAuthorityState,
  createSnapshot,
  locusKey,
} from "../authority/authority_state.js";
import { canonicalDigest, immutableClone } from "../support/canonical.js";
import { success, unavailable } from "../support/outcomes.js";

export function rebuildLocalTruthDerivedIndexes(state, schema) {
  try {
    requireCompatibleSchema(state, schema);
    validateBranchCatalog(state);
    const lineageByBranch = new Map();
    const locusHeadByBranch = new Map();
    for (const branch of state.branches.values()) {
      const rebuilt = rebuildBranchIndexes(state, schema, branch);
      lineageByBranch.set(branch.id, rebuilt.lineage);
      locusHeadByBranch.set(branch.id, rebuilt.locusHeads);
    }
    const next = cloneAuthorityState(state);
    next.lineageByBranch = lineageByBranch;
    next.locusHeadByBranch = locusHeadByBranch;
    return success(next);
  } catch (error) {
    return unavailable(
      "localTruthIndexRecoveryUnavailable",
      error instanceof Error ? error.message : String(error),
    );
  }
}

function rebuildBranchIndexes(state, schema, branch) {
  const checkpoint = state.checkpoints.get(branch.id) ?? null;
  if (branch.retired && !checkpoint && !state.commits.has(branch.headCommitId)) {
    return { lineage: new Map(), locusHeads: new Map() };
  }
  let lineage = new Map();
  let locusHeads = new Map();
  if (checkpoint) {
    validateCheckpoint(state, checkpoint, branch);
    lineage = new Map(checkpoint.lineage.map(([key, value]) => [key, immutableClone(value)]));
    locusHeads = new Map(checkpoint.locusHeads);
  }
  const commits = collectRequiredSegment(state, branch.headCommitId, checkpoint?.headCommitId ?? null);
  for (const commit of commits) {
    validateCommit(state, commit);
    if (commit.kind === "genesis") {
      const snapshot = requireSnapshot(state, commit.afterSnapshotId);
      locusHeads = genesisLocusHeads(schema, snapshot.values, commit.id);
      continue;
    }
    for (const operation of commit.operations) {
      locusHeads.set(locusKey(operation.entityId, operation.aspectId), commit.id);
    }
  }
  if (!checkpoint && commits[0]?.kind !== "genesis") {
    if (branch.retired) return { lineage, locusHeads };
    throw new Error(`branch ${branch.id} has neither a checkpoint nor a complete genesis segment`);
  }
  requireSnapshot(state, branch.snapshotId);
  return { lineage, locusHeads };
}

function validateCheckpoint(state, checkpoint, branch) {
  if (
    checkpoint.authorityId !== state.authorityId
    || checkpoint.schemaIdentity !== state.schemaIdentity
    || checkpoint.branchId !== branch.id
    || checkpoint.branch.id !== branch.id
    || checkpoint.headCommitId !== checkpoint.branch.headCommitId
    || checkpoint.snapshotId !== checkpoint.branch.snapshotId
  ) {
    throw new Error(`checkpoint ${branch.id} is foreign or internally inconsistent`);
  }
  if (checkpoint.lineage.length !== 0) {
    throw new Error(`legacy merge checkpoint ${branch.id} is retired`);
  }
  if (checkpoint.compactedCommitCount !== checkpoint.compactedCommitDigests.length) {
    throw new Error(`checkpoint ${branch.id} has an invalid compacted commit count`);
  }
  if (checkpoint.compactedSegmentDigest !== canonicalDigest(checkpoint.compactedCommitDigests)) {
    throw new Error(`checkpoint ${branch.id} has a corrupt compacted segment digest`);
  }
  const digest = canonicalDigest({
    branchId: checkpoint.branchId,
    branch: checkpoint.branch,
    headCommitId: checkpoint.headCommitId,
    snapshotId: checkpoint.snapshotId,
    values: checkpoint.values,
    lineage: checkpoint.lineage,
    locusHeads: checkpoint.locusHeads,
    priorCheckpointDigest: checkpoint.priorCheckpointDigest,
    compactedCommitDigests: checkpoint.compactedCommitDigests,
  });
  if (checkpoint.digest !== digest) {
    throw new Error(`checkpoint ${branch.id} failed its integrity digest`);
  }
  const snapshot = createSnapshot(state.authorityId, state.schemaIdentity, checkpoint.values);
  if (snapshot.id !== checkpoint.snapshotId) {
    throw new Error(`checkpoint ${branch.id} values do not reproduce its snapshot identity`);
  }
}

function collectRequiredSegment(state, headCommitId, stopCommitId) {
  const commits = [];
  const seen = new Set();
  let commitId = headCommitId;
  while (commitId && commitId !== stopCommitId) {
    if (seen.has(commitId)) throw new Error(`commit ancestry is cyclic at ${commitId}`);
    seen.add(commitId);
    const commit = state.commits.get(commitId);
    if (!commit) throw new Error(`required commit ${commitId} is unavailable`);
    commits.push(commit);
    commitId = commit.parentCommitId;
  }
  if (stopCommitId && commitId !== stopCommitId) {
    throw new Error(`checkpoint head ${stopCommitId} is not an ancestor of ${headCommitId}`);
  }
  return commits.reverse();
}

function validateCommit(state, commit) {
  if (commit.kind === "merge" || (commit.lineageUpdates?.length ?? 0) !== 0) {
    throw new Error(`legacy merge commit ${commit.id} is retired`);
  }
  const { id, integrityDigest, ...payload } = commit;
  const digest = canonicalDigest(payload);
  if (
    commit.authorityId !== state.authorityId
    || commit.schemaIdentity !== state.schemaIdentity
    || integrityDigest !== digest
    || id !== `truth-commit:${digest}`
  ) {
    throw new Error(`commit ${id} failed canonical integrity validation`);
  }
}

function validateBranchCatalog(state) {
  for (const branch of state.branches.values()) {
    const basisDigest = canonicalDigest({
      artifactFamily: "LocalTruthBasis",
      authorityId: branch.basis.authorityId,
      schemaIdentity: branch.basis.schemaIdentity,
      branchId: branch.basis.branchId,
      headCommitId: branch.basis.headCommitId,
      snapshotId: branch.basis.snapshotId,
      revision: branch.basis.revision,
    });
    if (
      branch.basis.authorityId !== state.authorityId
      || branch.basis.schemaIdentity !== state.schemaIdentity
      || branch.basis.branchId !== branch.id
      || branch.basis.headCommitId !== branch.headCommitId
      || branch.basis.snapshotId !== branch.snapshotId
      || branch.basis.identityDigest !== basisDigest
    ) {
      throw new Error(`branch ${branch.id} has a foreign or corrupt head basis`);
    }
    const seen = new Set();
    let cursor = branch;
    while (cursor) {
      if (seen.has(cursor.id)) throw new Error(`branch ancestry is cyclic at ${cursor.id}`);
      seen.add(cursor.id);
      cursor = cursor.parentBranchId ? state.branches.get(cursor.parentBranchId) : null;
      if (branch.parentBranchId && !state.branches.has(branch.parentBranchId)) {
        throw new Error(`branch ${branch.id} references a missing parent`);
      }
    }
  }
}

function requireSnapshot(state, snapshotId) {
  const snapshot = state.snapshots.get(snapshotId);
  if (!snapshot) throw new Error(`required snapshot ${snapshotId} is unavailable`);
  return snapshot;
}

function genesisLocusHeads(schema, values, commitId) {
  return new Map(Object.keys(values).flatMap((entityId) => schema.aspects.map((aspect) => [
    locusKey(entityId, aspect.id),
    commitId,
  ])));
}

function requireCompatibleSchema(state, schema) {
  if (!schema || schema.identity !== state.schemaIdentity) {
    throw new Error("derived-index recovery requires the authority's declared schema");
  }
}
