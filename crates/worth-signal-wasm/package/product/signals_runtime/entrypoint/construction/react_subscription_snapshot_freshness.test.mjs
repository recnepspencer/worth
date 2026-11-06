import assert from "node:assert/strict";
import test from "node:test";
import { Worker } from "node:worker_threads";

import { loadStoreModule } from "../../../host_capabilities_certification/module_loading/load_store_module.mjs";
import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

for (const deployment of ["mainThreadCompatibility", "workerFirst"]) {
  test(`${deployment}: first React subscription revalidates the render snapshot`, async () => {
    const previousWorker = globalThis.Worker;
    globalThis.Worker = Worker;
    const { createSignals, cleanup: cleanupSignals } = await loadSignalsModule({ rawSurface: "real" });
    const { createReactSignalsStore, cleanup: cleanupStore } = await loadStoreModule();
    let signals;
    let store;
    try {
      signals = await createSignals({ deployment });
      store = createReactSignalsStore(signals);
      const changed = await signals.inputAsync({ count: 0 });
      const unchanged = await signals.inputAsync({ count: 0 });
      const stableSnapshot = store.getSignalSnapshot(unchanged);
      assert.deepEqual(store.getSignalSnapshot(changed), { count: 0 });

      await changed.set({ count: 1 });
      const stopChanged = store.subscribeSignal(changed, () => {});
      const stopUnchanged = store.subscribeSignal(unchanged, () => {});
      try {
        assert.deepEqual(store.getSignalSnapshot(changed), { count: 1 });
        assert.equal(store.getSignalSnapshot(unchanged), stableSnapshot,
          "unchanged values retain snapshot reference identity when subscribing");
      } finally {
        stopChanged();
        stopUnchanged();
      }
    } finally {
      store?.dispose();
      await signals?.terminate();
      await cleanupStore();
      await cleanupSignals();
      globalThis.Worker = previousWorker;
    }
  });
}
