import {
  lookupInputSignalHandle,
  notifyInputSignalWrite,
} from "../handles.js";
import { RAW_SIGNAL_HANDLE } from "../symbols.js";

/**
 * Main-thread compatibility host tip surface.
 *
 * Worker-first roots advance host tips immediately, notify observers once,
 * and queue exactly one worker batch (worker_first_host_tip_commit.ts).
 * Compatibility roots have no worker: the raw runtime applies the same writes
 * in one transaction and projects dependents synchronously inside it. The
 * surface shape is identical so line bindings, forms, and any other authored
 * publisher use one tip-batch path in every deployment instead of falling
 * back to N independent signal.set() calls.
 *
 * Epochs are tracked per runtime and per input id: each committed tip bumps
 * the id's epoch, rollback() restores only ids whose epoch is still the one
 * this commit produced (and bumps again), and
 * applyCompatibilityCommittedTipBatch rejects a batch stamped with an epoch
 * that is no longer current. That makes a stale batch a loud TypeError rather
 * than a silent overwrite of a newer tip.
 */

const TIP_EPOCHS_BY_RUNTIME = new WeakMap();

function tipEpochsFor(rawSignals) {
  let epochs = TIP_EPOCHS_BY_RUNTIME.get(rawSignals);
  if (!epochs) {
    epochs = new Map();
    TIP_EPOCHS_BY_RUNTIME.set(rawSignals, epochs);
  }
  return epochs;
}

function readTipEpoch(rawSignals, id) {
  return tipEpochsFor(rawSignals).get(id) ?? 0;
}

function bumpTipEpoch(rawSignals, id) {
  const epochs = tipEpochsFor(rawSignals);
  const next = (epochs.get(id) ?? 0) + 1;
  epochs.set(id, next);
  return next;
}

function requireRegisteredInput(rawSignals, id, operation) {
  const handle = lookupInputSignalHandle(rawSignals, id);
  if (!handle) {
    throw new TypeError(
      `${operation} requires input ids authored on this runtime; `
        + `${JSON.stringify(id)} is not a live input handle of this runtime`,
    );
  }
  return handle;
}

function requireTipWriteArray(tipWrites, operation) {
  if (!Array.isArray(tipWrites)) {
    throw new TypeError(`${operation} requires an array of { id, value } tip writes`);
  }
  return tipWrites;
}

/**
 * Same admission as worker-first: writes without a string id are dropped
 * before projection; repeated ids collapse to the last value in the batch.
 */
function collectTipWrites(rawSignals, tipWrites, operation) {
  const byId = new Map();
  for (const write of requireTipWriteArray(tipWrites, operation)) {
    if (!write || typeof write.id !== "string") {
      continue;
    }
    byId.set(write.id, write);
  }
  const entries = [];
  for (const [id, write] of byId) {
    const handle = requireRegisteredInput(rawSignals, id, operation);
    entries.push({
      id,
      handle,
      rawHandle: handle[RAW_SIGNAL_HANDLE],
      value: write.value,
      previousValue: undefined,
    });
  }
  return entries;
}

function writeEntriesInOneTransaction(rawSignals, entries, valueOf) {
  rawSignals.transaction((tx) => {
    for (const entry of entries) {
      tx.set(entry.rawHandle, valueOf(entry));
    }
  });
  const epochById = new Map();
  for (const entry of entries) {
    epochById.set(entry.id, bumpTipEpoch(rawSignals, entry.id));
    notifyInputSignalWrite(entry.handle, valueOf(entry));
  }
  return epochById;
}

const EMPTY_IDS = Object.freeze([]);

export function commitCompatibilityHostTip(rawSignals, tipWrites) {
  const entries = collectTipWrites(rawSignals, tipWrites, "commitHostTipAndNotify");
  if (entries.length === 0) {
    return Object.freeze({
      changedIds: EMPTY_IDS,
      projectedReadableIds: EMPTY_IDS,
      epochById: Object.freeze(new Map()),
      rollback() {},
    });
  }
  for (const entry of entries) {
    entry.previousValue = entry.handle.peek();
  }
  const epochById = writeEntriesInOneTransaction(
    rawSignals,
    entries,
    (entry) => entry.value,
  );
  let rolledBack = false;
  return Object.freeze({
    changedIds: Object.freeze(entries.map((entry) => entry.id)),
    // The raw runtime projects dependents inside the transaction above, so
    // there is no host-side notify pass and nothing to list here. Worker-first
    // reports the readable ids its host projection touched.
    projectedReadableIds: EMPTY_IDS,
    epochById: Object.freeze(new Map(epochById)),
    rollback() {
      if (rolledBack) {
        return;
      }
      rolledBack = true;
      const restorable = entries.filter(
        (entry) => readTipEpoch(rawSignals, entry.id) === epochById.get(entry.id),
      );
      if (restorable.length === 0) {
        return;
      }
      writeEntriesInOneTransaction(
        rawSignals,
        restorable,
        (entry) => entry.previousValue,
      );
    },
  });
}

export function applyCompatibilityCommittedTipBatch(rawSignals, tipWrites) {
  const writes = requireTipWriteArray(tipWrites, "applyCommittedTipWorkerBatch");
  if (writes.length === 0) {
    return Promise.resolve(null);
  }
  const appliedIds = [];
  for (const write of writes) {
    if (!write || typeof write.id !== "string") {
      throw new TypeError(
        "applyCommittedTipWorkerBatch requires { id, value } tip writes",
      );
    }
    requireRegisteredInput(rawSignals, write.id, "applyCommittedTipWorkerBatch");
    if (write.epochAtWrite !== undefined) {
      const current = readTipEpoch(rawSignals, write.id);
      if (write.epochAtWrite !== current) {
        throw new TypeError(
          `applyCommittedTipWorkerBatch received a stale batch for ${JSON.stringify(write.id)}: `
            + `stamped epoch ${write.epochAtWrite}, current epoch ${current}`,
        );
      }
    }
    appliedIds.push(write.id);
  }
  // Tips were already applied by commitHostTipAndNotify's transaction; the
  // compatibility runtime has no worker copy to bring up to date.
  return Promise.resolve(Object.freeze({ appliedIds: Object.freeze(appliedIds) }));
}

export function publishCompatibilityAuthoredTipProjection(rawSignals, changedIds) {
  if (!Array.isArray(changedIds)) {
    throw new TypeError("publishAuthoredTipProjection requires an array of input ids");
  }
  for (const id of changedIds) {
    if (typeof id !== "string") {
      throw new TypeError("publishAuthoredTipProjection requires string input ids");
    }
    requireRegisteredInput(rawSignals, id, "publishAuthoredTipProjection");
  }
  // Dependents of these ids were projected by the runtime when their tips were
  // written; a notify-only pass has nothing left to publish on this deployment.
}

export function readCompatibilityTipEpoch(rawSignals, id) {
  return readTipEpoch(rawSignals, id);
}
