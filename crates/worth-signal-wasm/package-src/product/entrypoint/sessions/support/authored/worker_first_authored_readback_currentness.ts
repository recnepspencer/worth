import { pendingImportedHostTipIds } from "./worker_first_host_tip_commit.js";

/** Input tips whose owning write has not yet been acknowledged by the worker. */
export function pendingWorkerInputIds(authoredInputs, importContext) {
  const pending = pendingImportedHostTipIds(importContext);
  for (const [id, state] of authoredInputs) {
    if (state.invalidatedMessage === null && (state.hostTipEpoch ?? 0) > (state.workerTipEpoch ?? 0)) {
      pending.push(id);
    }
  }
  return pending;
}

/**
 * A worker result cannot replace a projection that consumes newer input tips.
 * Propagate that basis through readable edges in O(vertices + edges).
 */
export function pendingWorkerReadableIds(deps) {
  const pending = new Set(deps.pendingInputIds());
  if (pending.size === 0) return pending;
  const consumers = new Map();
  for (const [id, state] of deps.authoredReadables) {
    if (state.invalidatedMessage !== null) continue;
    for (const dependency of state.dependencyIds) {
      let targets = consumers.get(dependency);
      if (!targets) consumers.set(dependency, targets = []);
      targets.push(id);
    }
  }
  const queue = [...pending];
  for (let index = 0; index < queue.length; index += 1) {
    for (const id of consumers.get(queue[index]) ?? []) {
      if (pending.has(id)) continue;
      pending.add(id);
      queue.push(id);
    }
  }
  return pending;
}
