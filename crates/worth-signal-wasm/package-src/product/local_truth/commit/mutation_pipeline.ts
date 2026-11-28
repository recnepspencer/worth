import {
  branchWithHead,
  cloneAuthorityState,
  createCommit,
  createSnapshot,
  locusKey,
  readBranchSnapshot,
  requireCurrentBasis,
} from "../authority/authority_state.js";
import {
  materializeAspect,
  requireAspect,
  validateAspectCandidate,
} from "../schema/schema_declaration.js";
import { canonicalDigest, deepFreeze, immutableClone, isPlainRecord } from "../support/canonical.js";
import { advisory, denied, failed, success } from "../support/outcomes.js";

const VALIDATED_MUTATIONS = new WeakSet();
const PLANNED_COMMITS = new WeakSet();
const STAGED_COMMITS = new WeakSet();

export function admitLocalTruthMutation(state, schema, rawRequest, faultInjector) {
  try {
    const duplicate = findDuplicateRequest(state, rawRequest);
    if (duplicate) {
      return { state, outcome: duplicate };
    }
    inject(faultInjector, "validation");
    const validated = validateMutation(state, schema, rawRequest);
    inject(faultInjector, "planning");
    const planned = planCommit(state, validated);
    inject(faultInjector, "reconstruction");
    const staged = stageCommit(state, schema, planned);
    inject(faultInjector, "digesting");
    verifyStagedCommit(staged);
    inject(faultInjector, "publication");
    const nextState = publishStagedCommit(state, staged);
    return { state: nextState, outcome: success(staged.commit) };
  } catch (error) {
    if (error?.localTruthDenial) {
      return { state, outcome: denied(error.code, error.message, error.evidence ?? null) };
    }
    return {
      state,
      outcome: failed("localTruthMutationFailed", error instanceof Error ? error.message : String(error)),
    };
  }
}

export function validateMutation(state, schema, rawRequest) {
  if (!isPlainRecord(rawRequest)) {
    throw new TypeError("local truth mutation request must be a plain object");
  }
  const { branch, snapshot } = readBranchSnapshot(state, rawRequest.branchId);
  const basisCheck = requireCurrentBasis(state, branch, rawRequest.expectedBasis);
  if (!basisCheck.ok) {
    throw denial(basisCheck.code, `local truth mutation basis was rejected: ${basisCheck.code}`);
  }
  const operations = normalizeOperations(schema, snapshot, rawRequest);
  const validated = deepFreeze({
    artifactFamily: "ValidatedLocalTruthMutation",
    authorityId: state.authorityId,
    schemaIdentity: schema.identity,
    branchId: branch.id,
    expectedBasis: branch.basis,
    requestId: requireRequestId(rawRequest.requestId),
    operations,
    metadata: normalizeMetadata(rawRequest.metadata),
  });
  VALIDATED_MUTATIONS.add(validated);
  return validated;
}

export function planCommit(state, validated) {
  if (!VALIDATED_MUTATIONS.has(validated)) {
    throw new TypeError("commit planning requires an admitted validated mutation");
  }
  const plan = deepFreeze({
    artifactFamily: "PlannedLocalTruthCommit",
    authorityId: validated.authorityId,
    schemaIdentity: validated.schemaIdentity,
    branchId: validated.branchId,
    expectedBasis: validated.expectedBasis,
    requestId: validated.requestId,
    operations: validated.operations,
    metadata: validated.metadata,
    parentCommitId: state.branches.get(validated.branchId).headCommitId,
    beforeSnapshotId: state.branches.get(validated.branchId).snapshotId,
    counters: {
      validatedLoci: validated.operations.length,
      reconstructedEntities: new Set(validated.operations.map((operation) => operation.entityId)).size,
    },
  });
  PLANNED_COMMITS.add(plan);
  return plan;
}

export function stageCommit(state, schema, plan) {
  if (!PLANNED_COMMITS.has(plan)) {
    throw new TypeError("commit staging requires a sealed commit plan");
  }
  const before = state.snapshots.get(plan.beforeSnapshotId);
  const values = { ...before.values };
  for (const operation of plan.operations) {
    values[operation.entityId] = materializeAspect(
      schema,
      values[operation.entityId],
      operation.aspectId,
      operation.after,
    );
  }
  const snapshot = createSnapshot(state.authorityId, schema.identity, values);
  const commit = createCommit({
    artifactFamily: "LocalTruthCommit",
    authorityId: state.authorityId,
    authorityKind: state.authorityKind,
    schemaIdentity: schema.identity,
    branchId: plan.branchId,
    parentCommitId: plan.parentCommitId,
    beforeSnapshotId: plan.beforeSnapshotId,
    afterSnapshotId: snapshot.id,
    kind: "mutation",
    requestId: plan.requestId,
    operations: plan.operations,
    lineageUpdates: [],
    decisions: [],
    metadata: plan.metadata,
    counters: { committedLoci: plan.operations.length, ...plan.counters },
  });
  const staged = deepFreeze({
    artifactFamily: "StagedLocalTruthCommit",
    authorityId: state.authorityId,
    plan,
    snapshot,
    commit,
  });
  STAGED_COMMITS.add(staged);
  return staged;
}

export function publishStagedCommit(state, staged) {
  if (!STAGED_COMMITS.has(staged)) {
    throw new TypeError("commit publication requires authority-staged work");
  }
  const branch = state.branches.get(staged.plan.branchId);
  const basisCheck = requireCurrentBasis(state, branch, staged.plan.expectedBasis);
  if (!basisCheck.ok) {
    throw denial(basisCheck.code, "local truth branch advanced before commit publication");
  }
  const next = cloneAuthorityState(state);
  const revision = state.revision + 1;
  next.revision = revision;
  next.snapshots.set(staged.snapshot.id, staged.snapshot);
  next.commits.set(staged.commit.id, staged.commit);
  next.branches.set(branch.id, branchWithHead(branch, staged.commit, staged.snapshot, revision));
  const locusHeads = next.locusHeadByBranch.get(branch.id);
  for (const operation of staged.commit.operations) {
    locusHeads.set(locusKey(operation.entityId, operation.aspectId), staged.commit.id);
  }
  next.counters.commits += 1;
  next.requestLog.set(staged.plan.requestId, deepFreeze({
    requestDigest: canonicalDigest({
      branchId: staged.plan.branchId,
      operations: staged.plan.operations.map(({ entityId, aspectId, after }) => ({ entityId, aspectId, value: after })),
      metadata: staged.plan.metadata,
    }),
    commit: staged.commit,
  }));
  return next;
}

function findDuplicateRequest(state, rawRequest) {
  if (!rawRequest || typeof rawRequest.requestId !== "string") {
    return null;
  }
  const prior = state.requestLog.get(rawRequest.requestId);
  if (!prior) {
    return null;
  }
  const requestDigest = canonicalDigest({
    branchId: rawRequest.branchId,
    operations: canonicalRequestOperations(rawRequest.operations),
    metadata: rawRequest.metadata ?? null,
  });
  if (requestDigest !== prior.requestDigest) {
    return denied(
      "requestIdentityReuse",
      `requestId ${rawRequest.requestId} was already used for different local truth work`,
    );
  }
  return advisory(prior.commit, [deepFreeze({ code: "duplicateRequest", message: "existing commit returned" })]);
}

function canonicalRequestOperations(operations) {
  if (!Array.isArray(operations)) return operations;
  return operations
    .map((operation) => ({
      entityId: operation?.entityId,
      aspectId: operation?.aspectId,
      value: operation?.value,
    }))
    .sort(compareOperations);
}

function normalizeOperations(schema, snapshot, rawRequest) {
  if (!Array.isArray(rawRequest.operations) || rawRequest.operations.length === 0) {
    throw new TypeError("local truth mutation requires at least one aspect operation");
  }
  const seen = new Set();
  const operations = rawRequest.operations.map((rawOperation, index) => {
    if (!isPlainRecord(rawOperation)) {
      throw new TypeError(`local truth operation ${index} must be a plain object`);
    }
    const entityId = rawOperation.entityId;
    if (typeof entityId !== "string" || !Object.hasOwn(snapshot.values, entityId)) {
      throw denial("unsupportedEntityIdentity", `entity ${String(entityId)} is not present in the branch snapshot`);
    }
    const aspect = requireAspect(schema, rawOperation.aspectId);
    const key = locusKey(entityId, aspect.id);
    if (seen.has(key)) {
      throw denial("duplicateMutationLocus", `mutation contains duplicate locus ${entityId}/${aspect.id}`);
    }
    seen.add(key);
    const before = snapshot.values[entityId][aspect.field];
    return deepFreeze({
      entityId,
      aspectId: aspect.id,
      before: immutableClone(before),
      after: validateAspectCandidate(
        schema,
        aspect.id,
        rawOperation.value,
        `local truth operation ${index}`,
      ),
      evidenceDigest: canonicalDigest({ entityId, aspectId: aspect.id, before, after: rawOperation.value }),
    });
  });
  return deepFreeze(operations.sort(compareOperations));
}

function compareOperations(left, right) {
  return left.entityId.localeCompare(right.entityId) || left.aspectId.localeCompare(right.aspectId);
}

function verifyStagedCommit(staged) {
  if (staged.commit.afterSnapshotId !== staged.snapshot.id) {
    throw new Error("staged local truth commit does not bind its reconstructed snapshot");
  }
}

function normalizeMetadata(metadata) {
  if (metadata === undefined) {
    return null;
  }
  if (!isPlainRecord(metadata)) {
    throw new TypeError("local truth mutation metadata must be a plain object");
  }
  return immutableClone(metadata);
}

function requireRequestId(value) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError("local truth mutation requestId must be a non-empty string");
  }
  return value;
}

function denial(code, message, evidence = null) {
  return Object.assign(new Error(message), {
    localTruthDenial: true,
    code,
    evidence,
  });
}

function inject(faultInjector, point) {
  faultInjector?.(point);
}
