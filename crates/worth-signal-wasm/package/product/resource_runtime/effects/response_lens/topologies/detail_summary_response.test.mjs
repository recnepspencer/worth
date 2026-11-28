import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("detail responses close whole-response effect topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "detail-response-closeout");
    const response = signals.resource.response.detail()();
    const users = createDetailApi(signals, response, {
      effects: signals.resource.effects.branchNative(),
    });
    const line = users.line({ userId: "user:1" });

    await line.patch(signalsMod.resourcePatch.replace({
      id: "user:1",
      name: "Local",
    }));
    const localEffect = line.diagnostics().lastEffect;

    assert.deepEqual(line.value(), { id: "user:1", name: "Local" });
    assertWholeResponseEffect(localEffect, {
      locus: "detailResponse",
      lensSource: "resource.response.detail<T>()",
      topology: "detail",
      cost: {
        lookup: "detail-response",
        lookupBreadth: 0,
        traversal: "whole-response",
        traversalBreadth: 1,
        reconstruction: "replaceDetailResponse",
        reconstructionBreadth: 1,
      },
    });
    assert.equal(localEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(localEffect.profile.rebase, "unavailable");
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      localEffect.locusProof,
    );

    line.deliver(signalsMod.resourceDelivery.replace({
      packetId: "pkt-detail-closeout",
      basisId: null,
      nextValue: { id: "user:1", name: "Delivered" },
    }));
    const deliveryEffect = line.diagnostics().lastEffect;

    assert.deepEqual(line.value(), { id: "user:1", name: "Delivered" });
    assert.deepEqual(deliveryEffect.locus, { kind: "detailResponse" });
    assert.deepEqual(deliveryEffect.locusProof, localEffect.locusProof);
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      deliveryEffect.locusProof,
    );

    assertWholeResponsePatchDeniedWithoutSideEffects(
      line,
      () => line.patch(signalsMod.resourcePatch.item({
        itemId: "user:1",
        nextItem: { id: "user:1", name: "Illegal" },
      })),
      {
        message: /cannot lower effect locus "membership"/,
        requestedLocus: "membership",
      },
    );
  } finally {
    await runtime.cleanup();
  }
});

test("summary responses close whole-response effect topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "summary-response-closeout");
    const response = signals.resource.response.summary()();
    const totals = createSummaryApi(signals, response, {
      effects: signals.resource.effects.branchNative(),
    });
    const line = totals.line({});

    await line.patch(signalsMod.resourcePatch.replace({ open: 2, closed: 1 }));
    const localEffect = line.diagnostics().lastEffect;

    assert.deepEqual(line.value(), { open: 2, closed: 1 });
    assertWholeResponseEffect(localEffect, {
      locus: "summaryResponse",
      lensSource: "resource.response.summary<T>()",
      topology: "summary",
      cost: {
        lookup: "summary-response",
        lookupBreadth: 0,
        traversal: "whole-response",
        traversalBreadth: 1,
        reconstruction: "replaceSummaryResponse",
        reconstructionBreadth: 1,
      },
    });
    assert.equal(localEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(localEffect.profile.rebase, "unavailable");
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      localEffect.locusProof,
    );

    line.deliver(signalsMod.resourceDelivery.replace({
      packetId: "pkt-summary-closeout",
      basisId: null,
      nextValue: { open: 3, closed: 1 },
    }));
    const deliveryEffect = line.diagnostics().lastEffect;

    assert.deepEqual(line.value(), { open: 3, closed: 1 });
    assert.deepEqual(deliveryEffect.locus, { kind: "summaryResponse" });
    assert.deepEqual(deliveryEffect.locusProof, localEffect.locusProof);
    assert.deepEqual(
      line.history().verificationPackage().lifecycle.lastEffect.locusProof,
      deliveryEffect.locusProof,
    );

    assertWholeResponsePatchDeniedWithoutSideEffects(
      line,
      () => line.patch(signalsMod.resourcePatch.item({
        itemId: "summary:1",
        nextItem: { id: "summary:1" },
      })),
      {
        message: /cannot lower effect locus "membership"/,
        requestedLocus: "membership",
      },
    );
  } finally {
    await runtime.cleanup();
  }
});

function createDetailApi(signals, response, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url("/detail-closeout/:userId")
    .response(response)
    .detail({
      load: ({ userId }) => ({ id: userId, name: "First" }),
    });
}

function createSummaryApi(signals, response, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url("/summary-closeout")
    .response(response)
    .detail({
      load: () => ({ open: 1, closed: 0 }),
    });
}

function assertWholeResponseEffect(effect, expected) {
  assert.deepEqual(effect.locus, { kind: expected.locus });
  assert.equal(effect.locusProof.lensSource, expected.lensSource);
  assert.equal(effect.locusProof.topology, expected.topology);
  assert.equal(effect.locusProof.locus, expected.locus);
  assert.equal(effect.locusProof.patchScope, "line");
  assert.deepEqual(effect.locusProof.cost, expected.cost);
  assert.equal(effect.counters.responseLensBreadth, 1);
  assert.equal(effect.counters.effectLocusBreadth, 1);
}

function assertWholeResponsePatchDeniedWithoutSideEffects(
  line,
  patchAction,
  expectedDenial,
) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, (error) => {
    assert.match(error.message, expectedDenial.message);
    assert.equal(error.denialProof.requestedLocus, expectedDenial.requestedLocus);
    assert.equal(error.denialProof.reason, expectedDenial.reason ?? "unsupportedCapability");
    return true;
  });
  assert.deepEqual(line.value(), beforeValue);
  assert.deepEqual(line.diagnostics().lastEffect, beforeEffect);
}
