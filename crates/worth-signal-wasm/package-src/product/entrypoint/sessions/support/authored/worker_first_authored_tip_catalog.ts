/**
 * Worker tip catalog admission for authored signals.
 *
 * Branch tip moves (fork/apply/switch/…) can drop signals that the host still
 * treats as ready. Re-admit is a tip-boundary recovery cost, not an ordinary
 * set()/apply tax: bump the epoch when the tip catalog changes, then probe and
 * republish only ids still stamped for a prior tip.
 */

import {
  createAuthoredInputPublication,
  isWorkerFirstAuthoredInputPublicationReady,
} from "./worker_first_authored_input_state.js";
import {
  createAuthoredReadablePublication,
  isWorkerFirstAuthoredReadablePublicationReady,
  updateWorkerFirstAuthoredReadables,
} from "./worker_first_authored_readable_state.js";

export function createAuthoredTipCatalogAdmission() {
  let tipCatalogEpoch = 0;

  return Object.freeze({
    currentEpoch() {
      return tipCatalogEpoch;
    },
    markActiveTipCatalogChanged() {
      tipCatalogEpoch += 1;
    },
    stampAdmitted(state) {
      if (state) {
        state.admittedTipEpoch = tipCatalogEpoch;
      }
    },
    /**
     * Stamp only when the active tip is still the tip the publish targeted.
     * Tip moves during in-flight publish leave the id stale for readmit/ensure.
     */
    stampAdmittedIfEpoch(state, publishEpoch) {
      if (state && publishEpoch === tipCatalogEpoch) {
        state.admittedTipEpoch = tipCatalogEpoch;
      }
    },
  });
}

export async function readmitStaleAuthoredOntoActiveTip(deps) {
  await readmitStaleAuthoredInputsOntoActiveTip(deps);
  await readmitStaleAuthoredReadablesOntoActiveTip(deps);
}

export async function readmitStaleAuthoredInputsOntoActiveTip(deps) {
  const staleIds = [];
  for (const [id, authoredInput] of deps.authoredInputs) {
    if (!isWorkerFirstAuthoredInputPublicationReady(deps.authoredInputs, id)) {
      continue;
    }
    if ((authoredInput.admittedTipEpoch ?? -1) >= deps.tipCatalog.currentEpoch()) {
      continue;
    }
    staleIds.push(id);
  }
  for (const id of staleIds) {
    await ensureAuthoredInputPresentOnWorkerTip(deps, id);
  }
}

async function readmitStaleAuthoredReadablesOntoActiveTip(deps) {
  const staleIds = [];
  for (const [id, authoredReadable] of deps.authoredReadables) {
    if (!isWorkerFirstAuthoredReadablePublicationReady(deps.authoredReadables, id)) {
      continue;
    }
    if ((authoredReadable.admittedTipEpoch ?? -1) >= deps.tipCatalog.currentEpoch()) {
      continue;
    }
    if (!authoredReadable.publicationSpec) {
      continue;
    }
    staleIds.push(id);
  }
  for (const id of staleIds) {
    await ensureAuthoredReadablePresentOnWorkerTip(deps, id);
  }
}

export async function ensureAuthoredInputsPresentOnWorkerTip(deps, ids) {
  const uniqueIds = [...new Set((ids ?? []).filter((id) => typeof id === "string"))];
  for (const id of uniqueIds) {
    await ensureAuthoredInputPresentOnWorkerTip(deps, id);
  }
}

async function ensureAuthoredInputPresentOnWorkerTip(deps, id) {
  if (!isWorkerFirstAuthoredInputPublicationReady(deps.authoredInputs, id)) {
    return;
  }
  const authoredInput = deps.authoredInputs.get(id);
  if ((authoredInput.admittedTipEpoch ?? -1) >= deps.tipCatalog.currentEpoch()) {
    return;
  }
  try {
    await deps.bridge.readSignals({ signalIds: [id] });
    deps.tipCatalog.stampAdmitted(authoredInput);
    return;
  } catch (error) {
    if (!isUnknownSignalIdError(error, id)) {
      throw error;
    }
  }
  await deps.bridge.publishPortableGraph(
    createAuthoredInputPublication(
      id,
      authoredInput.currentValue,
      authoredInput.publicationOptions ?? {},
    ),
  );
  deps.tipCatalog.stampAdmitted(authoredInput);
}

export function tipBranchId(branch) {
  if (!branch || branch.id == null) {
    return null;
  }
  return String(branch.id);
}

/** Restore can rewrite tip catalog contents without changing branch id. */
export function historyOperationMayDropAuthoredCatalog(operation) {
  return typeof operation === "string"
    && (operation.includes("restore") || operation.includes("merge"));
}

async function ensureAuthoredReadablePresentOnWorkerTip(deps, id) {
  if (!isWorkerFirstAuthoredReadablePublicationReady(deps.authoredReadables, id)) {
    return;
  }
  const authoredReadable = deps.authoredReadables.get(id);
  if ((authoredReadable.admittedTipEpoch ?? -1) >= deps.tipCatalog.currentEpoch()) {
    return;
  }
  if (!authoredReadable.publicationSpec) {
    return;
  }
  try {
    const signalPacket = await deps.bridge.readSignals({ signalIds: [id] });
    updateWorkerFirstAuthoredReadables(deps.authoredReadables, signalPacket.signals);
    deps.tipCatalog.stampAdmitted(authoredReadable);
    return;
  } catch (error) {
    if (!isUnknownSignalIdError(error, id)) {
      throw error;
    }
  }
  await deps.bridge.publishPortableGraph(
    createAuthoredReadablePublication(id, authoredReadable.family, authoredReadable.publicationSpec),
  );
  const signalPacket = await deps.bridge.readSignals({ signalIds: [id] });
  updateWorkerFirstAuthoredReadables(deps.authoredReadables, signalPacket.signals);
  deps.tipCatalog.stampAdmitted(authoredReadable);
}

function isUnknownSignalIdError(error, signalId) {
  const detail = error instanceof Error ? error.message : String(error);
  return detail.includes("unknown signal id") && detail.includes(signalId);
}
