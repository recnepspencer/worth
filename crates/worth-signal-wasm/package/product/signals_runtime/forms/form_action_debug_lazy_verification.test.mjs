import assert from "node:assert/strict";
import test from "node:test";

import { loadSignalsModule } from "../module_loading/load_signals_module.mjs";
import { createGraphOperationalRuntime } from "../runtime_fixture/graph_operational_runtime.mjs";

function declareApprovalForm(signals) {
  return signals.form({
    source: { title: "Ship docs", approved: false },
    fields: ({ field }) => ({
      title: field("title"),
      approved: field("approved"),
    }),
    actions: ({ action }) => ({
      approve: action("approve", {
        patchPolicy: "allowEmpty",
        hostEffect: "workflow.approve",
        idempotency: "deny",
      }),
      archive: action("archive", {
        patchPolicy: "allowEmpty",
        hostEffect: "workflow.archive",
        idempotency: "deny",
      }),
    }),
  });
}

// Counts `verification()` calls made through the controller while forwarding
// everything else to it unchanged. Controllers are frozen, so the wrapper is
// a spread copy rather than a Proxy (which cannot override a frozen member).
function countVerificationCalls(form) {
  const counter = { calls: 0 };
  const counting = Object.freeze({
    ...form,
    verification: (...args) => {
      counter.calls += 1;
      return form.verification(...args);
    },
  });
  return { counting, counter };
}

test("action debug reads build the verification package only when verification or digest is read, once", async () => {
  const { wrapSignals, importProductModule, cleanup } = await loadSignalsModule();
  try {
    const { readActionDebug } = await importProductModule("forms/actions/debug.js");
    const signals = wrapSignals(createGraphOperationalRuntime());
    const form = declareApprovalForm(signals);
    for (let index = 0; index < 5; index += 1) {
      form.attemptAction("approve");
    }
    const { counting, counter } = countVerificationCalls(form);

    const debug = readActionDebug(counting, "approve");
    assert.equal(counter.calls, 0);
    assert.equal(debug.action, "approve");
    assert.equal(debug.pending, false);
    assert.equal(debug.latestExecution, null);
    assert.equal(debug.attempts.length, 5);
    assert.equal(counter.calls, 0);

    const first = debug.verification;
    const second = debug.verification;
    assert.equal(counter.calls, 1);
    assert.equal(first, second);
    assert.equal(first.packageDigest, form.verification().packageDigest);
    assert.equal(first.actionPlanDigest, debug.plan.planDigest);

    assert.equal(typeof debug.digest, "string");
    assert.equal(debug.digest, debug.digest);
    assert.equal(counter.calls, 1);

    // Reading digest first (without touching verification) also builds the
    // package exactly once.
    const digestFirst = readActionDebug(counting, "approve");
    assert.equal(counter.calls, 1);
    assert.equal(digestFirst.digest, debug.digest);
    assert.equal(counter.calls, 2);
    assert.deepEqual(digestFirst.verification, first);
    assert.equal(counter.calls, 2);
  } finally {
    await cleanup();
  }
});

test("debug reports stay frozen, enumerable, and equal to an eager read of the same state", async () => {
  const { wrapSignals, cleanup } = await loadSignalsModule();
  try {
    const signals = wrapSignals(createGraphOperationalRuntime());
    const form = declareApprovalForm(signals);
    form.attemptAction("approve");

    const debug = form.debugAction("approve");
    assert.ok(Object.isFrozen(debug));
    assert.ok(Object.keys(debug).includes("verification"));
    assert.ok(Object.keys(debug).includes("digest"));
    assert.equal(debug.verification.packageDigest, form.verification().packageDigest);
    assert.equal(
      debug.verification.actionLifecycleDigest,
      form.verification().digests.actionLifecycleDigest,
    );
    assert.equal(
      debug.verification.actionExecutionLifecycleDigest,
      form.verification().digests.actionExecutionLifecycleDigest,
    );
    assert.equal(debug.digest, form.debugAction("approve").digest);

    form.attemptAction("archive");
    assert.notEqual(form.debugAction("approve").digest, debug.digest);
    assert.equal(debug.digest, debug.digest);
  } finally {
    await cleanup();
  }
});
