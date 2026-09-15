import assert from "node:assert/strict";
import test from "node:test";

import { createRealLifecycleRuntime } from "../../runtime_fixture/real_lifecycle_runtime.mjs";
import { createDeferred } from "../../runtime_fixture/async/deferred.mjs";

function declareProduct(runtime, load) {
  const { mod, resource } = runtime;
  return resource.detail({
    params: mod.resourceParams(),
    normalizeParams: ({ productId }) => mod.resourceParamIdentity({ productId }, productId),
    load,
  });
}

test("a loaded Date is committed as its ISO string instead of being stored as an empty object", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const products = declareProduct(runtime, ({ productId }) => ({
      id: productId,
      shippedAt: new Date("2026-09-15T12:00:00.000Z"),
      tags: ["a", undefined, "b"],
      note: undefined,
      ratio: Number.POSITIVE_INFINITY,
    }));
    const line = products.line({ productId: "p1" });
    assert.equal(line.status().kind, "fulfilled");
    // JSON form: toJSON honored, undefined members omitted, undefined array
    // elements and non-finite numbers written as null.
    assert.deepEqual(line.value(), {
      id: "p1",
      shippedAt: "2026-09-15T12:00:00.000Z",
      tags: ["a", null, "b"],
      ratio: null,
    });
    assert.equal(Object.hasOwn(line.value(), "note"), false);
  } finally {
    await runtime.cleanup();
  }
});

test("a synchronous load whose value JSON cannot represent settles rejected instead of throwing out of line()", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const products = declareProduct(runtime, ({ productId }) => ({
      id: productId,
      sizes: new Set(["s", "m"]),
    }));
    const line = products.line({ productId: "p1" });
    assert.deepEqual(line.status(), {
      kind: "rejected",
      operation: "initialLoad",
      message:
        "resource line value cannot represent Set at $value.sizes: "
        + "JSON serializes it without its contents, so the line could not hold it faithfully",
      continuity: "noVisibleValueYet",
    });
    assert.equal(line.value(), null);
    assert.deepEqual(
      line.history().lifecycle.map((entry) => [entry.event, entry.status.kind]),
      [["materialized", "rejected"]],
    );
  } finally {
    await runtime.cleanup();
  }
});

test("a Map is refused by name rather than silently committed as a plain object", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const products = declareProduct(runtime, ({ productId }) => ({
      id: productId,
      attributes: new Map([["color", "red"]]),
    }));
    const line = products.line({ productId: "p1" });
    assert.equal(line.status().kind, "rejected");
    assert.match(line.status().message, /cannot represent Map at \$value\.attributes/);
  } finally {
    await runtime.cleanup();
  }
});

test("a function member is refused with its path instead of surfacing as a runtime boundary error", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const products = declareProduct(runtime, ({ productId }) => ({
      id: productId,
      format() {
        return productId;
      },
    }));
    const line = products.line({ productId: "p1" });
    assert.equal(line.status().kind, "rejected");
    assert.equal(
      line.status().message,
      "resource line value cannot represent function at $value.format: JSON has no form for it",
    );
  } finally {
    await runtime.cleanup();
  }
});

test("a refresh whose value JSON cannot represent settles rejected and keeps the previous visible value", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    let loads = 0;
    const products = declareProduct(runtime, ({ productId }) => {
      loads += 1;
      if (loads === 1) {
        return { id: productId, version: 1 };
      }
      return { id: productId, version: 2, sizes: new Set(["s"]) };
    });
    const line = products.line({ productId: "p1" });
    assert.deepEqual(line.value(), { id: "p1", version: 1 });

    line.refresh();

    assert.deepEqual(line.status(), {
      kind: "rejected",
      operation: "refresh",
      message:
        "resource line value cannot represent Set at $value.sizes: "
        + "JSON serializes it without its contents, so the line could not hold it faithfully",
      continuity: "preservedVisibleValue",
    });
    assert.deepEqual(line.value(), { id: "p1", version: 1 });
    assert.deepEqual(
      line.history().lifecycle.map((entry) => entry.event),
      ["materialized", "rejected"],
    );
  } finally {
    await runtime.cleanup();
  }
});

test("an asynchronous load rejected with a non-Error object still reports that object's message", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const deferred = createDeferred();
    const products = declareProduct(runtime, () => deferred.promise);
    const line = products.line({ productId: "p1" });
    assert.equal(line.status().kind, "pending");

    deferred.reject({ code: "hostRefused", message: "the host refused this load" });
    const settled = await line.awaitSettlement({ timeoutMs: 1_000 });

    assert.equal(settled.resultKind, "rejected");
    assert.equal(settled.status.message, "the host refused this load");
    assert.equal(line.status().message, "the host refused this load");
  } finally {
    await runtime.cleanup();
  }
});
