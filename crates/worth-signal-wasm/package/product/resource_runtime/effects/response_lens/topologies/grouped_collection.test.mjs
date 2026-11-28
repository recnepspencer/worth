import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("grouped responses lower item replacement through grouped collection loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "grouped-collection");
    let fullGroupReplacementCount = 0;
    let singleGroupItemReplacementCount = 0;
    const response = createTaskGroupedResponse(signals, {
      replaceGroups(value, groups) {
        fullGroupReplacementCount += 1;
        return { ...value, groups };
      },
      replaceGroupItem(value, groupId, itemId, nextItem) {
        singleGroupItemReplacementCount += 1;
        return {
          ...value,
          groups: replaceGroupedItem(value.groups, groupId, itemId, nextItem),
        };
      },
    });
    assert.equal(response.lensProof.topology, "groupedCollection");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "groupedCollection" && row.patchScope === "item",
    ), true);

    const tasks = createTaskGroupedApi(signals, response, "/grouped", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", group: "todo", title: "Replaced" },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.equal(readTask(line.value(), "todo", "task:1").title, "Replaced");
    assert.equal(fullGroupReplacementCount, 0);
    assert.equal(singleGroupItemReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "groupedCollection",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.grouped<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "groupedCollection");
    assert.equal(itemEffect.locusProof.locus, "groupedCollection");
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "group-key-item-id",
      lookupBreadth: 1,
      traversal: "single-group",
      traversalBreadth: 1,
      reconstruction: "replaceGroupItem",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-grouped",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", group: "todo", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(readTask(line.value(), "todo", "task:1").title, "Delivered");
    assert.equal(deliveryEffect.locus.kind, "groupedCollection");
    assert.equal(deliveryEffect.locusProof.locus, "groupedCollection");
    assert.deepEqual(deliveryEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.equal(singleGroupItemReplacementCount, 2);

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.equal(readTask(line.value(), "todo", "task:1").title, "Aspect");
    assert.equal(singleGroupItemReplacementCount, 5);
  } finally {
    await runtime.cleanup();
  }
});

test("grouped broad replacements preserve grouped topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskGroupedResponse(signals);
    const tasks = createTaskGroupedApi(signals, response, "/grouped-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      groups: {
        done: [{ id: "task:2", group: "done", title: "Broad" }],
      },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "groupedCollection");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-group-record",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceGroups",
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

test("grouped broad replacements deny corrupt grouped topology before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskGroupedResponse(signals);
    const tasks = createTaskGroupedApi(signals, response, "/grouped-broad-denial");
    const line = tasks.line({});

    assertGroupedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.replace({
      groups: {
        todo: [{ id: "task:1", group: "done", title: "Corrupt" }],
      },
    })), /group key "todo" to match groupId\(item\) "done"/);
  } finally {
    await runtime.cleanup();
  }
});

test("grouped responses deny malformed groups before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskGroupedResponse(signals, {
      groups: () => ({ todo: { id: "task:1", group: "todo", title: "First" } }),
    });
    const tasks = createTaskGroupedApi(signals, response, "/bad-grouped");
    const line = tasks.line({});

    assertGroupedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", group: "todo", title: "Replaced" },
    })), /group "todo" to be an array/);
  } finally {
    await runtime.cleanup();
  }
});

test("grouped responses deny invalid item group lookup before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskGroupedResponse(signals, {
      groupForItem: () => "",
    });
    const tasks = createTaskGroupedApi(signals, response, "/bad-group-lookup");
    const line = tasks.line({});

    assertGroupedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", group: "todo", title: "Replaced" },
    })), /groupForItem\(itemId\).*non-empty group id/);
  } finally {
    await runtime.cleanup();
  }
});

test("grouped responses deny duplicate item ids inside lookup group before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskGroupedResponse(signals, {
      groups: () => ({
        todo: [
          { id: "task:1", group: "todo", title: "First" },
          { id: "task:1", group: "todo", title: "Duplicate" },
        ],
      }),
    });
    const tasks = createTaskGroupedApi(signals, response, "/duplicate-grouped");
    const line = tasks.line({});

    assertGroupedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", group: "todo", title: "Replaced" },
    })), /duplicated grouped item id "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

test("grouped replaceGroupItem must preserve item and group identity", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskGroupedResponse(signals, {
      replaceGroupItem(value, groupId, itemId, nextItem) {
        return {
          ...value,
          groups: {
            [groupId]: [{ ...nextItem, id: itemId, group: "done" }],
          },
        };
      },
    });
    const tasks = createTaskGroupedApi(signals, response, "/corrupt-grouped");
    const line = tasks.line({});

    assertGroupedPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", group: "todo", title: "Replaced" },
    })), /group key "todo" to match groupId\(item\) "done"/);
  } finally {
    await runtime.cleanup();
  }
});

function assertGroupedPatchDeniedWithoutSideEffects(line, patchAction, errorPattern) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, errorPattern);
  assert.deepEqual(line.value(), beforeValue);
  assert.equal(line.diagnostics().lastEffect, beforeEffect);
}

function createTaskGroupedResponse(signals, overrides = {}) {
  return signals.resource.response.grouped()({
    itemId: (task) => task.id,
    groupId: (task) => task.group,
    groupForItem: overrides.groupForItem ?? (() => "todo"),
    groups: overrides.groups ?? ((value) => value.groups),
    replaceGroups: overrides.replaceGroups ?? (
      (value, groups) => ({ ...value, groups })
    ),
    replaceGroupItem: overrides.replaceGroupItem ?? (
      (value, groupId, itemId, nextItem) => ({
        ...value,
        groups: replaceGroupedItem(value.groups, groupId, itemId, nextItem),
      })
    ),
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskGroupedApi(signals, response, url, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({
      load: () => ({
        groups: {
          todo: [{ id: "task:1", group: "todo", title: "First" }],
          done: [],
        },
      }),
    });
}

function replaceGroupedItem(groups, groupId, itemId, nextItem) {
  return Object.fromEntries(
    Object.entries(groups).map(([key, items]) => [
      key,
      key === groupId
        ? items.map((item) => item.id === itemId ? nextItem : item)
        : items,
    ]),
  );
}

function readTask(value, groupId, itemId) {
  return value.groups[groupId].find((task) => task.id === itemId);
}
