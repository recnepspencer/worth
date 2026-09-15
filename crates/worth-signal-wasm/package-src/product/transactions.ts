import {
  activeComputedCallbackFrame,
  denySignalMutationDuringCallbackAuthoring,
} from "./callback_frames.js";
import { buildHostCapabilityTransportReport } from "./host_capability_reports.js";
import {
  notifyInputSignalWrite,
  requireProductSignalHandle,
  unwrapInputSignalTarget,
} from "./handles.js";
import { RAW_SIGNAL_HANDLE } from "./symbols.js";

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isPatchableSignalValue(value) {
  return isPlainObject(value) || Array.isArray(value);
}

function mergePatchValue(currentValue, patchValue, operation) {
  if (!isPatchableSignalValue(currentValue) || !isPatchableSignalValue(patchValue)) {
    throw new TypeError(`${operation} requires object or array values`);
  }
  if (Array.isArray(currentValue) || Array.isArray(patchValue)) {
    return patchValue;
  }
  return {
    ...currentValue,
    ...patchValue,
  };
}

function attachSerializedField(value, field, serialized) {
  if (!value || typeof value !== "object" || typeof serialized !== "string") {
    return value;
  }
  Object.defineProperty(value, field, {
    value: serialized,
    enumerable: true,
    configurable: false,
    writable: false,
  });
  return value;
}

function attachLiteralField(value, field, literal) {
  if (!value || typeof value !== "object") {
    return value;
  }
  Object.defineProperty(value, field, {
    value: literal,
    enumerable: true,
    configurable: false,
    writable: false,
  });
  return value;
}

function throwPortableImportUnavailableCallbacks(envelope, hostCapabilities) {
  const unavailableCallbacks = envelope?.definitions?.unavailableCallbacks;
  if (!Array.isArray(unavailableCallbacks) || unavailableCallbacks.length === 0) {
    return;
  }
  hostCapabilities.recordPortableImportDenial(unavailableCallbacks);
  const ids = unavailableCallbacks
    .map((artifact) => artifact?.id)
    .filter((id) => typeof id === "string" && id.length > 0)
    .join(", ");
  throw {
    code: "computeCallbackUnavailableForRuntimeEnvelopeImport",
    message: `runtime envelope import cannot restore callback-backed nodes without live callback registrations: ${ids}`,
  };
}

export function wrapTransaction(rawTx, rawSignals) {
  const stagedValues = new Map();

  function readCurrentValue(rawInput, handle) {
    return stagedValues.has(rawInput) ? stagedValues.get(rawInput) : handle.get();
  }

  function rememberStagedValue(rawInput, nextValue) {
    stagedValues.set(rawInput, nextValue);
  }

  return Object.freeze({
    set(input, value) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const rawInput = unwrapInputSignalTarget(input, rawSignals, "transaction.set");
      rawTx.set(rawInput, value);
      rememberStagedValue(rawInput, value);
      if (typeof input !== "string") {
        notifyInputSignalWrite(
          requireProductSignalHandle(input, rawSignals, "transaction.set"),
          value,
        );
      }
    },
    setWithAspects(input, value, aspects) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const rawInput = unwrapInputSignalTarget(input, rawSignals, "transaction.setWithAspects");
      rawTx.setWithAspects(rawInput, value, aspects);
      rememberStagedValue(rawInput, value);
      if (typeof input !== "string") {
        notifyInputSignalWrite(
          requireProductSignalHandle(input, rawSignals, "transaction.setWithAspects"),
          value,
        );
      }
    },
    setWithRegions(input, value, changedRegions) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const rawInput = unwrapInputSignalTarget(input, rawSignals, "transaction.setWithRegions");
      rawTx.setWithRegions(rawInput, value, changedRegions);
      rememberStagedValue(rawInput, value);
      if (typeof input !== "string") {
        notifyInputSignalWrite(
          requireProductSignalHandle(input, rawSignals, "transaction.setWithRegions"),
          value,
        );
      }
    },
    setWithRegionsAndAspects(input, value, changedRegions, aspects) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const rawInput = unwrapInputSignalTarget(
        input,
        rawSignals,
        "transaction.setWithRegionsAndAspects",
      );
      rawTx.setWithRegionsAndAspects(rawInput, value, changedRegions, aspects);
      rememberStagedValue(rawInput, value);
      if (typeof input !== "string") {
        notifyInputSignalWrite(
          requireProductSignalHandle(input, rawSignals, "transaction.setWithRegionsAndAspects"),
          value,
        );
      }
    },
    patch(input, patchValue) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const handle = requireProductSignalHandle(input, rawSignals, "transaction.patch");
      const rawInput = unwrapInputSignalTarget(input, rawSignals, "transaction.patch");
      const nextValue = mergePatchValue(
        readCurrentValue(rawInput, handle),
        patchValue,
        "transaction.patch(...)",
      );
      rawTx.set(rawInput, nextValue);
      rememberStagedValue(rawInput, nextValue);
      notifyInputSignalWrite(handle, nextValue);
    },
    free() {
      rawTx.free();
    },
    [Symbol.dispose]() {
      if (typeof rawTx[Symbol.dispose] === "function") {
        rawTx[Symbol.dispose]();
        return;
      }
      rawTx.free();
    },
  });
}

function requirePublishedGraphInput(inputOrName, publishedInputs, publishedInputHandleSet, rawSignals, graphId, operation) {
  if (typeof inputOrName === "string") {
    const publishedInput = publishedInputs[inputOrName];
    if (!publishedInput) {
      throw new TypeError(
        `${operation} only accepts published graph input names from graph \`${graphId}\`; \`${inputOrName}\` is not part of the public input contract`,
      );
    }
    return publishedInput[RAW_SIGNAL_HANDLE];
  }

  const rawInput = unwrapInputSignalTarget(inputOrName, rawSignals, operation);
  const signalId = typeof inputOrName?.id === "string" ? inputOrName.id : "<unknown>";
  if (!publishedInputHandleSet.has(inputOrName)) {
    throw new TypeError(
      `${operation} only accepts published graph inputs from graph \`${graphId}\`; \`${signalId}\` is outside the graph contract`,
    );
  }
  return rawInput;
}

function requireGraphInputWritable(authorities, graphId, inputName, operation) {
  const authority = authorities[inputName];
  if (!authority?.supportsWrite) {
    throw new TypeError(
      `${operation} cannot write public input \`${inputName}\` from graph \`${graphId}\` because its authority is \`${authority?.authority ?? "unknown"}\``,
    );
  }
}

function requireGraphInputPatchable(authorities, graphId, inputName, operation) {
  const authority = authorities[inputName];
  if (!authority?.supportsPatch) {
    throw new TypeError(
      `${operation} cannot patch public input \`${inputName}\` from graph \`${graphId}\` because its authority is \`${authority?.authority ?? "unknown"}\``,
    );
  }
}

export function wrapGraphTransaction(rawTx, rawSignals, graphId, publishedInputs, authorities) {
  const publishedInputHandleSet = new Set(Object.values(publishedInputs));
  const stagedValues = new Map();

  function readCurrentValue(rawInput, handle) {
    return stagedValues.has(rawInput) ? stagedValues.get(rawInput) : handle.get();
  }

  function rememberStagedValue(rawInput, nextValue) {
    stagedValues.set(rawInput, nextValue);
  }

  function resolve(inputOrName, operation, capability = "write") {
    const rawInput = requirePublishedGraphInput(
      inputOrName,
      publishedInputs,
      publishedInputHandleSet,
      rawSignals,
      graphId,
      operation,
    );
    const inputName = typeof inputOrName === "string"
      ? inputOrName
      : Object.entries(publishedInputs).find(([, handle]) => handle === inputOrName)?.[0];
    if (typeof inputName === "string") {
      if (capability === "patch") {
        requireGraphInputPatchable(authorities, graphId, inputName, operation);
      } else {
        requireGraphInputWritable(authorities, graphId, inputName, operation);
      }
    }
    const handle = typeof inputOrName === "string"
      ? publishedInputs[inputName]
      : requireProductSignalHandle(inputOrName, rawSignals, operation);
    return { rawInput, handle };
  }

  return Object.freeze({
    set(inputOrName, value) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const { rawInput, handle } = resolve(inputOrName, "graph.transaction.set");
      rawTx.set(rawInput, value);
      rememberStagedValue(rawInput, value);
      notifyInputSignalWrite(handle, value);
    },
    setWithAspects(inputOrName, value, aspects) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const { rawInput, handle } = resolve(inputOrName, "graph.transaction.setWithAspects");
      rawTx.setWithAspects(rawInput, value, aspects);
      rememberStagedValue(rawInput, value);
      notifyInputSignalWrite(handle, value);
    },
    setWithRegions(inputOrName, value, changedRegions) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const { rawInput, handle } = resolve(inputOrName, "graph.transaction.setWithRegions");
      rawTx.setWithRegions(rawInput, value, changedRegions);
      rememberStagedValue(rawInput, value);
      notifyInputSignalWrite(handle, value);
    },
    setWithRegionsAndAspects(inputOrName, value, changedRegions, aspects) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const { rawInput, handle } = resolve(
        inputOrName,
        "graph.transaction.setWithRegionsAndAspects",
      );
      rawTx.setWithRegionsAndAspects(rawInput, value, changedRegions, aspects);
      rememberStagedValue(rawInput, value);
      notifyInputSignalWrite(handle, value);
    },
    patch(inputOrName, patchValue) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const { rawInput, handle } = resolve(inputOrName, "graph.transaction.patch", "patch");
      const inputName = typeof inputOrName === "string"
        ? inputOrName
        : Object.entries(publishedInputs).find(([, publishedHandle]) => publishedHandle === inputOrName)?.[0];
      const nextValue = mergePatchValue(
        readCurrentValue(rawInput, handle),
        patchValue,
        "graph.transaction.patch(...)",
      );
      rawTx.set(rawInput, nextValue);
      rememberStagedValue(rawInput, nextValue);
      notifyInputSignalWrite(handle, nextValue);
    },
    free() {
      rawTx.free();
    },
    [Symbol.dispose]() {
      if (typeof rawTx[Symbol.dispose] === "function") {
        rawTx[Symbol.dispose]();
        return;
      }
      rawTx.free();
    },
  });
}

export function wrapAdapters(rawAdapters, hostCapabilities) {
  return Object.freeze({
    exportDefinitions() {
      return rawAdapters.export_definitions();
    },
    exportRuntimeEnvelope() {
      const envelope = attachSerializedField(
        rawAdapters.export_runtime_envelope(),
        "runtimeEnvelopeRestoreToken",
        rawAdapters.export_runtime_envelope_wire(),
      );
      hostCapabilities.recordExportedUnavailableCallbacks(
        envelope?.definitions?.unavailableCallbacks,
      );
      attachSerializedField(
        envelope,
        "runtimeEnvelopePortableWire",
        rawAdapters.export_runtime_envelope_portable_wire(),
      );
      return attachLiteralField(envelope, "runtimeEnvelopeRestoreMode", "SameRuntimeExact");
    },
    replaceRuntimeEnvelope(envelope) {
      throwPortableImportUnavailableCallbacks(envelope, hostCapabilities);
      if (typeof envelope?.runtimeEnvelopePortableWire === "string") {
        return rawAdapters.replace_runtime_envelope_portable_wire(envelope.runtimeEnvelopePortableWire);
      }
      return rawAdapters.replace_runtime_envelope(envelope);
    },
    restoreExactRuntimeEnvelope(envelope) {
      if (typeof envelope?.runtimeEnvelopeRestoreToken !== "string") {
        throw new TypeError(
          "adapters.restoreExactRuntimeEnvelope expects an artifact returned by adapters.exportRuntimeEnvelope()",
        );
      }
      return rawAdapters.replace_runtime_envelope_wire(envelope.runtimeEnvelopeRestoreToken);
    },
    discardExactRuntimeEnvelope(envelope) {
      if (typeof envelope?.runtimeEnvelopeRestoreToken !== "string") {
        throw new TypeError(
          "adapters.discardExactRuntimeEnvelope expects an artifact returned by adapters.exportRuntimeEnvelope()",
        );
      }
      return rawAdapters.discard_restore_token(envelope.runtimeEnvelopeRestoreToken);
    },
    runtimeProofReport() {
      return rawAdapters.runtime_proof_report();
    },
    hostCapabilityTransportReport(envelope) {
      const targetEnvelope = envelope ?? this.exportRuntimeEnvelope();
      return buildHostCapabilityTransportReport(
        targetEnvelope?.definitions?.unavailableCallbacks,
      );
    },
    free() {
      rawAdapters.free();
    },
    [Symbol.dispose]() {
      if (typeof rawAdapters[Symbol.dispose] === "function") {
        rawAdapters[Symbol.dispose]();
        return;
      }
      rawAdapters.free();
    },
  });
}
