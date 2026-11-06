import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

test("createWorkerRuntimeBridge reads committed source and derived signal truth without requiring published outputs", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const { createWorkerRuntimeBridge, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const bridge = createWorkerRuntimeBridge();
  try {
    await bridge.publishPortableGraph({
      policy: { preset: "development" },
      sources: [{ id: "counter", initial: 2 }],
      recipes: [
        {
          id: "doubleCounter",
          reads: ["counter"],
          expr: {
            kind: "sum",
            args: [
              { kind: "read", id: "counter" },
              { kind: "read", id: "counter" },
            ],
          },
          identity: { kind: "exact" },
        },
      ],
    });

    const initialPacket = await bridge.readSignals({
      signalIds: ["counter", "doubleCounter"],
    });

    assert.equal(initialPacket.envelopeFamily, "signalReadback");
    assert.equal(initialPacket.readbackMode, "CommittedSignalReadback");
    assert.equal(initialPacket.runtimeAuthority, "workerOwnedRuntime");
    assert.equal(initialPacket.signalReadbackBreadth, 2);
    assert.match(initialPacket.workerFirstTruthDigest, /^[0-9a-f]{64}$/);
    assert.match(initialPacket.packetDigest, /^[0-9a-f]{64}$/);
    assert.deepEqual(
      initialPacket.signals.map((entry) => [entry.id, entry.value]),
      [["counter", 2], ["doubleCounter", 4]],
    );

    await bridge.applyTransaction([{ kind: "set", id: "counter", value: 9 }]);
    const updatedPacket = await bridge.readSignals({
      signalIds: ["counter", "doubleCounter"],
    });

    assert.deepEqual(
      updatedPacket.signals.map((entry) => [entry.id, entry.value]),
      [["counter", 9], ["doubleCounter", 18]],
    );
    assert.notEqual(updatedPacket.workerFirstTruthDigest, initialPacket.workerFirstTruthDigest);
  } finally {
    await bridge.terminate();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});

test("worker bulk readback preserves request order across cold and changed dependency chains", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const { createWorkerRuntimeBridge, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const bridge = createWorkerRuntimeBridge();
  try {
    await bridge.publishPortableGraph({ policy: { preset: "operational" },
      sources: [{ id: "source", initial: 2 }],
      recipes: [
        { id: "middle", reads: ["source"], expr: { kind: "sum",
          args: [{ kind: "read", id: "source" }, { kind: "value", value: 1 }] }, identity: { kind: "exact" } },
        { id: "last", reads: ["middle"], expr: { kind: "sum",
          args: [{ kind: "read", id: "middle" }, { kind: "value", value: 1 }] }, identity: { kind: "exact" } },
      ] });
    const read = () => bridge.readSignals({ signalIds: ["last", "source", "middle"] });
    assert.deepEqual((await read()).signals.map(({ id, value }) => [id, value]),
      [["last", 4], ["source", 2], ["middle", 3]]);
    await bridge.applyTransaction([{ kind: "set", id: "source", value: 8 }]);
    await assert.rejects(bridge.readSignals({ signalIds: ["last", "missing"] }), /unknown signal id/);
    assert.deepEqual((await read()).signals.map(({ id, value }) => [id, value]),
      [["last", 10], ["source", 8], ["middle", 9]]);
  } finally {
    await bridge.terminate();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});

test("createWorkerRuntimeBridge signal readback rejects malformed signal batches before claiming worker truth", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const { createWorkerRuntimeBridge, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const bridge = createWorkerRuntimeBridge();
  try {
    await bridge.publishPortableGraph({
      policy: { preset: "development" },
      sources: [{ id: "counter", initial: 1 }],
      recipes: [],
    });

    await assert.rejects(
      () => bridge.readSignals({ signalIds: ["counter", "counter"] }),
      /duplicate signal id `counter`/,
    );
  } finally {
    await bridge.terminate();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});
