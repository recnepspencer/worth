import {
  activeComputedCallbackFrame,
  activeRuntimeCallbackReader,
  activeRuntimeCallbackReads,
  denySignalMutationDuringCallbackAuthoring,
  denySignalReadFromForeignRuntime,
  denyUnavailableRuntimeCallbackRead,
} from "./callback_frames.js";
import {
  DEBUG_NAME,
  INPUT_BASELINE_VALUE,
  PRODUCT_SIGNAL_KIND,
  RAW_SIGNAL_HANDLE,
  RAW_SIGNALS,
} from "./symbols.js";

const INPUT_BASELINE_VALUES = new WeakMap();
const SIGNAL_CLEANUP_CALLBACKS = new WeakMap();
const INPUT_WRITE_OBSERVERS = new WeakMap();
const INPUT_RESET_PREPARERS = new WeakMap();
// rawSignals -> Map<inputId, product input handle>. Lets the compatibility
// host tip surface resolve batch writes by id without a raw-runtime lookup.
const INPUT_HANDLES_BY_RUNTIME = new WeakMap();

function describeHandleKind(target) {
  if (!target || typeof target !== "function") {
    return null;
  }
  return target[PRODUCT_SIGNAL_KIND] ?? null;
}

function signalIdForError(target) {
  if (typeof target?.[DEBUG_NAME] === "string" && target[DEBUG_NAME].length > 0) {
    return target[DEBUG_NAME];
  }
  return typeof target?.id === "string" ? target.id : "<unknown>";
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isPatchableSignalValue(value) {
  return isPlainObject(value) || Array.isArray(value);
}

function cloneSignalValue(value) {
  if (typeof globalThis.structuredClone === "function") {
    try {
      return globalThis.structuredClone(value);
    } catch {
      return value;
    }
  }
  if (Array.isArray(value)) {
    return value.slice();
  }
  if (isPlainObject(value)) {
    return { ...value };
  }
  return value;
}

function mergePatchedValue(currentValue, patchValue, operation) {
  if (!isPatchableSignalValue(currentValue) || !isPatchableSignalValue(patchValue)) {
    throw new TypeError(`${operation} requires object or array values`);
  }
  if (Array.isArray(currentValue) || Array.isArray(patchValue)) {
    return cloneSignalValue(patchValue);
  }
  return {
    ...currentValue,
    ...patchValue,
  };
}

function mergeAssignedValue(currentValue, assignedFields, operation) {
  if (!isPlainObject(currentValue) || !isPlainObject(assignedFields)) {
    throw new TypeError(`${operation} requires plain object values`);
  }
  return {
    ...currentValue,
    ...assignedFields,
  };
}

function runRegisteredCleanup(signal) {
  const cleanups = SIGNAL_CLEANUP_CALLBACKS.get(signal);
  if (!cleanups || cleanups.length === 0) {
    return;
  }
  SIGNAL_CLEANUP_CALLBACKS.delete(signal);
  for (const cleanup of cleanups.splice(0)) {
    cleanup();
  }
}

function notifyInputWriteObservers(signal, nextValue) {
  const observers = INPUT_WRITE_OBSERVERS.get(signal);
  if (!observers || observers.length === 0) {
    return;
  }
  for (const observer of observers) {
    observer(cloneSignalValue(nextValue));
  }
}

function prepareRegisteredInputReset(signal) {
  const prepareReset = INPUT_RESET_PREPARERS.get(signal);
  if (typeof prepareReset !== "function") {
    const value = cloneSignalValue(INPUT_BASELINE_VALUES.get(signal));
    return {
      value,
      finalize() {
        INPUT_BASELINE_VALUES.set(signal, cloneSignalValue(value));
      },
    };
  }
  const prepared = prepareReset();
  if (!prepared || typeof prepared !== "object") {
    throw new TypeError("input reset preparer must return an object");
  }
  return {
    value: cloneSignalValue(prepared.value),
    finalize: typeof prepared.finalize === "function" ? prepared.finalize : () => {},
  };
}

function isProductSignalHandle(target) {
  return (
    target
    && typeof target === "function"
    && RAW_SIGNAL_HANDLE in target
    && RAW_SIGNALS in target
    && PRODUCT_SIGNAL_KIND in target
  );
}

function invalidTargetError(operation) {
  return new TypeError(
    `${operation} expects a string id, a product signal handle created by this package, or a callable view handle with a stable id`,
  );
}

function foreignRuntimeError(operation, signalId) {
  return new TypeError(
    `${operation} cannot use signal \`${signalId}\` from a different Signals runtime`,
  );
}

function nonInputMutationError(operation, kind, signalId) {
  return new TypeError(
    `${operation} expects an input handle, but received a ${kind} handle for \`${signalId}\``,
  );
}

export function requireProductSignalHandle(target, rawSignals, operation) {
  if (!isProductSignalHandle(target)) {
    throw invalidTargetError(operation);
  }
  if (target[RAW_SIGNALS] !== rawSignals) {
    throw foreignRuntimeError(operation, signalIdForError(target));
  }
  return target;
}

export function unwrapSignalTarget(target, rawSignals, operation = "signal operation") {
  if (typeof target === "string") {
    return target;
  }
  if (
    target
    && typeof target === "function"
    && !isProductSignalHandle(target)
    && typeof target.id === "string"
    && typeof target.get === "function"
  ) {
    return target.id;
  }
  return requireProductSignalHandle(target, rawSignals, operation)[RAW_SIGNAL_HANDLE];
}

export function unwrapInputSignalTarget(target, rawSignals, operation = "signal mutation") {
  const handle = requireProductSignalHandle(target, rawSignals, operation);
  const kind = describeHandleKind(handle);
  if (kind !== "input") {
    throw nonInputMutationError(operation, kind ?? "unknown", signalIdForError(handle));
  }
  return handle[RAW_SIGNAL_HANDLE];
}

export function wrapReadableSignal(rawHandle, rawSignals, kind = "signal", debugName = null) {
  const signal = function signal() {
    const frame = activeComputedCallbackFrame();
    if (frame) {
      if (frame.rawSignals !== rawSignals) {
        denySignalReadFromForeignRuntime(rawHandle.id);
      }
      frame.reads.add(rawHandle.id);
      const runtimeReads = activeRuntimeCallbackReads();
      if (runtimeReads) {
        frame.runtimeReadIds.add(rawHandle.id);
        if (Object.prototype.hasOwnProperty.call(runtimeReads, rawHandle.id)) {
          return runtimeReads[rawHandle.id];
        }
        const runtimeReader = activeRuntimeCallbackReader();
        if (runtimeReader) {
          return runtimeReader(rawHandle.id);
        }
        denyUnavailableRuntimeCallbackRead(rawHandle.id);
      }
    }
    return rawHandle.get();
  };

  Object.defineProperties(signal, {
    id: {
      enumerable: true,
      get() {
        return rawHandle.id;
      },
    },
    debugName: {
      enumerable: true,
      get() {
        return debugName;
      },
    },
    get: {
      enumerable: false,
      value() {
        const frame = activeComputedCallbackFrame();
        if (frame) {
          if (frame.rawSignals !== rawSignals) {
            denySignalReadFromForeignRuntime(rawHandle.id);
          }
          frame.reads.add(rawHandle.id);
          const runtimeReads = activeRuntimeCallbackReads();
          if (runtimeReads) {
            frame.runtimeReadIds.add(rawHandle.id);
            if (Object.prototype.hasOwnProperty.call(runtimeReads, rawHandle.id)) {
              return runtimeReads[rawHandle.id];
            }
            const runtimeReader = activeRuntimeCallbackReader();
            if (runtimeReader) {
              return runtimeReader(rawHandle.id);
            }
            denyUnavailableRuntimeCallbackRead(rawHandle.id);
          }
        }
        return rawHandle.get();
      },
    },
    value: {
      enumerable: false,
      value() {
        return signal.get();
      },
    },
    peek: {
      enumerable: false,
      value() {
        return rawHandle.peek();
      },
    },
    free: {
      enumerable: false,
      value() {
        try {
          runRegisteredCleanup(signal);
        } finally {
          return rawHandle.free();
        }
      },
    },
    [Symbol.dispose]: {
      enumerable: false,
      value() {
        try {
          runRegisteredCleanup(signal);
        } finally {
          if (typeof rawHandle[Symbol.dispose] === "function") {
            rawHandle[Symbol.dispose]();
            return;
          }
          rawHandle.free();
        }
      },
    },
    [RAW_SIGNAL_HANDLE]: {
      enumerable: false,
      value: rawHandle,
    },
    [RAW_SIGNALS]: {
      enumerable: false,
      value: rawSignals,
    },
    [PRODUCT_SIGNAL_KIND]: {
      enumerable: false,
      value: kind,
    },
    [DEBUG_NAME]: {
      enumerable: false,
      value: debugName,
    },
  });

  return signal;
}

export function wrapInputSignal(rawHandle, rawSignals, baselineValue, debugName = null) {
  const signal = wrapReadableSignal(rawHandle, rawSignals, "input", debugName);
  INPUT_BASELINE_VALUES.set(signal, cloneSignalValue(baselineValue));
  registerInputHandle(rawSignals, signal);
  Object.defineProperty(signal, "set", {
    enumerable: false,
    value(nextValue) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      const result = rawSignals.transaction((tx) => {
        tx.set(rawHandle, nextValue);
      });
      notifyInputWriteObservers(signal, nextValue);
      return result;
    },
  });
  Object.defineProperty(signal, "reset", {
    enumerable: false,
    value() {
      const preparedReset = prepareRegisteredInputReset(signal);
      const result = signal.set(preparedReset.value);
      preparedReset.finalize();
      return result;
    },
  });
  Object.defineProperty(signal, "patch", {
    enumerable: false,
    value(patchValue) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      return signal.set(
        mergePatchedValue(signal.get(), patchValue, "input.patch(...)"),
      );
    },
  });
  Object.defineProperty(signal, "assign", {
    enumerable: false,
    value(assignedFields) {
      if (activeComputedCallbackFrame()) {
        denySignalMutationDuringCallbackAuthoring();
      }
      return signal.set(
        mergeAssignedValue(signal.get(), assignedFields, "input.assign(...)"),
      );
    },
  });
  Object.defineProperty(signal, INPUT_BASELINE_VALUE, {
    enumerable: false,
    get() {
      return cloneSignalValue(INPUT_BASELINE_VALUES.get(signal));
    },
  });
  return signal;
}

function registerInputHandle(rawSignals, signal) {
  let handlesById = INPUT_HANDLES_BY_RUNTIME.get(rawSignals);
  if (!handlesById) {
    handlesById = new Map();
    INPUT_HANDLES_BY_RUNTIME.set(rawSignals, handlesById);
  }
  const id = signal.id;
  handlesById.set(id, signal);
  registerSignalCleanup(signal, () => {
    if (handlesById.get(id) === signal) {
      handlesById.delete(id);
    }
  });
}

export function lookupInputSignalHandle(rawSignals, id) {
  return INPUT_HANDLES_BY_RUNTIME.get(rawSignals)?.get(id) ?? null;
}

export function registerSignalCleanup(signal, cleanup) {
  if (typeof cleanup !== "function") {
    throw new TypeError("registerSignalCleanup expects a cleanup function");
  }
  const cleanups = SIGNAL_CLEANUP_CALLBACKS.get(signal) ?? [];
  cleanups.push(cleanup);
  SIGNAL_CLEANUP_CALLBACKS.set(signal, cleanups);
}

export function readInputBaselineValue(signal) {
  return cloneSignalValue(INPUT_BASELINE_VALUES.get(signal));
}

export function writeInputBaselineValue(signal, value) {
  INPUT_BASELINE_VALUES.set(signal, cloneSignalValue(value));
}

export function registerInputWriteObserver(signal, observer) {
  if (typeof observer !== "function") {
    throw new TypeError("registerInputWriteObserver expects an observer function");
  }
  const observers = INPUT_WRITE_OBSERVERS.get(signal) ?? [];
  observers.push(observer);
  INPUT_WRITE_OBSERVERS.set(signal, observers);
}

export function notifyInputSignalWrite(signal, nextValue) {
  notifyInputWriteObservers(signal, nextValue);
}

export function registerInputResetPreparer(signal, prepareReset) {
  if (typeof prepareReset !== "function") {
    throw new TypeError("registerInputResetPreparer expects a reset preparer function");
  }
  INPUT_RESET_PREPARERS.set(signal, prepareReset);
}

export function prepareInputReset(signal) {
  return prepareRegisteredInputReset(signal);
}
