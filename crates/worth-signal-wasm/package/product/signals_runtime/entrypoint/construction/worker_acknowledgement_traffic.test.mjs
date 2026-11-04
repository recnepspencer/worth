import assert from "node:assert/strict";
import test from "node:test";
import { Worker } from "node:worker_threads";
import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

test("warm acknowledgements use one exchange across graph sizes and preserve committed diagnostics", async () => {
  const previousWorker = globalThis.Worker;
  const methods = [];
  globalThis.Worker = class extends Worker {
    postMessage(message, ...args) {
      if (message.method) methods.push(message.method);
      return super.postMessage(message, ...args);
    }
  };
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  try {
    for (const count of [0, 1, 64]) {
      const signals = await createSignals({ deployment: "workerFirst" });
      try {
        const input = await signals.inputAsync(0);
        let output = input;
        for (let index = 0; index < count; index++) {
          output = await signals.computedAsync({ reads: [output.id],
            expr: { kind: "sum", args: [{ kind: "read", id: output.id }, { kind: "value", value: 1 }] },
            identity: { kind: "exact" } });
        }
        await input.set(1);
        const notifications = [];
        const stop = signals.diagnostics().subscribe(() => notifications.push(output()));
        methods.length = 0;
        await input.set(2);
        assert.equal(output(), count + 2);
        assert.deepEqual(methods, ["applyTransactionAcknowledged"], `graph size ${count}`);
        assert.deepEqual(notifications, [count + 2]);
        assert.ok(signals.diagnostics().health());
        assert.ok(signals.diagnostics().summaryNow());
        assert.ok(signals.diagnostics().historyNow());
        assert.ok(signals.diagnostics().latestFlow());
        stop.free();
      } finally { await signals.terminate(); }
    }
  } finally { globalThis.Worker = previousWorker; await cleanup(); }
});

test("imported acknowledgement preserves synchronous values, versions, history and restore artifacts", async () => {
  const previousWorker = globalThis.Worker;
  const methods = [];
  globalThis.Worker = class extends Worker {
    postMessage(message, ...args) {
      if (message.method) methods.push(message.method);
      return super.postMessage(message, ...args);
    }
  };
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  let source, signals;
  try {
    source = await createSignals({ deployment: "mainThreadCompatibility" });
    const count = source.input(0);
    const graph = source.graph("ack-import", { inputs: { count }, outputs: { count } });
    signals = await createSignals({ deployment: "workerFirst" });
    const imported = signals.importGraph(graph.exportDefinition(), graph.exportSnapshot());
    await imported.ready();
    const input = imported.input("count");
    methods.length = 0;
    await input.set(7);
    assert.deepEqual(methods, ["applyTransactionAcknowledged"]);
    assert.equal(input(), 7);
    assert.equal(imported.output("count")(), 7);
    assert.ok(signals.specialist().readVersions([input.id])[0]);
    assert.ok(signals.history().replay_for(input.id));
    assert.ok(signals.history().snapshot());
    assert.ok(imported.exportSnapshot().runtimeEnvelope);
    assert.deepEqual(methods, ["applyTransactionAcknowledged"], "synchronous facades must use committed artifacts");
  } finally {
    await signals?.terminate(); await source?.terminate();
    globalThis.Worker = previousWorker; await cleanup();
  }
});

test("a host callback's dependent write returns fresh downstream readback without extra polling", async () => {
  const previousWorker = globalThis.Worker;
  const methods = [];
  globalThis.Worker = class extends Worker {
    postMessage(message, ...args) {
      if (message.method) methods.push(message.method);
      return super.postMessage(message, ...args);
    }
  };
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  let signals;
  try {
    signals = await createSignals();
    const input = await signals.inputAsync(1);
    const callback = await signals.computedAsync(() => input() * 2);
    const output = await signals.computedAsync({ reads: [callback.id],
      expr: { kind: "sum", args: [{ kind: "read", id: callback.id }, { kind: "value", value: 1 }] },
      identity: { kind: "exact" } });
    methods.length = 0;
    await input.set(3);
    assert.equal(callback(), 6);
    assert.equal(output(), 7, "the final receipt must replace readback taken before the callback write");
    assert.deepEqual(methods, ["applyTransactionAcknowledged", "applyTransactionAcknowledged"]);
  } finally {
    await signals?.terminate(); globalThis.Worker = previousWorker; await cleanup();
  }
});
