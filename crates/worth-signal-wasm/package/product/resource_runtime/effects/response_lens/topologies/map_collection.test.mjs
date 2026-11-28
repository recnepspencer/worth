import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("map-backed responses lower item replacement through map collection loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "map-collection");
    let fullMapReplacementCount = 0;
    let singleEntryReplacementCount = 0;
    const response = createTaskMapResponse(signals, {
      replaceEntries(value, taskMap) {
        fullMapReplacementCount += 1;
        return { ...value, taskMapEntries: [...taskMap] };
      },
      replaceEntry(value, itemId, nextItem) {
        singleEntryReplacementCount += 1;
        return {
          ...value,
          taskMapEntries: replaceMapEntry(
            value.taskMapEntries,
            itemId,
            nextItem,
          ),
        };
      },
    });
    assert.equal(response.lensProof.topology, "mapCollection");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "mapCollection" && row.patchScope === "item",
    ), true);

    const tasks = createTaskMapApi(signals, response, "/map-collection", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.deepEqual(readTask(line.value(), "task:1"), {
      id: "task:1",
      title: "Replaced",
    });
    assert.equal(fullMapReplacementCount, 0);
    assert.equal(singleEntryReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "mapCollection",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.map<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "mapCollection");
    assert.equal(itemEffect.locusProof.locus, "mapCollection");
    assert.equal(itemEffect.locusProof.patchScope, "item");
    assert.equal(itemEffect.locusProof.effectLocusDigest.includes("mapCollection"), true);
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "map-key",
      lookupBreadth: 1,
      traversal: "single-map-entry",
      traversalBreadth: 1,
      reconstruction: "replaceEntry",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-map-collection",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(deliveryEffect.locus.kind, "mapCollection");
    assert.equal(deliveryEffect.locusProof.locus, "mapCollection");
    assert.equal(singleEntryReplacementCount, 2);

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, {
      lookup: "map-key",
      lookupBreadth: 1,
      traversal: "single-map-entry",
      traversalBreadth: 1,
      reconstruction: "replaceEntry",
      reconstructionBreadth: 1,
    });
    assert.equal(singleEntryReplacementCount, 5);
    assert.equal(fullMapReplacementCount, 0);
    assert.equal(readTask(line.value(), "task:1").title, "Aspect");
  } finally {
    await runtime.cleanup();
  }
});

test("map-backed responses deny malformed maps before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskMapResponse(signals, {
      entries: (value) => value.taskMap,
    });
    const tasks = signals.api({
      effects: signals.resource.effects.pessimistic(),
    }).url("/bad-map-collection")
      .response(response)
      .list({
        load: () => ({
          taskMapEntries: [["task:1", { id: "task:1", title: "First" }]],
        }),
      });
    const line = tasks.line({});
    const beforeValue = line.value();
    const beforeEffect = line.diagnostics().lastEffect;

    assert.throws(() => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /entries\(value\) to return a ReadonlyMap/);
    assert.deepEqual(line.value(), beforeValue);
    assert.equal(line.diagnostics().lastEffect, beforeEffect);
  } finally {
    await runtime.cleanup();
  }
});

test("map-backed responses admit structural ReadonlyMap entry views", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    let structuralEntryReadCount = 0;
    let singleEntryReplacementCount = 0;
    const response = createTaskMapResponse(signals, {
      entries(value) {
        structuralEntryReadCount += 1;
        return createReadonlyMapEntryView(value.taskMapEntries);
      },
      replaceEntry(value, itemId, nextItem) {
        singleEntryReplacementCount += 1;
        return {
          ...value,
          taskMapEntries: replaceMapEntry(value.taskMapEntries, itemId, nextItem),
        };
      },
    });
    const tasks = createTaskMapApi(signals, response, "/readonly-map-view");
    const line = tasks.line({});

    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Readonly View" },
    }));

    assert.equal(readTask(line.value(), "task:1").title, "Readonly View");
    assert.equal(line.diagnostics().lastEffect.locus.kind, "mapCollection");
    assert.equal(singleEntryReplacementCount, 1);
    assert.equal(structuralEntryReadCount > 0, true);
  } finally {
    await runtime.cleanup();
  }
});

test("map-backed replaceEntry must preserve the patched key", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskMapResponse(signals, {
      replaceEntry(value, itemId, nextItem) {
        return {
          ...value,
          taskMapEntries: [[itemId, { ...nextItem, id: "task:2" }]],
        };
      },
    });
    const tasks = createTaskMapApi(signals, response, "/corrupt-map-collection");
    const line = tasks.line({});
    const beforeValue = line.value();
    const beforeEffect = line.diagnostics().lastEffect;

    assert.throws(() => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /replaceEntry\(value, itemId, nextItem\) map key "task:1" to match itemId\(item\) "task:2"/);
    assert.deepEqual(line.value(), beforeValue);
    assert.equal(line.diagnostics().lastEffect, beforeEffect);
  } finally {
    await runtime.cleanup();
  }
});

test("map-backed broad replacements preserve map topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskMapResponse(signals);
    const tasks = createTaskMapApi(signals, response, "/map-collection-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      taskMapEntries: [["task:1", { id: "task:1", title: "Broad" }]],
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "mapCollection");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-map",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceEntries",
      reconstructionBreadth: 1,
    });
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      effect.locusProof,
    );
    assert.equal(readTask(line.value(), "task:1").title, "Broad");
  } finally {
    await runtime.cleanup();
  }
});

function createTaskMapResponse(signals, overrides = {}) {
  return signals.resource.response.map()({
    itemId: (task) => task.id,
    entries: overrides.entries ?? ((value) => new Map(value.taskMapEntries)),
    replaceEntries: overrides.replaceEntries ?? (
      (value, taskMap) => ({ ...value, taskMapEntries: [...taskMap] })
    ),
    replaceEntry: overrides.replaceEntry ?? (
      (value, itemId, nextItem) => ({
        ...value,
        taskMapEntries: replaceMapEntry(value.taskMapEntries, itemId, nextItem),
      })
    ),
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskMapApi(signals, response, url, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({
      load: () => ({
        taskMapEntries: [["task:1", { id: "task:1", title: "First" }]],
      }),
    });
}

function replaceMapEntry(taskMapEntries, itemId, nextItem) {
  const nextMap = new Map(taskMapEntries);
  nextMap.set(itemId, nextItem);
  return [...nextMap];
}

function readTask(value, itemId) {
  return new Map(value.taskMapEntries).get(itemId);
}

function createReadonlyMapEntryView(taskMapEntries) {
  const taskMap = new Map(taskMapEntries);
  return Object.freeze({
    get: (itemId) => taskMap.get(itemId),
    has: (itemId) => taskMap.has(itemId),
    values: () => taskMap.values(),
    entries: () => taskMap.entries(),
  });
}
