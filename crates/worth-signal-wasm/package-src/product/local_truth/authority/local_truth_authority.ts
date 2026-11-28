import { createInitialAuthorityState } from "./authority_state.js";
import { admitLocalTruthMutation } from "../commit/mutation_pipeline.js";
import {
  branchHistorySegment,
  createLocalTruthCheckpoint,
  forkLocalTruthBranch,
  inspectLocalTruthState,
} from "../history/branch_history.js";
import { readHistoricalSnapshot } from "../history/historical_snapshot.js";
import { requireDeclaredSchema } from "../schema/schema_declaration.js";
import { deepFreeze } from "../support/canonical.js";
import { denied, success } from "../support/outcomes.js";

const LOCAL_TRUTH_AUTHORITIES = new WeakSet();

export function createLocalTruthAuthority(
  options,
  internal = {
    faultInjector: null,
    onCommitted: null,
    onInitialize: null,
    onBranchFork: null,
    projection: null,
    acceptSerializedBases: false,
  },
) {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("signals.localTruth(...) requires an options object");
  }
  const schema = requireDeclaredSchema(options.schema, "signals.localTruth");
  let state = createInitialAuthorityState({
    authorityId: options.authorityId,
    schema,
    initialEntities: options.initialEntities,
    acceptSerializedBases: internal.acceptSerializedBases === true,
  });
  let terminated = false;
  let initialization;
  const requireActive = () => {
    if (terminated) {
      throw new Error("local truth authority has terminated");
    }
  };
  const authority = {
    kind: "typescriptInMemoryLocalTruth",
    schema,
    async inspect() {
      requireActive();
      await initialization;
      return inspectLocalTruthState(state, internal.projection?.counters?.());
    },
    async branch(branchId = "branch:main") {
      requireActive();
      await initialization;
      const branch = state.branches.get(branchId);
      return branch && !branch.retired
        ? success(branch)
        : denied("unknownLocalTruthBranch", `branch ${String(branchId)} is unavailable`);
    },
    async commit(request) {
      requireActive();
      await initialization;
      const result = admitLocalTruthMutation(state, schema, request, internal.faultInjector);
      state = result.state;
      if (result.outcome.posture === "advisory") {
        return result.outcome;
      }
      return attachDerivation(result.outcome, state, internal.onCommitted);
    },
    async forkBranch(request) {
      requireActive();
      await initialization;
      const result = forkLocalTruthBranch(state, request);
      state = result.state;
      if (result.outcome.posture !== "success") {
        return result.outcome;
      }
      const parent = state.branches.get(result.outcome.value.parentBranchId);
      const derivation = await internal.onBranchFork?.(result.outcome.value, parent, state);
      return success(deepFreeze({ ...result.outcome.value, derivation: derivation ?? null }));
    },
    async checkpoint(branchId) {
      requireActive();
      await initialization;
      const result = createLocalTruthCheckpoint(state, branchId);
      state = result.state;
      return result.outcome;
    },
    async history(branchId) {
      requireActive();
      await initialization;
      return branchHistorySegment(state, branchId);
    },
    async historicalSnapshot(request) {
      requireActive();
      await initialization;
      return readHistoricalSnapshot(state, request);
    },
    async derivation(branchId = "branch:main") {
      requireActive();
      await initialization;
      return internal.projection?.posture(branchId) ?? deepFreeze({
        artifactFamily: "LocalTruthSignalProjectionReceipt",
        branchId,
        commitId: null,
        posture: "Unavailable",
        reason: "noSignalProjectionBinding",
      });
    },
    async destroyDerivation(branchId) {
      requireActive();
      await initialization;
      if (!internal.projection) {
        return this.derivation(branchId);
      }
      return internal.projection.destroy(branchId);
    },
    async rebuildDerivation(branchId) {
      requireActive();
      await initialization;
      const branch = state.branches.get(branchId);
      if (!branch || branch.retired) {
        return denied("unknownLocalTruthBranch", `branch ${String(branchId)} is unavailable`);
      }
      if (!internal.projection) {
        return this.derivation(branchId);
      }
      return internal.projection.rebuild(branch, state.snapshots.get(branch.snapshotId));
    },
    async terminate() {
      terminated = true;
      state = null;
    },
  };
  const facade = deepFreeze(authority);
  LOCAL_TRUTH_AUTHORITIES.add(facade);
  const main = state.branches.get("branch:main");
  initialization = Promise.resolve(internal.onInitialize?.(
    main,
    state.snapshots.get(main.snapshotId),
    state,
  ));
  return facade;
}

export function isLocalTruthAuthority(value) {
  return Boolean(value && LOCAL_TRUTH_AUTHORITIES.has(value));
}

async function attachDerivation(outcome, state, project) {
  if (outcome.posture !== "success" && outcome.posture !== "advisory") {
    return outcome;
  }
  const commit = outcome.posture === "success"
    ? outcome.value.commit ?? outcome.value
    : outcome.value.commit ?? outcome.value;
  if (!commit || commit.artifactFamily !== "LocalTruthCommit") {
    return outcome;
  }
  const snapshot = state.snapshots.get(commit.afterSnapshotId);
  const derivation = await project?.(commit, snapshot, state) ?? null;
  return success(deepFreeze({
    artifactFamily: "LocalTruthCommitOutcome",
    commit,
    merge: outcome.value.commit ? outcome.value : null,
    derivation,
  }));
}
