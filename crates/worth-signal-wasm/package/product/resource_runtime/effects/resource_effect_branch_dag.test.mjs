import assert from "node:assert/strict";
import test from "node:test";

import { createRealResourceTestRuntime } from "../runtime_fixture/real_resource_runtime.mjs";
import { createBranchHead } from "../runtime_fixture/real_resource_signals.mjs";
import { createEffectLine, titlePatch } from "./resource_effect_dag_fixture.mjs";

test("ten concurrent sibling effects isolate random success and failure", async () => {
  const runtime = await createRealResourceTestRuntime();
  try {
    const canonicalBranch = createBranchHead(runtime.signals, "effect-dag-ten");
    const line = createEffectLine(runtime);
    const baselineBranchCount = runtime.signals.history().branches().length;
    await Promise.all(Array.from({ length: 10 }, (_, index) =>
      line.patch(titlePatch(runtime, index, `optimistic-${index}`))));

    const open = line.effects().open();
    assert.equal(open.length, 10);
    assert.equal(open[0].envelope.effectId, open[0].effectId);
    assert.equal(new Set(open.map((effect) => effect.branchId)).size, 10);
    assert.deepEqual(
      new Set(open.map((effect) => effect.nativeParentBranchId)),
      new Set([Number(canonicalBranch.id)]),
    );
    assert.equal(open.every((effect) => effect.dependencyEffectIds.length === 0), true);
    assert.equal(
      line.effects().projection().kind,
      "derivedEffectProjectionBranch",
    );
    const projectionBeforeRebuild = line.effects().projection();
    const rebuiltProjection = await line.effects().rebuildProjection();
    assert.deepEqual(
      rebuiltProjection.projectedValue,
      projectionBeforeRebuild.projectedValue,
    );
    assert.equal(
      rebuiltProjection.projectionDigest,
      projectionBeforeRebuild.projectionDigest,
    );
    assert.equal(rebuiltProjection.canonicalAuthority, false);

    const randomOrder = [7, 2, 9, 0, 5, 4, 1, 8, 3, 6];
    for (const index of randomOrder) {
      const effect = open[index];
      let settlement;
      if (index % 2 === 0) {
        settlement = await line.effects().confirm(effect.effectId);
      } else {
        settlement = await line.effects().reject(effect.effectId);
      }
      assert.equal(settlement.projection.plan.strategy, "affectedLocusRebuild");
      assert.equal(settlement.projection.plan.counters.affectedLocusCount, 1);
      assert.equal(settlement.projection.plan.counters.openEffectLookupCount, 0);
      assert.equal(settlement.projection.plan.counters.dependencyTraversalCount, 1);
      assert.equal(settlement.projection.plan.counters.reconstructionCount, 1);
      assert.equal(settlement.projection.plan.counters.fallbackBreadth, 0);
    }

    assert.equal(line.effects().open().length, 0);
    assert.equal(line.effects().projection().kind, "canonical");
    assert.deepEqual(
      line.value().items.map((item) => item.title),
      Array.from({ length: 10 }, (_, index) =>
        index % 2 === 0 ? `optimistic-${index}` : `loaded-${index}`),
    );
    assert.equal(
      runtime.signals.history().branches().length,
      baselineBranchCount,
    );
    assert.deepEqual(line.effects().counters(), {
      effectLookupCount: 10,
      pendingAdmissionCount: 0,
      openEffectCount: 0,
      dependencyIndexKeyCount: 0,
      locusIndexKeyCount: 0,
      retryLineageIndexKeyCount: 0,
    });
  } finally {
    await runtime.cleanup();
  }
});

test("declared child effects use a derived basis and retire with rejected parent", async () => {
  const runtime = await createRealResourceTestRuntime();
  try {
    const canonicalBranch = createBranchHead(runtime.signals, "effect-dag-child");
    const line = createEffectLine(runtime);
    await line.patch(titlePatch(runtime, 0, "parent"));
    const parent = line.effects().open()[0];
    const childPatch = runtime.mod.resourcePatch.dependsOn(
      titlePatch(runtime, 1, "child"),
      [parent.effectId],
    );
    await line.patch(childPatch);
    const child = line.effects().open().find(
      (effect) => effect.effectId !== parent.effectId,
    );

    assert.deepEqual(child.dependencyEffectIds, [parent.effectId]);
    assert.notEqual(child.dependencyBasisBranchId, null);
    assert.equal(child.nativeParentBranchId, child.dependencyBasisBranchId);
    assert.notEqual(child.nativeParentBranchId, Number(canonicalBranch.id));

    const childResponse = await line.effects().confirm(child.effectId);
    assert.equal(childResponse.kind, "responseRecorded");
    const result = await line.effects().reject(parent.effectId);
    assert.deepEqual(
      result.retired.map((retired) => retired.effectId),
      [child.effectId, parent.effectId],
    );
    assert.equal(line.effects().open().length, 0);
    assert.deepEqual(
      line.value().items.slice(0, 2).map((item) => item.title),
      ["loaded-0", "loaded-1"],
    );
  } finally {
    await runtime.cleanup();
  }
});

test("same-locus sibling confirmation converges independently of response order", async () => {
  const laterFirst = await confirmSameLocusInOrder([1, 0]);
  const earlierFirst = await confirmSameLocusInOrder([0, 1]);
  assert.equal(laterFirst, "second");
  assert.equal(earlierFirst, "second");
});

test("server revision outranks client admission order at one locus", async () => {
  const runtime = await createRealResourceTestRuntime();
  try {
    createBranchHead(runtime.signals, "effect-dag-server-revision");
    const line = createEffectLine(runtime);
    await line.patch(titlePatch(runtime, 0, "revision-20"));
    await line.patch(titlePatch(runtime, 0, "revision-10"));
    const [higherRevision, laterAdmission] = line.effects().open();
    await line.effects().confirm(higherRevision.effectId, {
      serverRevision: 20,
    });
    const lower = await line.effects().confirm(laterAdmission.effectId, {
      serverRevision: 10,
    });
    assert.equal(lower.reconciliation.conflict.kind, "superseded");
    assert.equal(line.value().items[0].title, "revision-20");
  } finally {
    await runtime.cleanup();
  }
});

test("dependent edit response waits for create confirmation then closes automatically", async () => {
  const runtime = await createRealResourceTestRuntime();
  try {
    createBranchHead(runtime.signals, "effect-dag-create-edit");
    const line = createEffectLine(runtime);
    await line.patch(runtime.mod.resourcePatch.insert({
      itemId: "item:new",
      placement: "append",
      nextItem: { id: "item:new", title: "created" },
    }));
    const createEffect = line.effects().open()[0];
    await line.patch(runtime.mod.resourcePatch.dependsOn(
      runtime.mod.resourcePatch.itemAspect({
        itemId: "item:new",
        aspect: "title",
        value: "edited",
      }),
      [createEffect.effectId],
    ));
    const editEffect = line.effects().open().find(
      (effect) => effect.effectId !== createEffect.effectId,
    );

    const recorded = await line.effects().confirm(editEffect.effectId);
    assert.equal(recorded.kind, "responseRecorded");
    assert.deepEqual(
      recorded.waitingOnDependencyEffectIds,
      [createEffect.effectId],
    );
    assert.equal(line.effects().open().length, 2);

    const confirmed = await line.effects().confirm(createEffect.effectId);
    assert.deepEqual(
      confirmed.automaticallySettled.map((entry) => entry.effectId),
      [editEffect.effectId],
    );
    assert.equal(line.effects().open().length, 0);
    assert.equal(
      line.value().items.find((item) => item.id === "item:new").title,
      "edited",
    );
  } finally {
    await runtime.cleanup();
  }
});

test("dependency denials are typed and side-effect free before branch creation", async () => {
  const runtime = await createRealResourceTestRuntime();
  try {
    createBranchHead(runtime.signals, "effect-dag-denials");
    const line = createEffectLine(runtime);
    await line.patch(titlePatch(runtime, 0, "first"));
    const first = line.effects().open()[0];
    const branchCount = () => runtime.signals.history().branches().length;
    const beforeUnknown = branchCount();
    await assert.rejects(
      line.patch(runtime.mod.resourcePatch.dependsOn(
        titlePatch(runtime, 1, "unknown"),
        ["missing-effect"],
      )),
      (error) => error.code === "unknownDependency",
    );
    assert.equal(branchCount(), beforeUnknown);

    const selfId = first.effectId.replace(/:1$/u, ":3");
    const beforeSelf = branchCount();
    await assert.rejects(
      line.patch(runtime.mod.resourcePatch.dependsOn(
        titlePatch(runtime, 2, "self"),
        [selfId],
      )),
      (error) => error.code === "selfDependency",
    );
    assert.equal(branchCount(), beforeSelf);

    await line.effects().reject(first.effectId);
    const beforeRetired = branchCount();
    await assert.rejects(
      line.patch(runtime.mod.resourcePatch.dependsOn(
        titlePatch(runtime, 3, "retired"),
        [first.effectId],
      )),
      (error) => error.code === "retiredDependency",
    );
    assert.equal(branchCount(), beforeRetired);

    await line.patch(titlePatch(runtime, 4, "older-open"));
    await line.patch(titlePatch(runtime, 5, "confirm-to-advance"));
    const [older, advancing] = line.effects().open();
    await line.effects().confirm(advancing.effectId);
    const beforeGeneration = branchCount();
    await assert.rejects(
      line.patch(runtime.mod.resourcePatch.dependsOn(
        titlePatch(runtime, 6, "incompatible"),
        [older.effectId],
      )),
      (error) => error.code === "generationIncompatible",
    );
    assert.equal(branchCount(), beforeGeneration);
  } finally {
    await runtime.cleanup();
  }
});

async function confirmSameLocusInOrder(order) {
  const runtime = await createRealResourceTestRuntime();
  try {
    createBranchHead(runtime.signals, `effect-dag-conflict-${order.join("")}`);
    const line = createEffectLine(runtime);
    await line.patch(titlePatch(runtime, 0, "first"));
    await line.patch(titlePatch(runtime, 0, "second"));
    const effects = line.effects().open();
    const settlements = [];
    for (const index of order) {
      settlements.push(await line.effects().confirm(effects[index].effectId));
    }
    assert.equal(
      settlements.some((settlement) =>
        settlement.reconciliation.conflict.kind === "superseded"),
      order[0] === 1,
    );
    return line.value().items[0].title;
  } finally {
    await runtime.cleanup();
  }
}
