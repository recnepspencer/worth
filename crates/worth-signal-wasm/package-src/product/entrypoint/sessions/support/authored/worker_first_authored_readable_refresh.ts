export async function refreshWorkerFirstAuthoredReadables(deps) {
  const finish = deps.changedHostDependencyIds?.length > 0 ? deps.hostRefreshes?.begin() : null;
  try {
    await refreshWorkerFirstAuthoredCallbackReadables(deps);
    await refreshWorkerFirstAuthoredReadableSignals(deps);
  } finally {
    finish?.();
  }
}

import {
  applyCommittedWorkerFirstAuthoredInputs,
} from "./worker_first_authored_input_state.js";
import {
  updateWorkerFirstAuthoredReadables,
} from "./worker_first_authored_readable_state.js";
import { hostDependenciesIntersect } from "./worker_first_host_dependency_records.js";
import { materializeWorkerCachedValue } from "../worker_cached_value.js";
import { pendingWorkerReadableIds } from "./worker_first_authored_readback_currentness.js";

export async function refreshWorkerFirstAuthoredCallbackReadables(deps) {
  const {
    bridge,
    authoredInputs,
    authoredReadables,
    authoredCallbacks,
    changedIds,
    changedHostDependencyIds = [],
    captureCallback,
    skipDirectReadableRefresh = false,
    awaitPublication = null,
  } = deps;
  const refreshAll = !Array.isArray(changedIds) || !Array.isArray(changedHostDependencyIds);
  if (
    !refreshAll
    && changedIds.length === 0
    && changedHostDependencyIds.length === 0
  ) {
    return;
  }
  const changedIdSet = refreshAll ? new Set() : new Set(changedIds);
  const changedHostDependencyIdSet = refreshAll ? new Set() : new Set(changedHostDependencyIds);
  const directlyChangedReadableIds = [];
  for (const [id, authoredReadable] of authoredReadables) {
    if (authoredReadable.invalidatedMessage !== null) {
      continue;
    }
    if (
      refreshAll
      || authoredReadable.dependencyIds.some((dependencyId) => changedIdSet.has(dependencyId))
    ) {
      directlyChangedReadableIds.push(id);
      changedIdSet.add(id);
    }
  }
  if (!skipDirectReadableRefresh && directlyChangedReadableIds.length > 0) {
    await refreshWorkerFirstAuthoredReadableIds({
      bridge,
      authoredReadables,
      readableIds: directlyChangedReadableIds,
      invalidationMessage: "worker-first authored readable no longer exists after worker runtime history mutation",
      awaitPublication,
      pendingInputIds: deps.pendingInputIds,
    });
  }
  const hiddenTransactionOps = [];
  for (const [id, authoredCallback] of authoredCallbacks) {
    const authoredReadable = authoredReadables.get(id);
    if (!authoredReadable || authoredReadable.invalidatedMessage !== null) {
      continue;
    }
    if (
      !refreshAll
      && !authoredCallback.dependencyIds.some((dependencyId) => changedIdSet.has(dependencyId))
      && !hostDependenciesIntersect(
        authoredCallback.hostDependencyIds,
        changedHostDependencyIdSet,
      )
    ) {
      continue;
    }
    const capture = captureCallback(authoredCallback.callback, authoredCallback.family);
    authoredCallback.dependencyIds = capture.reads;
    authoredCallback.hostDependencyIds = capture.hostDependencyIds;
    authoredCallback.hostDependencies = capture.hostDependencies;
    const nextValue = materializeWorkerCachedValue(capture.value);
    authoredReadable.currentValue = nextValue;
    authoredReadable.dependencyIds = [...capture.reads];
    authoredReadable.hostDependencyIds = [...capture.hostDependencyIds];
    authoredReadable.hostDependencies = [...capture.hostDependencies];
    // Only re-commit the hidden backing input when the recomputed value actually
    // changed. A redundant set to the identical value is not a semantic no-op at
    // the branch layer: it re-journals the backing input's dependents on the
    // active branch, which makes a freshly forked branch look "advanced" and
    // downgrades otherwise fast-forward merges. Skipping the equal-value write
    // keeps worker-first branch/merge parity with mainThreadCompatibility.
    const committedBackingValue = authoredInputs.get(authoredCallback.hiddenInputId)?.currentValue;
    if (workerFirstAuthoredValueUnchanged(committedBackingValue, nextValue)) {
      continue;
    }
    hiddenTransactionOps.push({
      kind: "set",
      id: authoredCallback.hiddenInputId,
      value: nextValue,
    });
    changedIdSet.add(id);
    changedIdSet.add(authoredCallback.hiddenInputId);
  }
  if (hiddenTransactionOps.length === 0) {
    return;
  }
  await bridge.applyTransaction(hiddenTransactionOps);
  applyCommittedWorkerFirstAuthoredInputs(authoredInputs, hiddenTransactionOps);
}

export async function refreshWorkerFirstAuthoredReadableSignals(deps) {
  const activeReadableIds = [...deps.authoredReadables.entries()]
    .filter(([, authoredReadable]) => authoredReadable.invalidatedMessage === null)
    .map(([id]) => id);
  if (activeReadableIds.length === 0) {
    return;
  }
  await refreshWorkerFirstAuthoredReadableIds({
    bridge: deps.bridge,
    authoredReadables: deps.authoredReadables,
    readableIds: activeReadableIds,
    invalidationMessage: "worker-first authored readable no longer exists after worker runtime mutation",
    awaitPublication: deps.awaitPublication ?? null,
    pendingInputIds: deps.pendingInputIds,
  });
}

async function refreshWorkerFirstAuthoredReadableIds(deps) {
  const readableIds = [];
  for (const readableId of deps.readableIds) {
    const authoredReadable = deps.authoredReadables.get(readableId);
    if (!authoredReadable || authoredReadable.invalidatedMessage !== null) {
      continue;
    }
    if (authoredReadable.publicationState === "pending") {
      if (typeof deps.awaitPublication === "function") {
        try {
          await deps.awaitPublication(readableId);
        } catch {
          // Publication failure already invalidated via tracker onFailure path.
          continue;
        }
      } else {
        // Still publishing — unknown-id must not sticky-kill this readable.
        continue;
      }
    }
    const refreshed = deps.authoredReadables.get(readableId);
    if (!refreshed || refreshed.invalidatedMessage !== null) {
      continue;
    }
    if (refreshed.publicationState === "pending") {
      continue;
    }
    readableIds.push(readableId);
  }
  const pending = pendingWorkerReadableIds(deps);
  const currentIds = readableIds.filter(id => !pending.has(id));
  if (currentIds.length === 0) {
    return;
  }
  // One transport round trip for the readable set, rather than one per node.
  // Preparation and adoption remain O(readable count + returned value size).
  try {
    const hostEpoch = deps.hostRefreshes?.epoch();
    const packet = await deps.bridge.readSignals({ signalIds: currentIds });
    adoptCurrentWorkerReadback(deps, packet.signals, hostEpoch);
  } catch (error) {
    if (!currentIds.some((id) => isUnknownSignalIdError(error, id))) {
      throw error;
    }
    // History can remove a subset of readables. Isolate missing identities only
    // in that recovery lane so live siblings still refresh and unrelated errors
    // retain their original failure behavior.
    await refreshSurvivingAuthoredReadables(deps, currentIds);
  }
}

async function refreshSurvivingAuthoredReadables(deps, readableIds) {
  const hostEpoch = deps.hostRefreshes?.epoch();
  const refreshedSignals = [];
  for (const id of readableIds) {
    try {
      const packet = await deps.bridge.readSignals({ signalIds: [id] });
      refreshedSignals.push(...packet.signals);
    } catch (error) {
      if (!isUnknownSignalIdError(error, id)) {
        throw error;
      }
      invalidateAuthoredReadable(deps.authoredReadables, id, deps.invalidationMessage);
    }
  }
  adoptCurrentWorkerReadback(deps, refreshedSignals, hostEpoch);
}

function adoptCurrentWorkerReadback(deps, signals, hostEpoch) {
  if (hostEpoch !== deps.hostRefreshes?.epoch()) return;
  // Writes can arrive while readback is in flight; check the dependency basis
  // again at adoption rather than promoting an older worker result to the tip.
  const pending = pendingWorkerReadableIds(deps);
  updateWorkerFirstAuthoredReadables(deps.authoredReadables,
    signals.filter(signal => !pending.has(signal.id)));
}

function invalidateAuthoredReadable(authoredReadables, id, message) {
  const authoredReadable = authoredReadables.get(id);
  if (authoredReadable) {
    authoredReadable.invalidatedMessage = message;
    if (authoredReadable.publicationState === "pending") {
      authoredReadable.publicationState = "failed";
    }
  }
}

function workerFirstAuthoredValueUnchanged(left, right) {
  if (left === undefined) {
    return false;
  }
  if (Object.is(left, right)) {
    return true;
  }
  if (typeof left !== typeof right || left === null || right === null) {
    return false;
  }
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right) || left.length !== right.length) {
      return false;
    }
    return left.every((entry, index) => workerFirstAuthoredValueUnchanged(entry, right[index]));
  }
  if (typeof left !== "object") {
    return false;
  }
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);
  if (leftKeys.length !== rightKeys.length) {
    return false;
  }
  return leftKeys.every(
    (key) =>
      Object.prototype.hasOwnProperty.call(right, key)
      && workerFirstAuthoredValueUnchanged(left[key], right[key]),
  );
}

function isUnknownSignalIdError(error, signalId) {
  const detail = error instanceof Error ? error.message : String(error);
  return detail.includes("unknown signal id") && detail.includes(signalId);
}
