import { readLineValue } from "./line_value_read.js";

/**
 * Resource line.signal() must expose binding-local value truth immediately.
 * Worker-first authored input publication is asynchronous; awaiting settlement
 * updates binding.state before valueSignal's worker transaction lands. Reading
 * the published readable computed by id can therefore still be null while
 * line.value() is already fulfilled — which breaks React store snapshots.
 */
export function createLineSignalLocalTruthHandle(materialization) {
  const underlying = materialization.binding.readableValueSignal;
  const read = () => readLineValue(materialization);
  const handle = function lineSignalLocalTruthHandle() {
    return read();
  };
  Object.defineProperties(handle, {
    id: {
      enumerable: true,
      get() {
        return underlying.id;
      },
    },
    debugName: {
      enumerable: true,
      get() {
        return underlying.debugName ?? null;
      },
    },
    get: {
      enumerable: false,
      value: read,
    },
    peek: {
      enumerable: false,
      value: read,
    },
    free: {
      enumerable: false,
      value() {},
    },
    [Symbol.dispose]: {
      enumerable: false,
      value() {},
    },
  });
  // Every consumer that keys on product identity (signals.graph outputs,
  // watch, React bindings, runtime-affinity checks) sees this handle as the
  // line's readable computed: its symbol-keyed identity is the computed's
  // own. Only the value read is binding-local.
  for (const symbol of Object.getOwnPropertySymbols(underlying)) {
    if (symbol === Symbol.dispose) {
      continue;
    }
    Object.defineProperty(handle, symbol, {
      enumerable: false,
      get() {
        return underlying[symbol];
      },
    });
  }
  return handle;
}
