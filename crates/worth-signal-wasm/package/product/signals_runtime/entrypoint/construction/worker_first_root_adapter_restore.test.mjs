import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

test("default worker-first root adapters restore runtime envelopes, preserve truth on denied import, and invalidate active graph reads", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });

  const compatibilitySignals = await createSignals({ deployment: "mainThreadCompatibility" });
  const count = compatibilitySignals.input(2, { debugName: "count" });
  const graph = compatibilitySignals.graph("workerFirstRootAdapterMutation", {
    inputs: { count },
    outputs: {
      doubled: compatibilitySignals.computedSpec("worker:first:root:adapter:doubled", {
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
  const definition = graph.exportDefinition();
  const baselineSnapshot = graph.exportSnapshot();
  graph.writeInput("count", 9);
  const changedSnapshot = graph.exportSnapshot();
  const outputId = graph.output("doubled").id;

  const callbackSignals = await createSignals({ deployment: "mainThreadCompatibility" });
  const callbackCount = callbackSignals.input(5);
  callbackSignals.scope("workerFirstRootAdapterDenied").computedSpec("callbackBacked", {
    compute: () => callbackCount.value() + 1,
  });
  const deniedArtifact = callbackSignals.adapters().exportRuntimeEnvelope();

  try {
    const workerSignals = await createSignals();
    const importedGraph = workerSignals.importGraph(definition, baselineSnapshot);
    await importedGraph.ready();

    assert.equal(workerSignals.read(outputId), 4);

    const deniedPortableImport = await workerSignals.adapters().replaceRuntimeEnvelope(deniedArtifact);
    assert.equal(deniedPortableImport.importOutcome, "Denied");
    assert.equal(workerSignals.read(outputId), 4);
    assert.equal(importedGraph.read().doubled, 4);

    const portableImport = await workerSignals.adapters().replaceRuntimeEnvelope(changedSnapshot.runtimeEnvelope);
    assert.equal(portableImport.importOutcome, "Admitted");
    assert.throws(
      () => importedGraph.read(),
      /replaced the active imported graph runtime/,
    );
    assert.throws(
      () => workerSignals.read(outputId),
      /currently available worker-first signal/,
    );

    const reimportedAfterPortable = workerSignals.importGraph(definition, changedSnapshot);
    await reimportedAfterPortable.ready();
    assert.equal(workerSignals.read(outputId), 18);
    const workerChangedArtifact = workerSignals.adapters().exportRuntimeEnvelope();
    const summaryBeforeExact = workerSignals.diagnostics().summaryNow();

    const exactImport = await workerSignals.adapters().restoreExactRuntimeEnvelope(workerChangedArtifact);
    assert.equal(exactImport.importOutcome, "AdmittedExact");
    assert.throws(
      () => reimportedAfterPortable.read(),
      /replaced the active imported graph runtime/,
    );
    // The imported graph handle is gone, but the root's live diagnostics
    // describe the runtime that now exists: the exactly restored one, which
    // has the same graph the replaced import had.
    const summaryAfterExact = workerSignals.diagnostics().summaryNow();
    assert.equal(summaryAfterExact.active_node_count, summaryBeforeExact.active_node_count);
    assert.equal(summaryAfterExact.dependency_edge_count, summaryBeforeExact.dependency_edge_count);

    const reimportedAfterExact = workerSignals.importGraph(definition, changedSnapshot);
    await reimportedAfterExact.ready();
    assert.equal(workerSignals.read(outputId), 18);

    const portableBaselineImport = await workerSignals.adapters().replaceRuntimeEnvelope(baselineSnapshot.runtimeEnvelope);
    assert.equal(portableBaselineImport.importOutcome, "Admitted");
    assert.throws(
      () => reimportedAfterExact.read(),
      /replaced the active imported graph runtime/,
    );

    const reimportedAfterBaseline = workerSignals.importGraph(definition, baselineSnapshot);
    await reimportedAfterBaseline.ready();
    assert.equal(workerSignals.read(outputId), 4);

    await reimportedAfterBaseline.terminate();
    await workerSignals.terminate();
  } finally {
    callbackSignals.free();
    compatibilitySignals.free();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});
