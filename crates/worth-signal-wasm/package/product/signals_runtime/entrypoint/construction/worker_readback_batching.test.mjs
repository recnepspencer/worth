import assert from "node:assert/strict";
import test from "node:test";
import { Worker } from "node:worker_threads";
import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

test("worker mutation readback round trips stay bounded as the derived chain grows", async () => {
  const previousWorker = globalThis.Worker;
  const packets = [];
  globalThis.Worker = class extends Worker {
    postMessage(message, ...args) {
      if (message.method === "readSignals") packets.push(message.args[0].signalIds.length);
      return super.postMessage(message, ...args);
    }
  };
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const measurements = [];
  try {
    for (const nodeCount of [1, 64]) {
      const signals = await createSignals({ deployment: "workerFirst" });
      try {
        const input = await signals.inputAsync(0);
        let output = input;
        for (let index = 0; index < nodeCount; index += 1) {
          const previous = output;
          output = await signals.computedAsync({ reads: [previous.id],
            expr: { kind: "sum", args: [{ kind: "read", id: previous.id }, { kind: "value", value: 1 }] },
            identity: { kind: "exact" } });
        }
        packets.length = 0;
        await input.set(1);
        assert.equal(output(), nodeCount + 1);
        measurements.push({ nodeCount, requests: packets.length, largestBatch: Math.max(...packets) });
      } finally { await signals.terminate(); }
    }
    const [small, large] = measurements;
    assert.ok(large.requests <= small.requests * 2 + 2,
      `readback must batch the graph, rather than serialize a round trip per node: ${JSON.stringify(measurements)}`);
    assert.ok(large.largestBatch > 1, "a multi-node readback must use the public bulk bridge operation");
  } finally {
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});

test("acknowledged full and regional writes release dependent worker readback", async () => {
  const { importProductModule, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const base = "entrypoint/sessions/support/authored/";
  const { createWorkerFirstAuthoredInputState, applyCommittedWorkerFirstAuthoredInputs } =
    await importProductModule(`${base}worker_first_authored_input_state.js`);
  const { pendingWorkerInputIds } = await importProductModule(`${base}worker_first_authored_readback_currentness.js`);
  const { createWorkerFirstAuthoredReadableState } = await importProductModule(`${base}worker_first_authored_readable_state.js`);
  const { refreshWorkerFirstAuthoredReadableSignals } = await importProductModule(`${base}worker_first_authored_readable_refresh.js`);
  try {
    for (const kind of ["set", "setWithRegions"]) {
      const input = createWorkerFirstAuthoredInputState(0);
      input.hostTipEpoch = 1;
      input.currentValue = 1;
      const inputs = new Map([["source", input]]);
      const readable = createWorkerFirstAuthoredReadableState("computed", 0, ["source"]);
      const deps = { authoredReadables: new Map([["derived", readable]]),
        pendingInputIds: () => pendingWorkerInputIds(inputs, null),
        bridge: { async readSignals() { return { signals: [{ id: "derived", value: 2 }] }; } } };
      await refreshWorkerFirstAuthoredReadableSignals(deps);
      assert.equal(readable.currentValue, 0, "unacknowledged input cannot admit older worker truth");
      applyCommittedWorkerFirstAuthoredInputs(inputs, [{ kind, id: "source", value: 1, epochAtWrite: 1 }]);
      await refreshWorkerFirstAuthoredReadableSignals(deps);
      assert.equal(readable.currentValue, 2, `${kind} acknowledgement must release its dependent readback`);
    }
  } finally { await cleanup(); }
});

test("bulk readback isolates history-removed readables and preserves unrelated failures", async () => {
  const { importProductModule, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const { refreshWorkerFirstAuthoredReadableSignals } = await importProductModule(
    "entrypoint/sessions/support/authored/worker_first_authored_readable_refresh.js");
  const { createWorkerFirstAuthoredReadableState } = await importProductModule(
    "entrypoint/sessions/support/authored/worker_first_authored_readable_state.js");
  const states = new Map(["live-a", "removed", "live-b"].map(id =>
    [id, createWorkerFirstAuthoredReadableState("computed", 0)]));
  const packets = [];
  const bridge = {
    async readSignals({ signalIds }) {
      packets.push(signalIds);
      if (signalIds.includes("removed")) throw new Error("unknown signal id `removed`");
      return { signals: signalIds.map(id => ({ id, value: 42 })) };
    },
  };
  const deps = { bridge, authoredReadables: states, pendingInputIds: () => [] };
  try {
    await refreshWorkerFirstAuthoredReadableSignals(deps);
    assert.ok(packets[0].length > 1, "the ordinary lane must attempt the bulk read first");
    assert.equal(states.get("live-a").currentValue, 42);
    assert.equal(states.get("live-b").currentValue, 42);
    assert.match(states.get("removed").invalidatedMessage, /no longer exists/);
    states.get("live-a").currentValue = 0;
    const failure = new Error("worker transport failed");
    bridge.readSignals = async () => { throw failure; };
    await assert.rejects(refreshWorkerFirstAuthoredReadableSignals(deps),
      error => error === failure);
    assert.equal(states.get("live-a").currentValue, 0, "transport failures cannot counterfeit fresh state");
  } finally { await cleanup(); }
});
