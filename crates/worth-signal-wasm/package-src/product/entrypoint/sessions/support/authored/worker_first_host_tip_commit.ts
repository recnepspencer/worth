import { materializeWorkerCachedValue } from "../worker_cached_value.js";
import { projectAuthoredInputTip } from "./worker_first_authored_tip_projection.js";

/**
 * Single host-tip ingress seam: advance tips for writes, project dependent
 * tips, and notify observers before any worker mutation queue work.
 *
 * tipWrites: readonly { id: string, value: unknown }[]
 */
export function commitHostTipAndNotify(deps, tipWrites) {
  const writes = Array.isArray(tipWrites) ? tipWrites : [];
  const changedIds = [];
  const rollbacks = [];
  const epochById = new Map();

  for (const write of writes) {
    if (!write || typeof write.id !== "string") {
      continue;
    }
    const authoredRollback = projectAuthoredInputTip(
      deps.authoredInputs,
      write.id,
      write.value,
    );
    if (authoredRollback) {
      rollbacks.push(authoredRollback.rollback);
      changedIds.push(write.id);
      epochById.set(write.id, authoredRollback.epochAtWrite);
      continue;
    }
    const importRollback = projectImportContextTip(deps, write.id, write.value);
    if (importRollback) {
      rollbacks.push(importRollback.rollback);
      changedIds.push(write.id);
      epochById.set(write.id, importRollback.epochAtWrite);
    }
  }

  const projectedReadableIds = notifyHostTipIds(deps, changedIds);
  return Object.freeze({
    changedIds: Object.freeze([...changedIds]),
    projectedReadableIds: Object.freeze([...projectedReadableIds]),
    epochById: Object.freeze(new Map(epochById)),
    rollback() {
      let restoredAny = false;
      for (let index = rollbacks.length - 1; index >= 0; index -= 1) {
        if (rollbacks[index]()) {
          restoredAny = true;
        }
      }
      if (restoredAny) {
        notifyHostTipIds(deps, changedIds);
      }
    },
  });
}

/** Tips already advanced; refresh dependents and notify once via signal ids. */
export function notifyHostTipIds(deps, changedIds) {
  const ids = Array.isArray(changedIds) ? changedIds : [];
  const projectedReadableIds = typeof deps.refreshAuthoredCallbackTipsFromHost === "function"
    ? deps.refreshAuthoredCallbackTipsFromHost(ids)
    : [];
  const notifyIds = [...ids, ...projectedReadableIds];
  if (
    notifyIds.length > 0
    && typeof deps.observations.deliverSignalIds === "function"
  ) {
    deps.observations.deliverSignalIds(notifyIds);
  }
  return projectedReadableIds;
}

export function tipWritesFromTransactionOps(transactionOps) {
  if (!Array.isArray(transactionOps)) {
    return [];
  }
  const writes = [];
  for (const op of transactionOps) {
    if (!op || typeof op.id !== "string") {
      continue;
    }
    // Patch fragments must be merged to complete set values before tip ingress.
    if (op.kind === "set" || op.kind === "setWithRegions") {
      writes.push({ id: op.id, value: op.value });
    }
  }
  return writes;
}

/** Stamp apply ops with the tip epochs just written so older confirms cannot clobber. */
export function stampTransactionOpsWithTipEpochs(transactionOps, epochById) {
  if (!Array.isArray(transactionOps) || !(epochById instanceof Map)) {
    return;
  }
  for (const op of transactionOps) {
    if (!op || typeof op.id !== "string" || !epochById.has(op.id)) {
      continue;
    }
    op.epochAtWrite = epochById.get(op.id);
  }
}

/**
 * Capture host tip values/epochs before import context rebuild, then restore any
 * tip that this completing apply does not own onto the replacement context.
 */
export function preserveImportTipsAcrossContextReplace(previousContext, nextContext, transactionOps) {
  if (!previousContext?.signalValueById || !nextContext?.signalValueById) {
    return;
  }
  const previousEpochs = importHostTipEpochs.get(previousContext);
  if (!(previousEpochs instanceof Map) || previousEpochs.size === 0) {
    return;
  }
  const applyEpochById = new Map();
  if (Array.isArray(transactionOps)) {
    for (const op of transactionOps) {
      if (op && typeof op.id === "string" && typeof op.epochAtWrite === "number") {
        applyEpochById.set(op.id, op.epochAtWrite);
      }
    }
  }
  const nextEpochs = ensureImportHostTipEpochs(nextContext);
  const previousWorkerEpochs = importWorkerTipEpochs.get(previousContext) ?? new Map();
  const nextWorkerEpochs = new Map();
  importWorkerTipEpochs.set(nextContext, nextWorkerEpochs);
  for (const [id, epoch] of previousEpochs) {
    if (!previousContext.signalValueById.has(id) || !nextContext.signalValueById.has(id)) {
      continue;
    }
    nextEpochs.set(id, epoch);
    const owningApplyEpoch = applyEpochById.get(id);
    nextWorkerEpochs.set(id, Math.max(previousWorkerEpochs.get(id) ?? 0, owningApplyEpoch ?? 0));
    if (owningApplyEpoch === epoch) {
      continue;
    }
    nextContext.signalValueById.set(id, previousContext.signalValueById.get(id));
  }
}

// Import contexts are frozen — keep tip epochs off-DOM in a WeakMap.
const importHostTipEpochs = new WeakMap();
const importWorkerTipEpochs = new WeakMap();

export function pendingImportedHostTipIds(context) {
  const hostEpochs = importHostTipEpochs.get(context);
  const workerEpochs = importWorkerTipEpochs.get(context);
  return [...(hostEpochs ?? [])]
    .filter(([id, epoch]) => epoch > (workerEpochs?.get(id) ?? 0))
    .map(([id]) => id);
}

function projectImportContextTip(deps, id, value) {
  const context = typeof deps.activeImportContext === "function"
    ? deps.activeImportContext()
    : null;
  if (!context?.signalValueById?.has(id)) {
    return null;
  }
  const epochs = ensureImportHostTipEpochs(context);
  const previousValue = context.signalValueById.get(id);
  const previousEpoch = epochs.get(id) ?? 0;
  const epochAtWrite = previousEpoch + 1;
  epochs.set(id, epochAtWrite);
  context.signalValueById.set(id, materializeWorkerCachedValue(value));
  return Object.freeze({
    epochAtWrite,
    rollback() {
      if ((epochs.get(id) ?? 0) !== epochAtWrite) {
        return false;
      }
      context.signalValueById.set(id, previousValue);
      epochs.set(id, previousEpoch);
      return true;
    },
  });
}

function ensureImportHostTipEpochs(context) {
  let epochs = importHostTipEpochs.get(context);
  if (!(epochs instanceof Map)) {
    epochs = new Map();
    importHostTipEpochs.set(context, epochs);
  }
  return epochs;
}
