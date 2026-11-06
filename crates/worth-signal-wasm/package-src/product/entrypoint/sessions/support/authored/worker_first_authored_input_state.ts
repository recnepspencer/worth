import { materializeWorkerCachedValue } from "../worker_cached_value.js";

export function nextGeneratedStandaloneSignalId(counters, family, scopeId = null) {
  const counterKey = scopeId ?? "__root__";
  const next = (counters.get(counterKey) ?? 0) + 1;
  counters.set(counterKey, next);
  if (typeof scopeId === "string" && scopeId.length > 0) {
    return `__WorthSignalScoped.${scopeId}.${family}.${next}`;
  }
  return `__WorthSignal.${family}.${next}`;
}

export function createAuthoredInputPublication(id, initial, options = {}) {
  // Authored inputs publish as sources only. A previous input-echo recipe
  // (`${id}.__workerFirstInputEcho` with reads:[id] => DEFAULT_ASPECT) was an
  // internal DAG subscriber that inflated aspect fan-out diagnostics and only
  // tracked aspect 0. Host readback uses the source id directly.
  return {
    policy: { preset: "operational" },
    sources: [
      {
        id,
        initial,
        ...(options.producesAspects === undefined
          ? {}
          : { producesAspects: options.producesAspects }),
      },
    ],
    recipes: [],
    outputIds: [],
  };
}

export function createWorkerFirstAuthoredInputState(
  initial,
  publicationState = "ready",
  publicationOptions = {},
) {
  return {
    baselineValue: materializeWorkerCachedValue(initial),
    currentValue: materializeWorkerCachedValue(initial),
    invalidatedMessage: null,
    publicationState,
    // Carried so tip re-admit republishes the same admission contract.
    publicationOptions: cloneAuthoredInputPublicationOptions(publicationOptions),
    // Stamped when the input is known present on the active worker tip catalog.
    admittedTipEpoch: 0,
    // Host tip ownership epoch — rollback only when this write still owns the tip.
    hostTipEpoch: 0,
    workerTipEpoch: 0,
  };
}

function cloneAuthoredInputPublicationOptions(options) {
  if (!options || typeof options !== "object") {
    return {};
  }
  return options.producesAspects === undefined
    ? {}
    : { producesAspects: options.producesAspects };
}

export function invalidateWorkerFirstAuthoredInputs(authoredInputs, message) {
  for (const authoredInput of authoredInputs.values()) {
    authoredInput.invalidatedMessage = message;
  }
}

export function hasMutableWorkerFirstAuthoredInputId(authoredInputs, id) {
  const authoredInput = authoredInputs.get(id);
  return authoredInput != null
    && authoredInput.invalidatedMessage === null
    && authoredInput.publicationState !== "failed";
}

export function isWorkerFirstAuthoredInputPublicationReady(authoredInputs, id) {
  const authoredInput = authoredInputs.get(id);
  return authoredInput != null
    && authoredInput.invalidatedMessage === null
    && authoredInput.publicationState === "ready";
}

export function readWorkerFirstAuthoredInputValue(authoredInputs, id) {
  const authoredInput = authoredInputs.get(id);
  if (!authoredInput) {
    return undefined;
  }
  if (authoredInput.invalidatedMessage !== null) {
    throw new TypeError(
      `worker-first authored input \`${id}\` cannot be used because ${authoredInput.invalidatedMessage}`,
    );
  }
  return authoredInput.currentValue;
}

export function applyCommittedWorkerFirstAuthoredInputs(authoredInputs, transactionOps) {
  for (const transactionOp of transactionOps) {
    if (!transactionOp || (transactionOp.kind !== "set" && transactionOp.kind !== "setWithRegions")
      || typeof transactionOp.id !== "string") {
      continue;
    }
    const authoredInput = authoredInputs.get(transactionOp.id);
    if (!authoredInput || authoredInput.invalidatedMessage !== null) {
      continue;
    }
    // Tip-epoch ownership: older applies must not clobber a newer tip.
    // Ops without epochAtWrite remain worker catch-up (readable refresh).
    if (typeof transactionOp.epochAtWrite === "number") {
      authoredInput.workerTipEpoch = Math.max(authoredInput.workerTipEpoch ?? 0, transactionOp.epochAtWrite);
    }
    if (
      typeof transactionOp.epochAtWrite === "number"
      && (authoredInput.hostTipEpoch ?? 0) !== transactionOp.epochAtWrite
    ) {
      continue;
    }
    authoredInput.currentValue = materializeWorkerCachedValue(transactionOp.value);
  }
}

export function readWorkerFirstAuthoredInputBaseline(authoredInputs, id) {
  const authoredInput = authoredInputs.get(id);
  if (!authoredInput || authoredInput.invalidatedMessage !== null) {
    throw new TypeError(
      `worker-first authored input \`${id}\` cannot read its baseline because it is not currently available`,
    );
  }
  return authoredInput.baselineValue;
}

export function writeWorkerFirstAuthoredInputBaseline(authoredInputs, id, value) {
  const authoredInput = authoredInputs.get(id);
  if (!authoredInput || authoredInput.invalidatedMessage !== null) {
    throw new TypeError(
      `worker-first authored input \`${id}\` cannot update its baseline because it is not currently available`,
    );
  }
  authoredInput.baselineValue = materializeWorkerCachedValue(value);
}

export function buildAuthoredInputMutationOperation(id, mutation, authoredInput) {
  if (!mutation || typeof mutation !== "object") {
    throw new TypeError("worker-first inputAsync mutation requires an operation object");
  }
  switch (mutation.kind) {
    case "set":
      return { kind: "set", id, value: mutation.value };
    case "reset":
      return { kind: "set", id, value: authoredInput.baselineValue };
    case "patch":
      return { kind: "set", id, value: mergeWorkerFirstPatchValue(authoredInput.currentValue, mutation.value) };
    default:
      throw new TypeError("worker-first inputAsync mutation kind is unsupported");
  }
}

export function mergeWorkerFirstPatchValue(currentValue, patchValue) {
  const currentIsObject = currentValue !== null && typeof currentValue === "object";
  const patchIsObject = patchValue !== null && typeof patchValue === "object";
  if (!currentIsObject || !patchIsObject) {
    throw new TypeError("worker-first input patch(...) requires object or array values");
  }
  if (Array.isArray(currentValue) || Array.isArray(patchValue)) {
    return patchValue;
  }
  return {
    ...currentValue,
    ...patchValue,
  };
}
