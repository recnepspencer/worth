import assert from "node:assert/strict";
import test from "node:test";

import { createDeferred } from "../runtime_fixture/async/deferred.mjs";
import { createRealLifecycleRuntime } from "../runtime_fixture/real_lifecycle_runtime.mjs";

function declareTwoFamilies(runtime, loads) {
  const { mod, resource } = runtime;
  const products = resource.detail({
    params: mod.resourceParams(),
    normalizeParams: ({ productId }) =>
      mod.resourceParamIdentity({ productId }, productId),
    load: ({ productId }) => loads.product(productId),
  });
  const orders = resource.detail({
    params: mod.resourceParams(),
    normalizeParams: ({ orderId }) =>
      mod.resourceParamIdentity({ orderId }, orderId),
    load: ({ orderId }) => loads.order(orderId),
  });
  return { products, orders };
}

test("resource.invalidateAll marks every materialized line across families and reports the breadth", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { products, orders } = declareTwoFamilies(runtime, {
      product: (productId) => ({ id: productId }),
      order: (orderId) => ({ id: orderId }),
    });
    const p1 = products.line({ productId: "p1" });
    const p2 = products.line({ productId: "p2" });
    const o1 = orders.line({ orderId: "o1" });

    assert.equal(runtime.resource.invalidateAll(), 3);

    for (const line of [p1, p2, o1]) {
      assert.deepEqual(line.freshness(), {
        kind: "stale",
        reason: "manualRuntimeInvalidateAll",
      });
      assert.equal(line.diagnostics().invalidationCount, 1);
      assert.equal(line.diagnostics().lastInvalidationCause, "manualRuntimeInvalidateAll");
      assert.equal(line.diagnostics().lastInvalidationScope, "runtimeAll");
      assert.equal(
        line.history().lifecycle.at(-1).event,
        "invalidated",
      );
    }
    assert.deepEqual(p1.value(), { id: "p1" });

    // A line materialized after the sweep is fresh: the sweep is an event,
    // not a standing rule.
    const p3 = products.line({ productId: "p3" });
    assert.deepEqual(p3.freshness(), { kind: "fresh" });
    assert.equal(p3.diagnostics().invalidationCount, 0);
  } finally {
    await runtime.cleanup();
  }
});

test("resource.invalidateAll skips released lines and counts only the lines it marked", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const { products, orders } = declareTwoFamilies(runtime, {
      product: (productId) => ({ id: productId }),
      order: (orderId) => ({ id: orderId }),
    });
    const p1 = products.line({ productId: "p1" });
    const p2 = products.line({ productId: "p2" });
    orders.line({ orderId: "o1" }).free();
    p2.free();

    assert.equal(runtime.resource.invalidateAll(), 1);
    assert.deepEqual(p1.freshness(), {
      kind: "stale",
      reason: "manualRuntimeInvalidateAll",
    });
    assert.equal(runtime.resource.invalidateAll(), 1);
    assert.equal(p1.diagnostics().invalidationCount, 2);
  } finally {
    await runtime.cleanup();
  }
});

test("resource.refreshAll reloads every materialized line across families with line.refresh semantics", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const loadCounts = { product: 0, order: 0 };
    const { products, orders } = declareTwoFamilies(runtime, {
      product: (productId) => ({ id: productId, version: ++loadCounts.product }),
      order: (orderId) => ({ id: orderId, version: ++loadCounts.order }),
    });
    const p1 = products.line({ productId: "p1" });
    const p2 = products.line({ productId: "p2" });
    const o1 = orders.line({ orderId: "o1" });
    assert.deepEqual([loadCounts.product, loadCounts.order], [2, 1]);

    assert.equal(runtime.resource.refreshAll(), 3);

    assert.deepEqual([loadCounts.product, loadCounts.order], [4, 2]);
    assert.deepEqual(p1.value(), { id: "p1", version: 3 });
    assert.deepEqual(p2.value(), { id: "p2", version: 4 });
    assert.deepEqual(o1.value(), { id: "o1", version: 2 });
    for (const line of [p1, p2, o1]) {
      assert.deepEqual(line.status(), { kind: "fulfilled", operation: "refresh" });
      assert.deepEqual(line.freshness(), { kind: "fresh" });
      assert.equal(line.diagnostics().lastOperation, "refresh");
    }
  } finally {
    await runtime.cleanup();
  }
});

test("resource.refreshAll supersedes a pending reload exactly like line.refresh", async () => {
  const runtime = await createRealLifecycleRuntime();
  try {
    const firstReload = createDeferred();
    let productLoads = 0;
    const { products } = declareTwoFamilies(runtime, {
      product: (productId) => {
        productLoads += 1;
        if (productLoads === 1) {
          return { id: productId, version: 1 };
        }
        if (productLoads === 2) {
          return firstReload.promise;
        }
        return { id: productId, version: productLoads };
      },
      order: (orderId) => ({ id: orderId }),
    });
    const p1 = products.line({ productId: "p1" });
    p1.refresh();
    assert.equal(p1.status().kind, "pending");

    assert.equal(runtime.resource.refreshAll(), 1);

    assert.deepEqual(p1.value(), { id: "p1", version: 3 });
    assert.ok(
      p1.history().lifecycle.some((entry) => entry.event === "superseded"),
      "the pending reload was superseded by refreshAll",
    );
    firstReload.resolve({ id: "p1", version: 2 });
    await Promise.resolve();
    // The superseded reload's late value does not overwrite the newer one.
    assert.deepEqual(p1.value(), { id: "p1", version: 3 });
  } finally {
    await runtime.cleanup();
  }
});
