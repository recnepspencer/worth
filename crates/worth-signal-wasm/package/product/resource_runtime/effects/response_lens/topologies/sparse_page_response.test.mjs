import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("sparse page responses lower item replacement through loaded page loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "sparse-page");
    let fullPageReplacementCount = 0;
    let singlePageItemReplacementCount = 0;
    const response = createTaskSparseResponse(signals, {
      replacePages(value, pages) {
        fullPageReplacementCount += 1;
        return { ...value, pages };
      },
      replacePageItem(value, pageId, itemId, nextItem) {
        singlePageItemReplacementCount += 1;
        return {
          ...value,
          pages: replaceSparsePageItem(value.pages, pageId, itemId, nextItem),
        };
      },
    });
    assert.equal(response.lensProof.topology, "sparsePage");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "sparsePage" && row.patchScope === "item",
    ), true);

    const tasks = createTaskSparseApi(signals, response, "/sparse", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", page: "page-a", title: "Replaced" },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.equal(readTask(line.value(), "page-a", "task:1").title, "Replaced");
    assert.equal(fullPageReplacementCount, 0);
    assert.equal(singlePageItemReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "sparsePage",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.sparse<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "sparsePage");
    assert.equal(itemEffect.locusProof.locus, "sparsePage");
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "sparse-page-item-id",
      lookupBreadth: 1,
      traversal: "loaded-page-chunk",
      traversalBreadth: 1,
      reconstruction: "replacePageItem",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-sparse",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", page: "page-a", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(readTask(line.value(), "page-a", "task:1").title, "Delivered");
    assert.equal(deliveryEffect.locus.kind, "sparsePage");
    assert.equal(deliveryEffect.locusProof.locus, "sparsePage");
    assert.deepEqual(deliveryEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.deepEqual(deliveryEffect.optimistic.confirmation, {
      kind: "consumedCanonicalServerTruth",
      previousEffectId: itemEffect.effectId,
      previousPlanId: itemEffect.plan.planId,
      previousBranchId: itemEffect.optimistic.branchId,
      previousSnapshotId: itemEffect.optimistic.snapshotId,
      locusMatches: true,
      valueChanged: true,
      detail:
        "server delivery consumed canonical server truth after a pending speculative resource effect",
    });
    assert.equal(singlePageItemReplacementCount, 2);

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.equal(readTask(line.value(), "page-a", "task:1").title, "Aspect");
    assert.equal(singlePageItemReplacementCount, 5);
  } finally {
    await runtime.cleanup();
  }
});

test("sparse page broad replacements preserve sparse topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskSparseResponse(signals);
    const tasks = createTaskSparseApi(signals, response, "/sparse-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      pages: {
        "page-b": [{ id: "task:2", page: "page-b", title: "Broad" }],
      },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "sparsePage");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-sparse-pages",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replacePages",
      reconstructionBreadth: 1,
    });
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      effect.locusProof,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("sparse page broad replacements deny corrupt page topology before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskSparseResponse(signals);
    const tasks = createTaskSparseApi(signals, response, "/sparse-broad-denial");
    const line = tasks.line({});

    assertSparsePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.replace({
      pages: {
        "page-a": [{ id: "task:1", page: "page-b", title: "Corrupt" }],
      },
    })), /page key "page-a" to match pageId\(item\) "page-b"/);
  } finally {
    await runtime.cleanup();
  }
});

test("sparse page responses deny unloaded page item patches before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskSparseResponse(signals, {
      pageForItem: () => "page-b",
    });
    const tasks = createTaskSparseApi(signals, response, "/sparse-unloaded");
    const line = tasks.line({});

    assertSparsePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:2",
      nextItem: { id: "task:2", page: "page-b", title: "Unloaded" },
    })), /could not find loaded sparse page itemId "task:2"/);
  } finally {
    await runtime.cleanup();
  }
});

test("sparse page responses deny malformed loaded pages before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskSparseResponse(signals, {
      pages: () => ({ "page-a": { id: "task:1", page: "page-a", title: "First" } }),
    });
    const tasks = createTaskSparseApi(signals, response, "/bad-sparse");
    const line = tasks.line({});

    assertSparsePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", page: "page-a", title: "Replaced" },
    })), /loaded page "page-a" to be an array/);
  } finally {
    await runtime.cleanup();
  }
});

test("sparse page responses deny duplicate item ids inside one loaded page", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskSparseResponse(signals, {
      pages: () => ({
        "page-a": [
          { id: "task:1", page: "page-a", title: "First" },
          { id: "task:1", page: "page-a", title: "Duplicate" },
        ],
      }),
    });
    const tasks = createTaskSparseApi(signals, response, "/duplicate-sparse");
    const line = tasks.line({});

    assertSparsePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", page: "page-a", title: "Replaced" },
    })), /duplicated sparse page item id "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

test("sparse replacePageItem must preserve item and loaded page identity", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskSparseResponse(signals, {
      replacePageItem(value, pageId, itemId, nextItem) {
        return {
          ...value,
          pages: {
            [pageId]: [{ ...nextItem, id: itemId, page: "page-b" }],
          },
        };
      },
    });
    const tasks = createTaskSparseApi(signals, response, "/corrupt-sparse");
    const line = tasks.line({});

    assertSparsePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", page: "page-a", title: "Replaced" },
    })), /page key "page-a" to match pageId\(item\) "page-b"/);
  } finally {
    await runtime.cleanup();
  }
});

function assertSparsePatchDeniedWithoutSideEffects(line, patchAction, errorPattern) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, errorPattern);
  assert.deepEqual(line.value(), beforeValue);
  assert.equal(line.diagnostics().lastEffect, beforeEffect);
}

function createTaskSparseResponse(signals, overrides = {}) {
  return signals.resource.response.sparse()({
    itemId: (task) => task.id,
    pageId: (task) => task.page,
    pageForItem: overrides.pageForItem ?? (() => "page-a"),
    pages: overrides.pages ?? ((value) => value.pages),
    replacePages: overrides.replacePages ?? (
      (value, pages) => ({ ...value, pages })
    ),
    replacePageItem: overrides.replacePageItem ?? (
      (value, pageId, itemId, nextItem) => ({
        ...value,
        pages: replaceSparsePageItem(value.pages, pageId, itemId, nextItem),
      })
    ),
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskSparseApi(signals, response, url, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({
      load: () => ({
        pages: {
          "page-a": [{ id: "task:1", page: "page-a", title: "First" }],
        },
      }),
    });
}

function replaceSparsePageItem(pages, pageId, itemId, nextItem) {
  return Object.fromEntries(
    Object.entries(pages).map(([key, items]) => [
      key,
      key === pageId
        ? items.map((item) => item.id === itemId ? nextItem : item)
        : items,
    ]),
  );
}

function readTask(value, pageId, itemId) {
  return value.pages[pageId].find((task) => task.id === itemId);
}
