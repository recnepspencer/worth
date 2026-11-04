import assert from "node:assert/strict";
import test from "node:test";
import { Worker } from "node:worker_threads";
import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

for (const sourceKind of ["authored declarative", "authored callback", "imported declarative"]) {
  test(`worker replies cannot regress a newer ${sourceKind} tip`, async () => {
    const previousWorker = globalThis.Worker;
    globalThis.Worker = Worker;
    const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
    let signals, compatibility;
    try {
      signals = await createSignals({ deployment: "workerFirst" });
      let input;
      if (sourceKind.startsWith("imported")) {
        compatibility = await createSignals({ deployment: "mainThreadCompatibility" });
        const count = compatibility.input(0);
        const graph = compatibility.graph("burst", { inputs: { count }, outputs: { count } });
        const imported = signals.importGraph(graph.exportDefinition(), graph.exportSnapshot());
        await imported.ready();
        input = imported.input("count");
      } else {
        input = await signals.inputAsync(0);
      }
      const doubled = sourceKind.endsWith("callback")
        ? await signals.computedAsync(() => input() * 2)
        : await signals.computedAsync({ reads: [input.id],
          expr: { kind: "sum", args: [{ kind: "read", id: input.id }, { kind: "read", id: input.id }] },
          identity: { kind: "exact" } });
      const observed = [];
      const subscription = signals.watch(doubled, () => { observed.push(doubled()); });
      const writes = Array.from({ length: 12 }, (_, index) => input.set(index + 1));
      // Imported writes admit after their already-resolved ready() boundary;
      // allow that ingress microtask without awaiting any worker reply.
      if (sourceKind.startsWith("imported")) await Promise.resolve();
      assert.equal(doubled(), 24, "host tip must advance before acknowledgement");
      const afterBurst = observed.length;
      await writes[0];
      assert.equal(doubled(), 24, "an older acknowledgement cannot own a newer derived tip");
      await Promise.all(writes);
      assert.equal(doubled(), 24);
      assert.ok(observed.slice(afterBurst).every(value => value === 24),
        `older worker truth reached observers: ${JSON.stringify(observed.slice(afterBurst))}`);
      subscription.free();
    } finally {
      await signals?.terminate();
      await compatibility?.terminate();
      await cleanup();
      globalThis.Worker = previousWorker;
    }
  });
}

test("a write arriving while a committed acknowledgement is in flight retains the newer projection", async () => {
  const previousWorker = globalThis.Worker;
  let hold = false, releaseReadback, sawReadback;
  const blocked = new Promise(resolve => { sawReadback = resolve; });
  globalThis.Worker = class extends Worker {
    emit(event, message, ...args) {
      if (event === "message" && hold && message.value?.transaction) {
        hold = false;
        releaseReadback = () => super.emit(event, message, ...args);
        sawReadback();
        return true;
      }
      return super.emit(event, message, ...args);
    }
  };
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  let signals;
  try {
    signals = await createSignals({ deployment: "workerFirst" });
    const input = await signals.inputAsync(0);
    const output = await signals.computedAsync({ reads: [input.id],
      expr: { kind: "read", id: input.id }, identity: { kind: "exact" } });
    hold = true;
    const first = input.set(1);
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("worker did not reach the readback boundary")), 5000);
      blocked.then(() => { clearTimeout(timer); resolve(); });
    });
    const second = input.set(2);
    assert.equal(output(), 2);
    releaseReadback();
    releaseReadback = null;
    await first;
    assert.equal(output(), 2, "an in-flight read must recheck its basis before adoption");
    await second;
    assert.equal(output(), 2);
  } finally {
    releaseReadback?.();
    await signals?.terminate();
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});

test("a completed host dependency refresh cannot be replaced by an older acknowledgement", async () => {
  const previousWorker = globalThis.Worker;
  const blocked = Promise.withResolvers();
  let hold = false, release = () => {};
  globalThis.Worker = class extends Worker {
    emit(event, message, ...args) {
      if (event === "message" && hold && message.value?.transaction) {
        hold = false;
        release = () => super.emit(event, message, ...args);
        blocked.resolve();
        return true;
      }
      return super.emit(event, message, ...args);
    }
  };
  const { createSignals, hostCapabilityPlan, persistenceCapability, cleanup } =
    await loadSignalsModule({ rawSurface: "real" });
  let signals, writing;
  let current = { revision: 1 };
  try {
    signals = await createSignals({ hostCapabilities: hostCapabilityPlan({
      persistence: persistenceCapability({ source: { current: () => current } }),
    }) });
    const input = await signals.inputAsync(0);
    const output = await signals.computedAsync(() => signals.host.persistence.value().revision);
    hold = true;
    writing = input.set(1);
    await blocked.promise;
    current = { revision: 2 };
    await signals.host.persistence.commit();
    assert.equal(output(), 2);
    release();
    await writing;
    assert.equal(output(), 2, "cached readback predates the host refresh and must be discarded");
  } finally {
    release(); await writing?.catch(() => {}); await signals?.terminate();
    globalThis.Worker = previousWorker; await cleanup();
  }
});
