import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";
import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

test("root shutdown retains imported sources until a pending authored callback completes", { timeout: 20_000 }, async () => {
  const previousWorker = globalThis.Worker;
  const entered = Promise.withResolvers();
  let holdMutation = false;
  let releaseMutation = () => {};
  class GatedWorker extends NodeWorker {
    postMessage(message, ...options) {
      if (holdMutation && message.method === "applyTransactionAcknowledged") {
        holdMutation = false;
        releaseMutation = () => {
          releaseMutation = () => {};
          super.postMessage(message, ...options);
        };
        entered.resolve();
        return;
      }
      return super.postMessage(message, ...options);
    }
  }
  globalThis.Worker = GatedWorker;
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  let compatibility = null;
  let signals = null;
  let mutation = null;
  try {
    compatibility = await createSignals({ deployment: "mainThreadCompatibility" });
    const count = compatibility.input(1);
    const source = compatibility.graph("shutdownSource", {
      inputs: { count },
      outputs: {
        doubled: compatibility.spec.output("shutdownDoubled", {
          reads: [count.id],
          expr: {
            kind: "sum",
            args: [{ kind: "read", id: count.id }, { kind: "read", id: count.id }],
          },
          identity: { kind: "exact" },
        }),
      },
    });
    signals = await createSignals();
    const imported = signals.importGraph(source.exportDefinition(), source.exportSnapshot());
    await imported.ready();
    const importedCount = imported.input("count");
    let closingRequested = false;
    const readsWhileClosing = [];
    const dependent = signals.computed(() => {
      const value = importedCount();
      if (closingRequested) readsWhileClosing.push(value);
      return value * 3;
    });
    assert.equal(dependent(), 3);
    await signals.settleAuthoredWork();

    holdMutation = true;
    mutation = imported.writeInput("count", 2);
    await entered.promise;
    closingRequested = true;
    const closing = signals.terminate();
    assert.equal(signals.terminate(), closing, "shutdown must share its pending completion");
    let closed = false;
    void closing.then(() => { closed = true; }, () => { closed = true; });
    await new Promise(setImmediate);
    assert.equal(closed, false, "shutdown must wait for the held authored mutation");
    assert.equal(importedCount(), 2, "the imported source must remain readable while work is pending");

    releaseMutation();
    await mutation;
    await closing;
    assert.ok(readsWhileClosing.includes(2), "the pending callback must read the live source after shutdown was requested");
    assert.throws(() => importedCount(), /terminated|active|available|disposed|invalidated/u);
  } finally {
    releaseMutation();
    await mutation?.catch(() => {});
    await signals?.terminate().catch(() => {});
    await compatibility?.terminate();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});
