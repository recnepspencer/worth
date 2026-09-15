import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import test from "node:test";
import { fileURLToPath } from "node:url";

const reactDir = path.dirname(fileURLToPath(import.meta.url));

async function loadStoreModule() {
  const tempDir = await mkdtemp(path.join(tmpdir(), "worth-signal-react-store-"));
  const sourceFiles = [
    ["model.ts", "model.js"],
    ["signal_snapshot_equality.ts", "signal_snapshot_equality.js"],
    ["store.ts", "store.js"],
  ];
  try {
    for (const [sourceName, outputName] of sourceFiles) {
      const sourcePath = path.join(reactDir, sourceName);
      const source = await readFile(sourcePath, "utf8");
      const transformed = stripTypeScriptTypes(source, { mode: "transform" });
      await writeFile(path.join(tempDir, outputName), transformed, "utf8");
    }
    const moduleUrl = new URL(`file:///${path.join(tempDir, "store.js").replace(/\\/g, "/")}`);
    const loaded = await import(moduleUrl.href);
    return { ...loaded, cleanup: () => rm(tempDir, { recursive: true, force: true }) };
  } catch (error) {
    await rm(tempDir, { recursive: true, force: true });
    throw error;
  }
}

function flushMicrotasks() {
  return new Promise((resolve) => queueMicrotask(resolve));
}

test("createReactSignalsStore does not monkey-patch shared transaction facades", async () => {
  const { createReactSignalsStore, cleanup } = await loadStoreModule();
  try {
    const originalTransaction = () => ({ touchedNodes: 1 });
    const originalBatch = () => ({ touchedNodes: 2 });
    let diagnosticsCallback = null;
    const signals = {
      read(target) {
        return typeof target === "string" ? `${target}:snapshot` : target.get();
      },
      watch() {
        return { label: "runtime-handle" };
      },
      nuke() {
        return true;
      },
      diagnostics() {
        return {
          latestObservation: () => null,
          latestFlow: () => null,
          performanceSummary: () => ({ deliveredObservationCount: 0 }),
          subscribe(callback) {
            diagnosticsCallback = callback;
            return {
              free() {
                diagnosticsCallback = null;
              },
            };
          },
        };
      },
      compatibilityApp() {
        throw new Error("compatibilityApp should not be needed for app-first store reads");
      },
      transaction: originalTransaction,
      batch: originalBatch,
    };

    const store = createReactSignalsStore(signals);

    assert.equal(signals.transaction, originalTransaction);
    assert.equal(signals.batch, originalBatch);

    store.dispose();
    assert.equal(diagnosticsCallback, null);

    assert.equal(signals.transaction, originalTransaction);
    assert.equal(signals.batch, originalBatch);
  } finally {
    await cleanup();
  }
});

test("createReactSignalsStore enters root read on every snapshot, keeps stable snapshots, and refreshes diagnostics locally", async () => {
  const { createReactSignalsStore, cleanup } = await loadStoreModule();
  try {
    let watchCallback = null;
    let diagnosticsVersion = 0;
    let readCalls = 0;
    let compatibilityReads = 0;
    let transactionCalls = 0;
    let batchCalls = 0;
    let diagnosticsCallback = null;
    const handle = {
      id: "count",
      get() {
        throw new Error("store should use signals.read instead of handle.get for shared snapshots");
      },
    };

    // The runtime value changes only when the test says so; every read hands
    // back a fresh copy, the way the compatibility deployment deserializes.
    let countValue = { total: 1 };
    const foreignHandle = { id: "foreign", get: () => "foreign" };
    const signals = {
      read(target) {
        readCalls += 1;
        const id = typeof target === "string" ? target : target.id;
        if (id === "foreign") {
          throw new Error("`foreign` is not a currently available worker-first signal");
        }
        return { ...countValue };
      },
      watch(target, callback) {
        assert.equal(target, handle);
        watchCallback = callback;
        return { runtime: "watch-handle" };
      },
      nuke(runtimeHandle) {
        assert.deepEqual(runtimeHandle, { runtime: "watch-handle" });
        return true;
      },
      diagnostics() {
        return {
          latestObservation: () => ({ version: diagnosticsVersion }),
          latestFlow: () => null,
          performanceSummary: () => ({ version: diagnosticsVersion }),
          subscribe(callback) {
            diagnosticsCallback = callback;
            return {
              free() {
                diagnosticsCallback = null;
              },
            };
          },
        };
      },
      compatibilityApp() {
        return {
          read() {
            compatibilityReads += 1;
            return "compatibility";
          },
        };
      },
      transaction(callback) {
        transactionCalls += 1;
        callback({ set() {} });
        diagnosticsVersion += 1;
        diagnosticsCallback?.();
        return { touchedNodes: 1 };
      },
      batch(callback) {
        batchCalls += 1;
        callback({ set() {} });
        diagnosticsVersion += 1;
        diagnosticsCallback?.();
        return { touchedNodes: 2 };
      },
    };

    const store = createReactSignalsStore(signals);
    const diagnosticsSnapshots = [];
    const unsubscribeDiagnostics = store.subscribeDiagnostics(() => {
      diagnosticsSnapshots.push(store.getDiagnosticsSnapshot());
    });

    // Unsubscribed (React renders before it subscribes): every snapshot enters
    // root read, and an unchanged value keeps the cached reference.
    const first = store.getSignalSnapshot(handle);
    const second = store.getSignalSnapshot(handle);
    assert.deepEqual(first, { total: 1 });
    assert.equal(second, first, "an unchanged value must keep its snapshot reference");
    assert.equal(readCalls, 2, "every snapshot read enters root read");
    assert.equal(compatibilityReads, 0);

    // A changed value with no watch yet is adopted by the next read.
    countValue = { total: 5 };
    const changedBeforeSubscribe = store.getSignalSnapshot(handle);
    assert.deepEqual(changedBeforeSubscribe, { total: 5 });
    assert.notEqual(changedBeforeSubscribe, first);
    assert.equal(readCalls, 3);

    // Root read authority is never bypassed by the cache.
    assert.throws(
      () => store.getSignalSnapshot(foreignHandle),
      /not a currently available worker-first signal/,
    );
    assert.equal(readCalls, 4);

    const unsubscribeSignal = store.subscribeSignal(handle, () => {});
    assert.ok(watchCallback, "subscribeSignal should establish one runtime watch");
    // Subscribed and current: the read is the authority check, the snapshot is
    // the cached reference.
    countValue = { total: 7 };
    assert.equal(store.getSignalSnapshot(handle), changedBeforeSubscribe);
    assert.equal(readCalls, 5);

    diagnosticsVersion += 1;
    diagnosticsCallback?.();
    watchCallback({ triggerMatched: true, meaningfulChange: true });
    await flushMicrotasks();

    // A watch notice invalidates the cache; the next read adopts the new value.
    const third = store.getSignalSnapshot(handle);
    assert.deepEqual(third, { total: 7 });
    assert.notEqual(third, changedBeforeSubscribe);
    assert.equal(readCalls, 6);
    assert.equal(store.getSignalSnapshot(handle), third);
    assert.equal(readCalls, 7);

    signals.transaction(() => {});
    await flushMicrotasks();
    signals.batch(() => {});
    await flushMicrotasks();

    assert.equal(transactionCalls, 1);
    assert.equal(batchCalls, 1);
    assert.equal(diagnosticsSnapshots.length, 3);
    assert.deepEqual(
      diagnosticsSnapshots.map((snapshot) => snapshot.performanceSummary.version),
      [1, 2, 3],
    );

    unsubscribeSignal();
    unsubscribeDiagnostics();
    store.dispose();
    assert.equal(diagnosticsCallback, null);
  } finally {
    await cleanup();
  }
});
