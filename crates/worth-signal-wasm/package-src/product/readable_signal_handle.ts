import {
  activeComputedCallbackFrame,
  activeRuntimeCallbackReader,
  activeRuntimeCallbackReads,
  denySignalReadFromForeignRuntime,
  denyUnavailableRuntimeCallbackRead,
} from "./callback_frames.js";
import { runRegisteredCleanup } from "./signal_handle_cleanup.js";
import {
  DEBUG_NAME,
  PRODUCT_SIGNAL_KIND,
  RAW_SIGNAL_HANDLE,
  RAW_SIGNALS,
} from "./symbols.js";

export function wrapReadableSignal(rawHandle, rawSignals, kind = "signal", debugName = null) {
  // Native recipes own their nodes beyond the lifetime of a borrowed FFI wrapper.
  // Callback snapshots address those nodes by stable identity, without FFI access.
  const signalId = rawHandle.id;
  const signal = function signal() {
    const frame = activeComputedCallbackFrame();
    if (frame) {
      if (frame.rawSignals !== rawSignals) {
        denySignalReadFromForeignRuntime(signalId);
      }
      frame.reads.add(signalId);
      const runtimeReads = activeRuntimeCallbackReads();
      if (runtimeReads) {
        frame.runtimeReadIds.add(signalId);
        if (Object.prototype.hasOwnProperty.call(runtimeReads, signalId)) {
          return runtimeReads[signalId];
        }
        const runtimeReader = activeRuntimeCallbackReader();
        if (runtimeReader) {
          return runtimeReader(signalId);
        }
        denyUnavailableRuntimeCallbackRead(signalId);
      }
    }
    return rawHandle.get();
  };

  Object.defineProperties(signal, {
    id: {
      enumerable: true,
      get() {
        return signalId;
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
            denySignalReadFromForeignRuntime(signalId);
          }
          frame.reads.add(signalId);
          const runtimeReads = activeRuntimeCallbackReads();
          if (runtimeReads) {
            frame.runtimeReadIds.add(signalId);
            if (Object.prototype.hasOwnProperty.call(runtimeReads, signalId)) {
              return runtimeReads[signalId];
            }
            const runtimeReader = activeRuntimeCallbackReader();
            if (runtimeReader) {
              return runtimeReader(signalId);
            }
            denyUnavailableRuntimeCallbackRead(signalId);
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

