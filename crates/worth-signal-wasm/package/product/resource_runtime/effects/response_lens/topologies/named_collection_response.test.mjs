import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("named collection responses lower item replacement through named collection loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "named-collection");
    let fullCollectionReplacementCount = 0;
    let singleCollectionItemReplacementCount = 0;
    const response = createTaskNamedResponse(signals, {
      replaceCollections(value, collections) {
        fullCollectionReplacementCount += 1;
        return { ...value, collections };
      },
      replaceCollectionItem(value, collectionId, itemId, nextItem) {
        singleCollectionItemReplacementCount += 1;
        return {
          ...value,
          collections: replaceNamedCollectionItem(
            value.collections,
            collectionId,
            itemId,
            nextItem,
          ),
        };
      },
    });
    assert.equal(response.lensProof.topology, "namedCollection");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "namedCollection" && row.patchScope === "item",
    ), true);

    const tasks = createTaskNamedApi(signals, response, "/named", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", collection: "backlog", title: "Replaced" },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.equal(readTask(line.value(), "backlog", "task:1").title, "Replaced");
    assert.equal(fullCollectionReplacementCount, 0);
    assert.equal(singleCollectionItemReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "namedCollection",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.named<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "namedCollection");
    assert.equal(itemEffect.locusProof.locus, "namedCollection");
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "collection-key-item-id",
      lookupBreadth: 1,
      traversal: "single-named-collection",
      traversalBreadth: 1,
      reconstruction: "replaceCollectionItem",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-named",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", collection: "backlog", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(readTask(line.value(), "backlog", "task:1").title, "Delivered");
    assert.equal(deliveryEffect.locus.kind, "namedCollection");
    assert.equal(deliveryEffect.locusProof.locus, "namedCollection");
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
    assert.equal(singleCollectionItemReplacementCount, 2);

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.equal(readTask(line.value(), "backlog", "task:1").title, "Aspect");
    assert.equal(singleCollectionItemReplacementCount, 5);
  } finally {
    await runtime.cleanup();
  }
});

test("named collection broad replacements preserve named topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(signals);
    const tasks = createTaskNamedApi(signals, response, "/named-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      collections: {
        active: [{ id: "task:2", collection: "active", title: "Broad" }],
      },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "namedCollection");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-named-collection-record",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceCollections",
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

test("multiple collection responses expose the named collection topology lane", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(
      signals,
      {},
      signals.resource.response.multiple,
    );
    const tasks = createTaskNamedApi(signals, response, "/multiple");
    const line = tasks.line({});

    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", collection: "backlog", title: "Multiple" },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.equal(readTask(line.value(), "backlog", "task:1").title, "Multiple");
    assert.equal(effect.locus.kind, "namedCollection");
    assert.equal(effect.locusProof.lensSource, "resource.response.multiple<T>()(...)");
    assert.equal(effect.locusProof.topology, "namedCollection");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "collection-key-item-id",
      lookupBreadth: 1,
      traversal: "single-named-collection",
      traversalBreadth: 1,
      reconstruction: "replaceCollectionItem",
      reconstructionBreadth: 1,
    });
  } finally {
    await runtime.cleanup();
  }
});

test("named collection broad replacements deny corrupt collection topology before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(signals);
    const tasks = createTaskNamedApi(signals, response, "/named-broad-denial");
    const line = tasks.line({});

    assertNamedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.replace({
      collections: {
        backlog: [{ id: "task:1", collection: "active", title: "Corrupt" }],
      },
    })), /collection key "backlog" to match collectionId\(item\) "active"/);
  } finally {
    await runtime.cleanup();
  }
});

test("named collection responses deny invalid collection lookup before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(signals, {
      collectionForItem: () => "",
    });
    const tasks = createTaskNamedApi(signals, response, "/bad-named-lookup");
    const line = tasks.line({});

    assertNamedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", collection: "backlog", title: "Replaced" },
    })), /collectionForItem\(itemId\).*non-empty collection id/);
  } finally {
    await runtime.cleanup();
  }
});

test("named collection responses deny malformed collections before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(signals, {
      collections: () => ({
        backlog: { id: "task:1", collection: "backlog", title: "First" },
      }),
    });
    const tasks = createTaskNamedApi(signals, response, "/bad-named");
    const line = tasks.line({});

    assertNamedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", collection: "backlog", title: "Replaced" },
    })), /collection "backlog" to be an array/);
  } finally {
    await runtime.cleanup();
  }
});

test("named collection responses deny duplicate item ids before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(signals, {
      collections: () => ({
        backlog: [
          { id: "task:1", collection: "backlog", title: "First" },
          { id: "task:1", collection: "backlog", title: "Duplicate" },
        ],
      }),
    });
    const tasks = createTaskNamedApi(signals, response, "/duplicate-named");
    const line = tasks.line({});

    assertNamedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", collection: "backlog", title: "Replaced" },
    })), /duplicated named collection item id "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

test("named replaceCollectionItem must preserve item and collection identity", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskNamedResponse(signals, {
      replaceCollectionItem(value, collectionId, itemId, nextItem) {
        return {
          ...value,
          collections: {
            [collectionId]: [{ ...nextItem, id: itemId, collection: "active" }],
          },
        };
      },
    });
    const tasks = createTaskNamedApi(signals, response, "/corrupt-named");
    const line = tasks.line({});

    assertNamedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", collection: "backlog", title: "Replaced" },
    })), /collection key "backlog" to match collectionId\(item\) "active"/);
  } finally {
    await runtime.cleanup();
  }
});

function assertNamedPatchDeniedWithoutSideEffects(line, patchAction, errorPattern) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, errorPattern);
  assert.deepEqual(line.value(), beforeValue);
  assert.equal(line.diagnostics().lastEffect, beforeEffect);
}

function createTaskNamedResponse(
  signals,
  overrides = {},
  responseFactory = signals.resource.response.named,
) {
  return responseFactory()({
    itemId: (task) => task.id,
    collectionId: (task) => task.collection,
    collectionForItem: overrides.collectionForItem ?? (() => "backlog"),
    collections: overrides.collections ?? ((value) => value.collections),
    replaceCollections: overrides.replaceCollections ?? (
      (value, collections) => ({ ...value, collections })
    ),
    replaceCollectionItem: overrides.replaceCollectionItem ?? (
      (value, collectionId, itemId, nextItem) => ({
        ...value,
        collections: replaceNamedCollectionItem(
          value.collections,
          collectionId,
          itemId,
          nextItem,
        ),
      })
    ),
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskNamedApi(signals, response, url, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({
      load: () => ({
        collections: {
          backlog: [{ id: "task:1", collection: "backlog", title: "First" }],
          active: [],
        },
      }),
    });
}

function replaceNamedCollectionItem(collections, collectionId, itemId, nextItem) {
  return Object.fromEntries(
    Object.entries(collections).map(([key, items]) => [
      key,
      key === collectionId
        ? items.map((item) => item.id === itemId ? nextItem : item)
        : items,
    ]),
  );
}

function readTask(value, collectionId, itemId) {
  return value.collections[collectionId].find((task) => task.id === itemId);
}
