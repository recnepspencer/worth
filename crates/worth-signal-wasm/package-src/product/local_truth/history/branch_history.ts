import {
  cloneAuthorityState,
  createBasis,
  readBranchSnapshot,
  requireCurrentBasis,
} from "../authority/authority_state.js";
import { canonicalDigest, deepFreeze } from "../support/canonical.js";
import { denied, success } from "../support/outcomes.js";

export function forkLocalTruthBranch(state, request) {
  try {
    const { branch: parent } = readBranchSnapshot(state, request?.parentBranchId);
    const basisCheck = requireCurrentBasis(state, parent, request?.expectedParentBasis);
    if (!basisCheck.ok) {
      return { state, outcome: denied(basisCheck.code, "local truth fork basis is stale or foreign") };
    }
    const name = requireBranchName(request?.name);
    const branchId = `branch:${state.nextBranchSequence}:${canonicalDigest({
      authorityId: state.authorityId,
      parentBranchId: parent.id,
      forkCommitId: parent.headCommitId,
      forkSnapshotId: parent.snapshotId,
      forkRevision: state.revision,
      name,
    })}`;
    const basis = createBasis({
      authorityId: state.authorityId,
      schemaIdentity: state.schemaIdentity,
      branchId,
      headCommitId: parent.headCommitId,
      snapshotId: parent.snapshotId,
      revision: state.revision,
    });
    const branch = deepFreeze({
      artifactFamily: "LocalTruthBranchReceipt",
      id: branchId,
      name,
      kind: "ordinary",
      parentBranchId: parent.id,
      forkCommitId: parent.headCommitId,
      forkSnapshotId: parent.snapshotId,
      forkRevision: state.revision,
      headCommitId: parent.headCommitId,
      snapshotId: parent.snapshotId,
      retired: false,
      basis,
    });
    const next = cloneAuthorityState(state);
    next.nextBranchSequence += 1;
    next.branches.set(branch.id, branch);
    next.locusHeadByBranch.set(branch.id, new Map(state.locusHeadByBranch.get(parent.id)));
    next.counters.branches += 1;
    return { state: next, outcome: success(branch) };
  } catch (error) {
    return {
      state,
      outcome: denied("localTruthBranchForkDenied", error instanceof Error ? error.message : String(error)),
    };
  }
}

export function createLocalTruthCheckpoint(state, branchId) {
  const { branch, snapshot } = readBranchSnapshot(state, branchId);
  const priorCheckpoint = state.checkpoints.get(branch.id) ?? null;
  const lineage = [];
  const locusHeads = [...state.locusHeadByBranch.get(branch.id)]
    .sort(([left], [right]) => left.localeCompare(right));
  const compactedCommitDigests = collectCommitSegment(
    state,
    branch.headCommitId,
    priorCheckpoint?.headCommitId ?? null,
  )
    .map((commit) => commit.integrityDigest);
  const checkpoint = deepFreeze({
    artifactFamily: "LocalTruthCheckpoint",
    authorityId: state.authorityId,
    schemaIdentity: state.schemaIdentity,
    branchId: branch.id,
    branch,
    headCommitId: branch.headCommitId,
    snapshotId: snapshot.id,
    values: snapshot.values,
    lineage,
    locusHeads,
    priorCheckpointDigest: priorCheckpoint?.digest ?? null,
    compactedCommitCount: compactedCommitDigests.length,
    compactedCommitDigests,
    compactedSegmentDigest: canonicalDigest(compactedCommitDigests),
    digest: canonicalDigest({
      branchId: branch.id,
      branch,
      headCommitId: branch.headCommitId,
      snapshotId: snapshot.id,
      values: snapshot.values,
      lineage,
      locusHeads,
      priorCheckpointDigest: priorCheckpoint?.digest ?? null,
      compactedCommitDigests,
    }),
  });
  const next = cloneAuthorityState(state);
  next.checkpoints.set(branch.id, checkpoint);
  return { state: compactIfFullyCheckpointed(next), outcome: success(checkpoint) };
}

export function inspectLocalTruthState(state, runtimeCounters = null) {
  const branches = [...state.branches.values()]
    .map((branch) => deepFreeze({ ...branch }))
    .sort((left, right) => left.id.localeCompare(right.id));
  const heads = Object.fromEntries(branches.map((branch) => [branch.id, branch.basis]));
  const values = Object.fromEntries(branches.map((branch) => [
    branch.id,
    state.snapshots.get(branch.snapshotId).values,
  ]));
  return deepFreeze({
    artifactFamily: "LocalTruthInspection",
    authorityId: state.authorityId,
    authorityKind: state.authorityKind,
    schemaIdentity: state.schemaIdentity,
    supportPosture: "inMemoryProcessLocal",
    revision: state.revision,
    branches,
    heads,
    values,
    decisionLog: state.decisionLog,
    counters: { ...state.counters, ...(runtimeCounters ?? {}) },
    digest: canonicalDigest({ revision: state.revision, branches, heads, values, decisionLog: state.decisionLog }),
  });
}

export function branchHistorySegment(state, branchId) {
  const branch = state.branches.get(branchId);
  if (!branch) {
    return denied("unknownLocalTruthBranch", `branch ${String(branchId)} is unavailable`);
  }
  const checkpoint = state.checkpoints.get(branchId) ?? null;
  const commits = [];
  let commitId = branch.headCommitId;
  while (commitId && commitId !== checkpoint?.headCommitId) {
    const commit = state.commits.get(commitId);
    if (!commit) {
      return denied("corruptLocalTruthHistory", `commit ${commitId} is unavailable`);
    }
    commits.push(commit);
    commitId = commit.parentCommitId;
  }
  commits.reverse();
  return success(deepFreeze({
    artifactFamily: "LocalTruthHistorySegment",
    branchId,
    checkpoint,
    fromCommitId: checkpoint?.headCommitId ?? commits[0]?.id ?? null,
    toCommitId: commits.at(-1)?.id ?? null,
    commits,
    digest: canonicalDigest({
      checkpoint: checkpoint?.digest ?? null,
      commits: commits.map((commit) => commit.integrityDigest),
    }),
  }));
}

function compactIfFullyCheckpointed(state) {
  const activeBranches = [...state.branches.values()].filter((branch) => !branch.retired);
  if (!activeBranches.every((branch) => (
    state.checkpoints.get(branch.id)?.headCommitId === branch.headCommitId
  ))) {
    return state;
  }
  const retainedSnapshotIds = new Set(activeBranches.flatMap((branch) => [
    branch.snapshotId,
    branch.forkSnapshotId,
  ]));
  const retainedCommits = new Map();
  const retainedSnapshots = new Map(
    [...state.snapshots].filter(([id]) => retainedSnapshotIds.has(id)),
  );
  if (
    retainedCommits.size === state.commits.size
    && retainedSnapshots.size === state.snapshots.size
  ) {
    return state;
  }
  const next = cloneAuthorityState(state);
  const activeBranchIds = new Set(activeBranches.map((branch) => branch.id));
  next.branches = new Map([...state.branches].filter(([id]) => activeBranchIds.has(id)));
  next.checkpoints = new Map([...state.checkpoints].filter(([id]) => activeBranchIds.has(id)));
  next.commits = retainedCommits;
  next.snapshots = retainedSnapshots;
  next.locusHeadByBranch = new Map(
    [...next.locusHeadByBranch].filter(([id]) => activeBranchIds.has(id)),
  );
  const activeHeadIds = new Set(activeBranches.map((branch) => branch.headCommitId));
  next.requestLog = new Map(
    [...state.requestLog].filter(([, entry]) => activeHeadIds.has(entry.commit.id)),
  );
  next.counters.compactions += 1;
  return next;
}

function collectCommitSegment(state, headCommitId, stopCommitId) {
  const commits = [];
  let commitId = headCommitId;
  while (commitId && commitId !== stopCommitId) {
    const commit = state.commits.get(commitId);
    if (!commit) break;
    commits.push(commit);
    commitId = commit.parentCommitId;
  }
  commits.reverse();
  return commits;
}

function requireBranchName(value) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError("local truth branch name must be a non-empty string");
  }
  return value;
}
