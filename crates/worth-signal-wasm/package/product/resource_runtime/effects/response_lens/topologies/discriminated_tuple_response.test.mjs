import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("discriminated tuple responses lower item replacement through active variant loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "discriminated-tuple");
    const response = createTaskTupleResponse(signals);
    assert.equal(response.lensProof.topology, "discriminatedTuple");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "discriminatedTuple" && row.patchScope === "item",
    ), true);

    const tasks = createTaskTupleApi(signals, response, "/tuple", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.equal(readPrimaryTask(line.value(), "task:1").title, "Replaced");
    assert.deepEqual(itemEffect.locus, {
      kind: "discriminatedTuple",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.discriminated<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "discriminatedTuple");
    assert.equal(itemEffect.locusProof.locus, "discriminatedTuple");
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "tuple-discriminator-item-id",
      lookupBreadth: 1,
      traversal: "active-variant-items",
      traversalBreadth: 1,
      reconstruction: "replaceVariantItems",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-tuple",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(readPrimaryTask(line.value(), "task:1").title, "Delivered");
    assert.equal(deliveryEffect.locus.kind, "discriminatedTuple");
    assert.deepEqual(deliveryEffect.locusProof.cost, itemEffect.locusProof.cost);

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.equal(readPrimaryTask(line.value(), "task:1").title, "Aspect");
  } finally {
    await runtime.cleanup();
  }
});

test("discriminated tuple broad replacements preserve active variant topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTupleResponse(signals);
    const tasks = createTaskTupleApi(signals, response, "/tuple-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      kind: "secondary",
      secondary: [{ id: "task:2", title: "Broad" }],
      meta: { total: 1 },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "discriminatedTuple");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-tuple-envelope",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceVariantItems",
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

test("discriminated tuple responses deny unknown discriminators before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTupleResponse(signals);
    const tasks = createTaskTupleApi(signals, response, "/tuple-unknown");
    const line = tasks.line({});

    assertTuplePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.replace({
      kind: "archived",
      archived: [{ id: "task:1", title: "Bad" }],
      meta: { total: 1 },
    })), /discriminator "archived" to name a declared variant/);
  } finally {
    await runtime.cleanup();
  }
});

test("discriminated tuple replaceItems must preserve active discriminator", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTupleResponse(signals, {
      primary: {
        items: (value) => value.primary,
        replaceItems: (_value, nextItems) => ({
          kind: "secondary",
          secondary: nextItems,
          meta: { total: nextItems.length },
        }),
      },
    });
    const tasks = createTaskTupleApi(signals, response, "/tuple-corrupt");
    const line = tasks.line({});

    assertTuplePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /preserve discriminator "primary"/);
  } finally {
    await runtime.cleanup();
  }
});

function createTaskTupleResponse(signals, variants = {}) {
  return signals.resource.response.discriminated()({
    itemId: (task) => task.id,
    discriminator: (value) => value.kind,
    variants: {
      primary: variants.primary ?? {
        items: (value) => value.primary,
        replaceItems: (value, nextItems) => ({
          ...value,
          primary: [...nextItems],
        }),
      },
      secondary: variants.secondary ?? {
        items: (value) => value.secondary,
        replaceItems: (value, nextItems) => ({
          ...value,
          secondary: [...nextItems],
        }),
      },
    },
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskTupleApi(signals, response, url, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({
      load: () => ({
        kind: "primary",
        primary: [{ id: "task:1", title: "First" }],
        meta: { total: 1 },
      }),
    });
}

function readPrimaryTask(value, itemId) {
  return value.primary.find((task) => task.id === itemId);
}

function assertTuplePatchDeniedWithoutSideEffects(line, patchAction, errorPattern) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, errorPattern);
  assert.deepEqual(line.value(), beforeValue);
  assert.equal(line.diagnostics().lastEffect, beforeEffect);
}
