import {
  canonicalDigest,
  canonicalId,
  deepFreeze,
  immutableClone,
  isPlainRecord,
} from "../support/canonical.js";
import { validateSchemaValue } from "../schema/schema_declaration.js";

const ISSUED_LOCAL_TRUTH_BASES = new WeakSet();

export function createInitialAuthorityState({
  authorityId,
  schema,
  initialEntities,
  acceptSerializedBases = false,
}) {
  if (typeof authorityId !== "string" || authorityId.trim() === "") {
    throw new TypeError("local truth authority id must be a non-empty string");
  }
  if (!isPlainRecord(initialEntities) || Object.keys(initialEntities).length === 0) {
    throw new TypeError("local truth authority requires at least one initial entity");
  }
  const values = Object.fromEntries(
    Object.keys(initialEntities)
      .sort()
      .map((entityId) => [
        requireEntityId(entityId),
        validateSchemaValue(schema, initialEntities[entityId], `initial entity ${entityId}`),
      ]),
  );
  const snapshot = createSnapshot(authorityId, schema.identity, values);
  const commitPayload = {
    artifactFamily: "LocalTruthCommit",
    authorityId,
    authorityKind: "typescriptInMemoryLocalTruth",
    schemaIdentity: schema.identity,
    branchId: "branch:main",
    parentCommitId: null,
    beforeSnapshotId: null,
    afterSnapshotId: snapshot.id,
    kind: "genesis",
    operations: [],
    lineageUpdates: [],
    decisions: [],
    counters: { committedLoci: 0, reconstructedEntities: Object.keys(values).length },
  };
  const commit = createCommit(commitPayload);
  const basis = createBasis({
    authorityId,
    schemaIdentity: schema.identity,
    branchId: "branch:main",
    headCommitId: commit.id,
    snapshotId: snapshot.id,
    revision: 0,
  });
  const branch = deepFreeze({
    artifactFamily: "LocalTruthBranchReceipt",
    id: "branch:main",
    name: "main",
    kind: "ordinary",
    parentBranchId: null,
    forkCommitId: commit.id,
    forkSnapshotId: snapshot.id,
    forkRevision: 0,
    headCommitId: commit.id,
    snapshotId: snapshot.id,
    retired: false,
    basis,
  });
  return {
    authorityId,
    authorityKind: "typescriptInMemoryLocalTruth",
    acceptSerializedBases,
    schemaIdentity: schema.identity,
    revision: 0,
    nextBranchSequence: 1,
    branches: new Map([[branch.id, branch]]),
    snapshots: new Map([[snapshot.id, snapshot]]),
    commits: new Map([[commit.id, commit]]),
    checkpoints: new Map(),
    requestLog: new Map(),
    locusHeadByBranch: new Map([[branch.id, createGenesisLocusHeads(schema, values, commit.id)]]),
    decisionLog: [],
    derivations: new Map(),
    counters: {
      commits: 1,
      branches: 1,
      projections: 0,
      rebuilds: 0,
      serializedBreadth: 0,
      roundTrips: 0,
      compactions: 0,
    },
  };
}

export function createSnapshot(authorityId, schemaIdentity, values) {
  const frozenValues = immutableClone(values);
  return deepFreeze({
    artifactFamily: "LocalTruthSnapshot",
    id: canonicalId("truth-snapshot", { authorityId, schemaIdentity, values: frozenValues }),
    authorityId,
    schemaIdentity,
    values: frozenValues,
  });
}

export function createCommit(payload) {
  const identityPayload = { ...payload };
  const integrityDigest = canonicalDigest(identityPayload);
  return deepFreeze({ ...payload, id: `truth-commit:${integrityDigest}`, integrityDigest });
}

export function createBasis(fields) {
  const identityDigest = canonicalDigest({ artifactFamily: "LocalTruthBasis", ...fields });
  const basis = deepFreeze({ artifactFamily: "LocalTruthBasis", ...fields, identityDigest });
  ISSUED_LOCAL_TRUTH_BASES.add(basis);
  return basis;
}

export function branchWithHead(branch, commit, snapshot, revision) {
  const basis = createBasis({
    authorityId: commit.authorityId,
    schemaIdentity: commit.schemaIdentity,
    branchId: branch.id,
    headCommitId: commit.id,
    snapshotId: snapshot.id,
    revision,
  });
  return deepFreeze({
    ...branch,
    headCommitId: commit.id,
    snapshotId: snapshot.id,
    basis,
  });
}

export function cloneAuthorityState(state) {
  return {
    ...state,
    branches: new Map(state.branches),
    snapshots: new Map(state.snapshots),
    commits: new Map(state.commits),
    checkpoints: new Map(state.checkpoints),
    requestLog: new Map(state.requestLog),
    locusHeadByBranch: cloneNestedMaps(state.locusHeadByBranch),
    decisionLog: state.decisionLog.slice(),
    derivations: new Map(state.derivations),
    counters: { ...state.counters },
  };
}

export function readBranchSnapshot(state, branchId) {
  const branch = requireActiveBranch(state, branchId);
  const snapshot = state.snapshots.get(branch.snapshotId);
  if (!snapshot) {
    throw new Error(`local truth snapshot ${branch.snapshotId} is unavailable`);
  }
  return { branch, snapshot };
}

export function requireActiveBranch(state, branchId) {
  const branch = state.branches.get(branchId);
  if (!branch || branch.retired) {
    throw new TypeError(`local truth branch ${String(branchId)} is unavailable`);
  }
  return branch;
}

export function requireCurrentBasis(state, branch, basis) {
  if (!basis || basis.artifactFamily !== "LocalTruthBasis") {
    return { ok: false, code: "localTruthBasisRequired" };
  }
  const fieldsMatch = basis.authorityId === state.authorityId
    && basis.schemaIdentity === state.schemaIdentity
    && basis.branchId === branch.id;
  if (!fieldsMatch) {
    return { ok: false, code: "foreignLocalTruthBasis" };
  }
  if (!ISSUED_LOCAL_TRUTH_BASES.has(basis) && !state.acceptSerializedBases) {
    return { ok: false, code: "forgedLocalTruthBasis" };
  }
  if (basis.identityDigest !== canonicalDigest({
    artifactFamily: "LocalTruthBasis",
    authorityId: basis.authorityId,
    schemaIdentity: basis.schemaIdentity,
    branchId: basis.branchId,
    headCommitId: basis.headCommitId,
    snapshotId: basis.snapshotId,
    revision: basis.revision,
  })) {
    return { ok: false, code: "forgedLocalTruthBasis" };
  }
  if (basis.headCommitId !== branch.headCommitId || basis.snapshotId !== branch.snapshotId) {
    return { ok: false, code: "staleLocalTruthBasis" };
  }
  return { ok: true };
}

export function locusKey(entityId, aspectId) {
  return `${entityId.length}:${entityId}${aspectId}`;
}

function createGenesisLocusHeads(schema, values, commitId) {
  const heads = new Map();
  for (const entityId of Object.keys(values)) {
    for (const aspect of schema.aspects) {
      heads.set(locusKey(entityId, aspect.id), commitId);
    }
  }
  return heads;
}

function cloneNestedMaps(source) {
  return new Map([...source].map(([key, value]) => [key, new Map(value)]));
}

function requireEntityId(value) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError("local truth entity ids must be non-empty strings");
  }
  return value;
}
