import { hostDependenciesIntersect } from "./worker_first_host_dependency_records.js";
import { materializeWorkerCachedValue } from "../worker_cached_value.js";
import { evaluateWorkerFirstDeclarativeTip } from "../../../worker_first_declarative_expr.js";

/**
 * Advance one authored input tip without enqueueing worker apply.
 * Returns a rollback handle, or null when the id is not an available authored input.
 * Rollback is conditional: only restores when this write still owns the tip epoch.
 */
export function projectAuthoredInputTip(authoredInputs, id, value) {
  const authoredInput = authoredInputs.get(id);
  if (
    !authoredInput
    || authoredInput.invalidatedMessage !== null
    || authoredInput.publicationState === "failed"
  ) {
    return null;
  }
  const previousValue = authoredInput.currentValue;
  const previousEpoch = authoredInput.hostTipEpoch ?? 0;
  const epochAtWrite = previousEpoch + 1;
  authoredInput.hostTipEpoch = epochAtWrite;
  authoredInput.currentValue = materializeWorkerCachedValue(value);
  return Object.freeze({
    id,
    epochAtWrite,
    rollback() {
      if ((authoredInput.hostTipEpoch ?? 0) !== epochAtWrite) {
        return false;
      }
      authoredInput.currentValue = previousValue;
      authoredInput.hostTipEpoch = previousEpoch;
      return true;
    },
  });
}

/**
 * Host-side tip projection for authored declarative + callback readables.
 * Recomputes dependent tips from local input tips without waiting on
 * worker publication/apply.
 */
export function refreshAuthoredCallbackTipsFromHost(deps) {
  const {
    authoredReadables,
    authoredCallbacks,
    changedIds,
    changedHostDependencyIds = [],
    captureCallback,
    readSignalValue,
  } = deps;
  if (!Array.isArray(changedIds) || changedIds.length === 0) {
    if (!Array.isArray(changedHostDependencyIds) || changedHostDependencyIds.length === 0) {
      return [];
    }
  }

  const changedIdSet = new Set(changedIds ?? []);
  const changedHostDependencyIdSet = new Set(changedHostDependencyIds ?? []);
  const projectedReadableIds = [];

  // Expand through declarative readable dependency edges so callback tips that
  // depend on those readables also recompute in this host turn.
  let grew = true;
  while (grew) {
    grew = false;
    for (const [id, authoredReadable] of authoredReadables) {
      if (authoredReadable.invalidatedMessage !== null || changedIdSet.has(id)) {
        continue;
      }
      if (authoredReadable.dependencyIds.some((dependencyId) => changedIdSet.has(dependencyId))) {
        changedIdSet.add(id);
        grew = true;
      }
    }
  }

  // Recompute declarative tips before callbacks so chained
  // input → declarative → callback projections see fresh tip values.
  const declarativeDone = new Set();
  let declarativeProgress = true;
  while (declarativeProgress) {
    declarativeProgress = false;
    for (const [id, authoredReadable] of authoredReadables) {
      if (
        !changedIdSet.has(id)
        || declarativeDone.has(id)
        || authoredCallbacks.has(id)
        || authoredReadable.invalidatedMessage !== null
        || !authoredReadable.publicationSpec?.expr
      ) {
        continue;
      }
      const pendingDeclarativeDeps = authoredReadable.dependencyIds.some((dependencyId) => {
        const depReadable = authoredReadables.get(dependencyId);
        return changedIdSet.has(dependencyId)
          && depReadable?.publicationSpec?.expr
          && !authoredCallbacks.has(dependencyId)
          && !declarativeDone.has(dependencyId);
      });
      if (pendingDeclarativeDeps) {
        continue;
      }
      const value = evaluateWorkerFirstDeclarativeTip(
        authoredReadable.publicationSpec,
        readSignalValue,
        "tip.project.declarative",
      );
      authoredReadable.currentValue = materializeWorkerCachedValue(value);
      declarativeDone.add(id);
      projectedReadableIds.push(id);
      declarativeProgress = true;
    }
  }

  for (const [id, authoredCallback] of authoredCallbacks) {
    const authoredReadable = authoredReadables.get(id);
    if (!authoredReadable || authoredReadable.invalidatedMessage !== null) {
      continue;
    }
    if (
      !authoredCallback.dependencyIds.some((dependencyId) => changedIdSet.has(dependencyId))
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
    authoredReadable.currentValue = materializeWorkerCachedValue(capture.value);
    authoredReadable.dependencyIds = [...capture.reads];
    authoredReadable.hostDependencyIds = [...capture.hostDependencyIds];
    authoredReadable.hostDependencies = [...capture.hostDependencies];
    changedIdSet.add(id);
    projectedReadableIds.push(id);
  }

  return projectedReadableIds;
}
