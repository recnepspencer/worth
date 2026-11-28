import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("entity-store responses lower item replacement through entity-store effect loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "entity-store");
    let fullRecordReplacementCount = 0;
    let singleEntityReplacementCount = 0;
    const response = signals.resource.response.entityStore()({
      itemId: (task) => task.id,
      entities: (value) => value.entities,
      replaceEntities: (value, entities) => {
        fullRecordReplacementCount += 1;
        return { ...value, entities };
      },
      replaceEntity: (value, itemId, nextItem) => {
        singleEntityReplacementCount += 1;
        return {
          ...value,
          entities: replaceEntityWithoutReadingSiblings(
            value.entities,
            itemId,
            nextItem,
          ),
        };
      },
      aspects: signals.resource.response.objectAspects()({
        title: "title",
      }),
    });
    assert.equal(response.lensProof.topology, "entityStore");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "entityStore" && row.patchScope === "item",
    ), true);

    const tasks = signals.api({
      effects: signals.resource.effects.branchNative(),
    }).url("/entity-store")
      .response(response)
      .list({
        load: () => ({
          entities: { "task:1": { id: "task:1", title: "First" } },
          total: 1,
        }),
      });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    }));

    const itemEffect = line.diagnostics().lastEffect;
    assert.deepEqual(line.value().entities["task:1"], {
      id: "task:1",
      title: "Replaced",
    });
    assert.equal(fullRecordReplacementCount, 0);
    assert.equal(singleEntityReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "entityStore",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.entityStore<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "entityStore");
    assert.equal(itemEffect.locusProof.locus, "entityStore");
    assert.equal(itemEffect.locusProof.patchScope, "item");
    assert.equal(itemEffect.locusProof.effectLocusDigest.includes("entityStore"), true);
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "entity-id",
      lookupBreadth: 1,
      traversal: "single-entity-record",
      traversalBreadth: 1,
      reconstruction: "replaceEntity",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-entity-store",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.deepEqual(line.value().entities["task:1"], {
      id: "task:1",
      title: "Delivered",
    });
    assert.equal(fullRecordReplacementCount, 0);
    assert.equal(singleEntityReplacementCount, 2);
    assert.equal(deliveryEffect.locus.kind, "entityStore");
    assert.equal(deliveryEffect.locusProof.locus, "entityStore");

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.deepEqual(line.value().entities["task:1"], {
      id: "task:1",
      title: "Aspect",
    });
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, {
      lookup: "entity-id",
      lookupBreadth: 1,
      traversal: "single-entity-record",
      traversalBreadth: 1,
      reconstruction: "replaceEntity",
      reconstructionBreadth: 1,
    });
    assert.equal(fullRecordReplacementCount, 0);
    assert.equal(singleEntityReplacementCount, 5);

    const beforeValue = line.value();
    const beforeEffect = line.diagnostics().lastEffect;
    await assert.rejects(
      line.patch(tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:2", title: "Wrong identity" },
      })),
      /preserve item identity/,
    );
    assert.deepEqual(line.value(), beforeValue);
    assert.deepEqual(line.diagnostics().lastEffect, beforeEffect);
  } finally {
    await runtime.cleanup();
  }
});

test("entity-store broad replacements preserve normalized topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = signals.resource.response.entityStore()({
      itemId: (task) => task.id,
      entities: (value) => value.entities,
      replaceEntities: (value, entities) => ({ ...value, entities }),
      replaceEntity: (value, itemId, nextItem) => ({
        ...value,
        entities: { ...value.entities, [itemId]: nextItem },
      }),
    });
    const tasks = signals.api({
      effects: signals.resource.effects.pessimistic(),
    }).url("/entity-store-broad")
      .response(response)
      .list({
        load: () => ({
          entities: { "task:1": { id: "task:1", title: "First" } },
        }),
      });
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      entities: { "task:2": { id: "task:2", title: "Broad" } },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "entityStore");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-entity-record",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceEntities",
      reconstructionBreadth: 1,
    });
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      effect.locusProof,
    );
    assert.equal(line.value().entities["task:2"].title, "Broad");
  } finally {
    await runtime.cleanup();
  }
});

function replaceEntityWithoutReadingSiblings(entities, itemId, nextItem) {
  const nextEntities = Object.create(Object.getPrototypeOf(entities));
  const siblingDescriptors = Object.getOwnPropertyDescriptors(entities);
  delete siblingDescriptors[itemId];
  Object.defineProperties(nextEntities, siblingDescriptors);
  Object.defineProperty(nextEntities, itemId, {
    value: nextItem,
    enumerable: true,
    configurable: true,
    writable: true,
  });
  return nextEntities;
}

test("malformed entity-store responses deny before value diagnostics or effect proof changes", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = signals.resource.response.entityStore()({
      itemId: (task) => task.id,
      entities: (value) => value.entities,
      replaceEntities: (value, entities) => ({ ...value, entities }),
      replaceEntity: (value, itemId, nextItem) => ({
        ...value,
        entities: { ...value.entities, [itemId]: nextItem },
      }),
    });
    const tasks = signals.api({
      effects: signals.resource.effects.pessimistic(),
    }).url("/bad-entity-store")
      .response(response)
      .list({
        load: () => ({ entities: [{ id: "task:1", title: "First" }] }),
      });
    const line = tasks.line({});
    const beforeValue = line.value();
    const beforeEffect = line.diagnostics().lastEffect;

    assert.throws(() => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /entities\(value\) to return an object record/);
    assert.deepEqual(line.value(), beforeValue);
    assert.equal(line.diagnostics().lastEffect, beforeEffect);
  } finally {
    await runtime.cleanup();
  }
});

test("entity-store replaceEntity must preserve record key identity before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = signals.resource.response.entityStore()({
      itemId: (task) => task.id,
      entities: (value) => value.entities,
      replaceEntities: (value, entities) => ({ ...value, entities }),
      replaceEntity: (value, itemId, nextItem) => ({
        ...value,
        entities: {
          ...value.entities,
          [itemId]: { ...nextItem, id: "task:2" },
        },
      }),
    });
    const tasks = signals.api({
      effects: signals.resource.effects.pessimistic(),
    }).url("/corrupt-entity-store-replace")
      .response(response)
      .list({
        load: () => ({
          entities: { "task:1": { id: "task:1", title: "First" } },
        }),
      });
    const line = tasks.line({});
    const beforeValue = line.value();
    const beforeEffect = line.diagnostics().lastEffect;

    assert.throws(() => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /replaceEntity\(value, itemId, nextItem\) entity key "task:1" to match itemId\(item\) "task:2"/);
    assert.deepEqual(line.value(), beforeValue);
    assert.equal(line.diagnostics().lastEffect, beforeEffect);
  } finally {
    await runtime.cleanup();
  }
});

test("entity-store direct replacement must keep the patched key visible", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = signals.resource.response.entityStore()({
      itemId: (task) => task.id,
      entities: (value) => value.entities,
      replaceEntities: (value, entities) => ({ ...value, entities }),
      replaceEntity: (value, itemId) => {
        const { [itemId]: _removedEntity, ...entities } = value.entities;
        return { ...value, entities };
      },
    });
    const tasks = signals.api({
      effects: signals.resource.effects.pessimistic(),
    }).url("/missing-entity-store-replace")
      .response(response)
      .list({
        load: () => ({
          entities: { "task:1": { id: "task:1", title: "First" } },
        }),
      });
    const line = tasks.line({});
    const beforeValue = line.value();
    const beforeEffect = line.diagnostics().lastEffect;

    assert.throws(() => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /replaceEntity\(value, itemId, nextItem\) to preserve entity id "task:1"/);
    assert.deepEqual(line.value(), beforeValue);
    assert.equal(line.diagnostics().lastEffect, beforeEffect);
  } finally {
    await runtime.cleanup();
  }
});
