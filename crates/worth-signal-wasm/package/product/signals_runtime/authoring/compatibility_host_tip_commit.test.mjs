import assert from "node:assert/strict";
import test from "node:test";

import {
  flushMicrotasks,
  withTipNotifyWorld,
} from "../entrypoint/construction/worker_first_tip_react_notify_support.mjs";
import { loadSignalsModule } from "../module_loading/load_signals_module.mjs";

// Main-thread compatibility roots expose the same host tip surface as
// worker-first roots (host_tip_surface.d.ts), so line bindings, forms, and
// any authored publisher use one tip-batch path on every deployment.

async function withCompatibilitySignals(run) {
  const signalsModule = await loadSignalsModule({ rawSurface: "real" });
  let signals = null;
  try {
    signals = await signalsModule.createSignals({ deployment: "mainThreadCompatibility" });
    await run(signals);
  } finally {
    if (signals) {
      await signals.terminate();
    }
    await signalsModule.cleanup();
  }
}

test("compatibility commitHostTipAndNotify applies every write in one transaction and projects dependents synchronously", async () => {
  await withCompatibilitySignals(async (signals) => {
    const first = signals.input(1, { debugName: "compatTip.first" });
    const second = signals.input(10, { debugName: "compatTip.second" });
    const sum = signals.computed(() => first() + second(), { debugName: "compatTip.sum" });
    assert.equal(sum(), 11);

    let sumNotices = 0;
    const watch = signals.watch(sum, () => {
      sumNotices += 1;
    });

    const result = signals.commitHostTipAndNotify([
      { id: first.id, value: 2 },
      { id: second.id, value: 20 },
    ]);

    // Both tips are visible before any microtask: the raw transaction
    // committed them together and projected the dependent inside it.
    assert.equal(first(), 2);
    assert.equal(second(), 20);
    assert.equal(sum(), 22);
    assert.deepEqual(result.changedIds, [first.id, second.id]);
    assert.deepEqual(result.projectedReadableIds, []);
    assert.deepEqual([...result.epochById], [[first.id, 1], [second.id, 1]]);
    assert.ok(Object.isFrozen(result));
    assert.ok(Object.isFrozen(result.changedIds));

    await flushMicrotasks();
    assert.equal(sumNotices, 1, "one batch commit notifies a dependent watcher once");

    const again = signals.commitHostTipAndNotify([{ id: first.id, value: 3 }]);
    assert.deepEqual([...again.epochById], [[first.id, 2]]);
    assert.equal(sum(), 23);
    await flushMicrotasks();
    assert.equal(sumNotices, 2);
    watch.free();
  });
});

test("compatibility tip epochs: an older rollback does not clobber a newer tip, and a current rollback restores the previous value", async () => {
  await withCompatibilitySignals(async (signals) => {
    const count = signals.input(0, { debugName: "compatTip.rollbackEpoch" });

    const tipA = signals.commitHostTipAndNotify([{ id: count.id, value: 10 }]);
    const tipB = signals.commitHostTipAndNotify([{ id: count.id, value: 20 }]);
    assert.equal(count(), 20);

    tipA.rollback();
    assert.equal(count(), 20, "older tip rollback must not clobber the newer tip epoch");

    tipB.rollback();
    assert.equal(count(), 10, "the current tip rolls back to the value it replaced");
    // Rollback is itself a committed tip: the epoch moved on, so a second
    // rollback of the same result is a no-op rather than a second restore.
    signals.commitHostTipAndNotify([{ id: count.id, value: 30 }]);
    tipB.rollback();
    assert.equal(count(), 30);
  });
});

test("compatibility applyCommittedTipWorkerBatch validates epochs: a stale batch is a loud TypeError, a current one resolves its applied ids", async () => {
  await withCompatibilitySignals(async (signals) => {
    const count = signals.input(0, { debugName: "compatTip.applyEpoch" });
    const tip = signals.commitHostTipAndNotify([{ id: count.id, value: 1 }]);
    const epochAtWrite = tip.epochById.get(count.id);

    const applied = await signals.applyCommittedTipWorkerBatch([
      { id: count.id, value: 1, epochAtWrite },
    ]);
    assert.deepEqual(applied, { appliedIds: [count.id] });
    assert.equal(count(), 1, "apply must not re-tip");

    signals.commitHostTipAndNotify([{ id: count.id, value: 2 }]);
    assert.throws(
      () => signals.applyCommittedTipWorkerBatch([{ id: count.id, value: 1, epochAtWrite }]),
      {
        name: "TypeError",
        message: `applyCommittedTipWorkerBatch received a stale batch for ${JSON.stringify(count.id)}: stamped epoch 1, current epoch 2`,
      },
    );
    assert.equal(count(), 2, "a stale batch changes nothing");

    assert.equal(await signals.applyCommittedTipWorkerBatch([]), null);
    assert.throws(
      () => signals.applyCommittedTipWorkerBatch([{ value: 1 }]),
      /applyCommittedTipWorkerBatch requires \{ id, value \} tip writes/,
    );
  });
});

test("compatibility tip surface admits only live input ids of this runtime and collapses duplicate ids like worker-first", async () => {
  await withCompatibilitySignals(async (signals) => {
    const count = signals.input(0, { debugName: "compatTip.admission" });
    const derived = signals.computed(() => count() * 2, { debugName: "compatTip.admissionDerived" });

    assert.throws(
      () => signals.commitHostTipAndNotify([{ id: "not-an-input", value: 1 }]),
      /commitHostTipAndNotify requires input ids authored on this runtime; "not-an-input" is not a live input handle/,
    );
    assert.throws(
      () => signals.commitHostTipAndNotify([{ id: derived.id, value: 1 }]),
      /is not a live input handle of this runtime/,
    );
    assert.throws(
      () => signals.commitHostTipAndNotify("nope"),
      /commitHostTipAndNotify requires an array of \{ id, value \} tip writes/,
    );
    assert.throws(
      () => signals.publishAuthoredTipProjection([derived.id]),
      /publishAuthoredTipProjection requires input ids authored on this runtime/,
    );

    // Writes without a string id are dropped; repeated ids keep the last value.
    const result = signals.commitHostTipAndNotify([
      null,
      { value: 99 },
      { id: 42, value: 99 },
      { id: count.id, value: 5 },
      { id: count.id, value: 7 },
    ]);
    assert.deepEqual(result.changedIds, [count.id]);
    assert.equal(count(), 7);

    const empty = signals.commitHostTipAndNotify([]);
    assert.deepEqual(empty.changedIds, []);
    assert.equal(empty.epochById.size, 0);
    empty.rollback();

    // A freed input leaves the registry, so a late tip for it is refused
    // instead of writing into a dead handle.
    const freedId = count.id;
    count.free();
    assert.throws(
      () => signals.commitHostTipAndNotify([{ id: freedId, value: 1 }]),
      /is not a live input handle of this runtime/,
    );
  });
});

test("compatibility settleAuthoredWork resolves immediately, counts its invocations, and scopes share the root surface", async () => {
  await withCompatibilitySignals(async (signals) => {
    assert.equal(signals.authoredSettleInvocationCount(), 0);
    await signals.settleAuthoredWork();
    assert.equal(signals.authoredSettleInvocationCount(), 1);

    const scope = signals.scope("compatTip.scope");
    const count = scope.input(0, { debugName: "compatTip.scoped" });
    const result = scope.commitHostTipAndNotify([{ id: count.id, value: 4 }]);
    assert.equal(count(), 4);
    assert.deepEqual([...result.epochById], [[count.id, 1]]);
    await scope.settleAuthoredWork();
    assert.equal(scope.authoredSettleInvocationCount(), 2);
    assert.equal(signals.authoredSettleInvocationCount(), 2);
    scope.publishAuthoredTipProjection([count.id]);
    assert.deepEqual(
      await scope.applyCommittedTipWorkerBatch([{ id: count.id, value: 4, epochAtWrite: 1 }]),
      { appliedIds: [count.id] },
    );
  });
});

test("worker-first and compatibility roots return the same host tip commit shape for the same writes", async () => {
  await withTipNotifyWorld(async ({ signals: workerFirst, openCompatibility }) => {
    const compatibility = await openCompatibility();
    const workerInput = workerFirst.input(0, { debugName: "tipParity.count" });
    const compatInput = compatibility.input(0, { debugName: "tipParity.count" });

    const workerResult = workerFirst.commitHostTipAndNotify([{ id: workerInput.id, value: 5 }]);
    const compatResult = compatibility.commitHostTipAndNotify([{ id: compatInput.id, value: 5 }]);

    assert.deepEqual(
      Object.keys(compatResult).sort(),
      Object.keys(workerResult).sort(),
    );
    assert.deepEqual(compatResult.changedIds, [compatInput.id]);
    assert.deepEqual(workerResult.changedIds, [workerInput.id]);
    assert.equal(typeof compatResult.epochById.get(compatInput.id), "number");
    assert.equal(typeof workerResult.epochById.get(workerInput.id), "number");
    assert.equal(typeof compatResult.rollback, "function");
    assert.equal(typeof workerResult.rollback, "function");
    for (const member of [
      "commitHostTipAndNotify",
      "applyCommittedTipWorkerBatch",
      "publishAuthoredTipProjection",
      "settleAuthoredWork",
      "authoredSettleInvocationCount",
    ]) {
      assert.equal(typeof compatibility[member], "function", `compatibility ${member}`);
      assert.equal(typeof workerFirst[member], "function", `worker-first ${member}`);
    }
    await workerFirst.settleAuthoredWork();
  });
});
