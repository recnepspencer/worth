import assert from "node:assert/strict";
import test from "node:test";

import { createDeferred } from "../runtime_fixture/async/deferred.mjs";
import { createRealLifecycleRuntime } from "../runtime_fixture/real_lifecycle_runtime.mjs";

test("immediately stale policy keeps freshness honest across revalidation", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    let version = 0;
    const detail = resource.detail({
      params: mod.resourceParams(),
      policy: mod.resourcePolicyProfiles.immediatelyStale(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => ({ id: productId, version: ++version }),
    });

    const line = detail.line({ productId: "p1" });
    const revalidateStatus = line.revalidate();

    assert.deepEqual(revalidateStatus, {
      kind: "fulfilled",
      operation: "revalidate",
    });
    assert.deepEqual(line.freshness(), {
      kind: "stale",
      reason: "policyProfile",
    });
    assert.equal(line.diagnostics().policyProfileName, "immediatelyStale");
    assert.equal(line.diagnostics().request.auth.kind, "anonymous");
    assert.equal(line.diagnostics().lastOperation, "revalidate");
    assert.equal(line.diagnostics().revalidateCount, 1);
  } finally {
    await runtime.cleanup();
  }
});

test("refresh rejection preserves the visible value and records stale rejection truth", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    let callCount = 0;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: callCount };
        }
        throw new Error("refresh failed");
      },
    });

    const line = detail.line({ productId: "p1" });
    const refreshStatus = line.refresh();

    assert.deepEqual(line.value(), { id: "p1", version: 1 });
    assert.deepEqual(refreshStatus, {
      kind: "rejected",
      operation: "refresh",
      message: "refresh failed",
      continuity: "preservedVisibleValue",
    });
    assert.deepEqual(line.freshness(), {
      kind: "stale",
      reason: "refreshRejected",
    });
    assert.equal(line.diagnostics().rejectionCount, 1);
    assert.equal(line.diagnostics().preservedVisibleValueOnLastRejection, true);
    assert.equal(line.diagnostics().lastErrorMessage, "refresh failed");
  } finally {
    await runtime.cleanup();
  }
});

test("promise-backed refresh enters pending and preserves the visible value until settlement", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    let callCount = 0;
    const deferred = createDeferred();
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: 1 };
        }
        return deferred.promise;
      },
    });

    const line = detail.line({ productId: "p1" });
    const refreshStatus = line.refresh();

    assert.deepEqual(refreshStatus, {
      kind: "pending",
      operation: "refresh",
      continuity: "preservedVisibleValue",
    });
    assert.deepEqual(line.value(), { id: "p1", version: 1 });
    assert.deepEqual(line.status(), refreshStatus);
    assert.deepEqual(line.freshness(), {
      kind: "stale",
      reason: "refreshPending",
    });
    assert.equal(line.diagnostics().lastOutcome, "pending");
    assert.equal(line.diagnostics().pendingOperation, "refresh");

    deferred.resolve({ id: "p1", version: 2 });
    await deferred.promise;
    await Promise.resolve();

    assert.deepEqual(line.value(), { id: "p1", version: 2 });
    assert.deepEqual(line.status(), {
      kind: "fulfilled",
      operation: "refresh",
    });
    assert.equal(line.diagnostics().pendingOperation, null);
    assert.equal(line.diagnostics().refreshCount, 1);
    assert.equal(line.diagnostics().visibleValueVersion, 2);
  } finally {
    await runtime.cleanup();
  }
});

test("resource lines expose a first-class awaitSettlement lane for pending refresh truth", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    let callCount = 0;
    const deferred = createDeferred();
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: 1 };
        }
        return deferred.promise;
      },
    });

    const line = detail.line({ productId: "p1" });
    line.refresh();
    const settlementPromise = line.awaitSettlement();

    deferred.resolve({ id: "p1", version: 2 });
    const settlement = await settlementPromise;

    assert.equal(settlement.resultKind, "fulfilled");
    assert.deepEqual(settlement.status, {
      kind: "fulfilled",
      operation: "refresh",
    });
    assert.deepEqual(settlement.value, { id: "p1", version: 2 });
    assert.equal(settlement.summary.current.status.kind, "fulfilled");
  } finally {
    await runtime.cleanup();
  }
});

test("resource lines expose an execution object with settled() over the same lifecycle truth", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    let callCount = 0;
    const deferred = createDeferred();
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: 1 };
        }
        return deferred.promise;
      },
    });

    const line = detail.line({ productId: "p1" });
    line.refresh();
    const execution = line.execute();

    deferred.resolve({ id: "p1", version: 2 });
    const settlement = await execution.settled();

    assert.equal(settlement.resultKind, "fulfilled");
    assert.deepEqual(settlement.value, { id: "p1", version: 2 });
    assert.throws(() => line.value(), /cannot be used after line\.free/);
  } finally {
    await runtime.cleanup();
  }
});

test("resource families expose optionalLine and execute as first-class final-form lanes", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    let callCount = 0;
    const deferred = createDeferred();
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: 1 };
        }
        return deferred.promise;
      },
    });

    assert.equal(detail.optionalLine({ enabled: false }), null);

    const resident = detail.optionalLine({ productId: "p1" });
    assert.ok(resident);
    resident.refresh();

    const execution = detail.execute({ productId: "p1" });
    deferred.resolve({ id: "p1", version: 2 });
    const settlement = await execution.settled();

    assert.equal(settlement.resultKind, "fulfilled");
    assert.deepEqual(settlement.value, { id: "p1", version: 2 });
  } finally {
    await runtime.cleanup();
  }
});

test("resource line awaitSettlement resolves timedOut when the caller deadline elapses first and the line keeps settling", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    const deferred = createDeferred();
    let callCount = 0;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: 1 };
        }
        return deferred.promise;
      },
    });

    const line = detail.line({ productId: "p1" });
    line.refresh();

    const timedOut = await line.awaitSettlement({ timeoutMs: 1 });

    assert.equal(timedOut.resultKind, "timedOut");
    assert.deepEqual(timedOut.status, {
      kind: "timedOut",
      operation: "refresh",
      continuity: "preservedVisibleValue",
    });
    assert.equal(timedOut.confirmationKind, null);
    assert.equal(timedOut.mutationResponse, null);
    assert.deepEqual(timedOut.freshness, { kind: "stale", reason: "refreshPending" });
    // The deadline was the waiter's, not the line's: the line is still pending
    // and the visible value is intact.
    assert.deepEqual(line.status(), {
      kind: "pending",
      operation: "refresh",
      continuity: "preservedVisibleValue",
    });
    assert.deepEqual(line.value(), { id: "p1", version: 1 });

    deferred.resolve({ id: "p1", version: 2 });
    const settled = await line.awaitSettlement({ timeoutMs: 1_000 });
    assert.equal(settled.resultKind, "fulfilled");
    assert.deepEqual(settled.value, { id: "p1", version: 2 });
    assert.deepEqual(line.status(), { kind: "fulfilled", operation: "refresh" });
  } finally {
    await runtime.cleanup();
  }
});

test("resource line awaitSettlement rejects if the line is freed before settlement", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    const deferred = createDeferred();
    let callCount = 0;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        callCount += 1;
        if (callCount === 1) {
          return { id: productId, version: 1 };
        }
        return deferred.promise;
      },
    });

    const line = detail.line({ productId: "p1" });
    line.refresh();
    const settlement = line.awaitSettlement();
    line.free();

    await assert.rejects(
      () => settlement,
      /resource line awaitSettlement was cancelled because line\.free\(\) released the line/,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("superseded pending refresh completions do not overwrite newer reload truth", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    const firstDeferred = createDeferred();
    const secondDeferred = createDeferred();
    let reloadCount = 0;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => {
        if (reloadCount === 0) {
          reloadCount += 1;
          return { id: productId, version: 1 };
        }
        reloadCount += 1;
        return reloadCount === 2 ? firstDeferred.promise : secondDeferred.promise;
      },
    });

    const line = detail.line({ productId: "p1" });
    line.refresh();
    const secondRefreshStatus = line.refresh();

    assert.deepEqual(secondRefreshStatus, {
      kind: "pending",
      operation: "refresh",
      continuity: "preservedVisibleValue",
    });
    assert.equal(line.diagnostics().supersessionCount, 1);
    assert.equal(line.diagnostics().lastSupersededOperation, "refresh");

    firstDeferred.resolve({ id: "p1", version: 2 });
    await firstDeferred.promise;
    await Promise.resolve();
    assert.deepEqual(line.value(), { id: "p1", version: 1 });
    assert.equal(line.status().kind, "pending");

    secondDeferred.resolve({ id: "p1", version: 3 });
    await secondDeferred.promise;
    await Promise.resolve();

    assert.deepEqual(line.value(), { id: "p1", version: 3 });
    assert.deepEqual(line.status(), {
      kind: "fulfilled",
      operation: "refresh",
    });
  } finally {
    await runtime.cleanup();
  }
});


test("resource lines can be explicitly freed and rematerialized", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => ({ id: productId }),
    });

    const first = detail.line({ productId: "p1" });
    const firstRuntimeLineId = first.descriptor().runtimeLineId;
    first.free();

    const second = detail.line({ productId: "p1" });

    assert.notEqual(first, second);
    assert.notEqual(second.descriptor().runtimeLineId, firstRuntimeLineId);
  } finally {
    await runtime.cleanup();
  }
});

test("freed resource lines reject further operations", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => ({ id: productId }),
    });

    const line = detail.line({ productId: "p1" });
    line.free();

    assert.throws(() => line.value(), /cannot be used after line\.free/);
    assert.throws(() => line.status(), /cannot be used after line\.free/);
    assert.throws(() => line.freshness(), /cannot be used after line\.free/);
    assert.throws(() => line.request(), /cannot be used after line\.free/);
    assert.throws(() => line.download(), /cannot be used after line\.free/);
    assert.throws(() => line.diagnostics(), /cannot be used after line\.free/);
    assert.throws(() => line.invalidate(), /cannot be used after line\.free/);
    assert.throws(() => line.refresh(), /cannot be used after line\.free/);
    assert.throws(() => line.revalidate(), /cannot be used after line\.free/);
    assert.throws(
      () => line.view((value) => value.id),
      /cannot be used after line\.free/,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("line free disposes line-scoped views with the owning lifecycle", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { mod, resource } = runtime;
    const detail = resource.detail({
      params: mod.resourceParams(),
      normalizeParams: ({ productId }) =>
        mod.resourceParamIdentity({ productId }, productId),
      load: ({ productId }) => ({ id: productId, label: productId }),
    });

    const line = detail.line({ productId: "p1" });
    const view = line.view((value) => value.label);
    line.free();

    assert.throws(() => view(), /resource line view cannot be used after line\.free/);
  } finally {
    await runtime.cleanup();
  }
});
