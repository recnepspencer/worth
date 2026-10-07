import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

test("default worker-first root watch and effect consume committed worker observation delivery for imported-graph mutation", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });

  const compatibilitySignals = await createSignals({ deployment: "mainThreadCompatibility" });
  const count = compatibilitySignals.input(4, { debugName: "count" });
  const graph = compatibilitySignals.graph("workerFirstRootObservationDelivery", {
    inputs: { count },
    outputs: {
      doubled: compatibilitySignals.computedSpec("worker:first:root:observation:doubled", {
        reads: [count.id],
        expr: {
          kind: "sum",
          args: [
            { kind: "read", id: count.id },
            { kind: "read", id: count.id },
          ],
        },
        identity: { kind: "exact" },
      }),
    },
  });
  graph.writeInput("count", 9);
  const definition = graph.exportDefinition();
  const snapshot = graph.exportSnapshot();
  const outputId = graph.output("doubled").id;

  try {
    const workerSignals = await createSignals();
    const importedGraph = workerSignals.importGraph(definition, snapshot);
    await importedGraph.ready();

    const notices = [];
    let effectCount = 0;
    const watchHandle = workerSignals.watch(outputId, (notice) => {
      notices.push(notice);
    });
    const effectHandle = workerSignals.effect(importedGraph.output("doubled"), () => {
      effectCount += 1;
    });

    const runSummary = await importedGraph.writeInput("count", 11);

    assert.equal(workerSignals.read(outputId), 22);
    assert.ok(runSummary.nodesRecomputed >= 1);
    assert.equal(notices.length, 1);
    assert.equal(notices[0].signalId, outputId);
    assert.equal(notices[0].visited, true);
    assert.equal(notices[0].recomputed, true);
    assert.equal(notices[0].meaningfulChange, true);
    assert.equal(notices[0].triggerMatched, true);
    assert.equal(effectCount, 1);

    await importedGraph.writeInput("count", 11);
    assert.equal(notices.length, 1);
    assert.equal(effectCount, 1);

    watchHandle.free();
    effectHandle.free();
    await importedGraph.writeInput("count", 12);
    assert.equal(workerSignals.read(outputId), 24);
    assert.equal(notices.length, 1);
    assert.equal(effectCount, 1);

    await importedGraph.terminate();
    workerSignals.free();
  } finally {
    compatibilitySignals.free();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});
