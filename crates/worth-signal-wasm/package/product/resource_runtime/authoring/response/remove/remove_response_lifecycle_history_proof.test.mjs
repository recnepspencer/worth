import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("remove response plans carry lifecycle proof and digest parity for collection deletion plus summary patch", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "remove-lifecycle-delete-summary");
    const collectionResponse = runtime.signals.resource.response.collection({
      itemId: (item) => item.id,
      items: (value) => value.items,
      replaceItems: (value, nextItems) => ({ ...value, items: [...nextItems] }),
      summaries: runtime.signalsMod.resourceValueSummaries({
        total: {
          read: (value) => value.total,
          write: (value, total) => ({ ...value, total }),
        },
      }),
    });
    const taskList = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks")
      .response(collectionResponse)
      .list({
        load: () => ({
          items: [
            { id: "t1", title: "First" },
            { id: "t2", title: "Second" },
          ],
          total: 2,
        }),
      });
    const taskCounts = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/task-counts")
      .response(collectionResponse)
      .list({
        load: () => ({
          items: [],
          total: 2,
        }),
      });
    const taskLine = taskList.line({});
    const countsLine = taskCounts.line({});
    const removeLine = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()({ total: "total" }))
      .remove({
        reconciles: [{
          family: taskList,
          params: () => ({}),
          fallback: "deletionUnavailable",
          collection: { kind: "delete" },
        }, {
          family: taskCounts,
          params: () => ({}),
          fallback: "refetchRequired",
          summary: { kind: "summary", summary: "total" },
        }],
        load: ({ taskId }) => ({ id: taskId, total: 1 }),
      })
      .line({ taskId: "t1" });

    const plan = removeLine.mutationResponse();

    assert.equal(plan.lifecycleProof.count, 2);
    assert.equal(plan.lifecycleProof.entries[0].entryKind, "reconciliation");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "item:t1");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.locusKind, "membership");
    assert.equal(plan.lifecycleProof.entries[1].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[1].mergeRebase.granularity, "summary:total");
    assert.match(plan.lifecycleProof.rollbackDigest, /notApplicable/);
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:item:t1/);
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:summary:total/);
    assert.equal(
      removeLine.summary().diagnostics.latest.mutationResponseRollbackDigest,
      plan.lifecycleProof.rollbackDigest,
    );
    assert.equal(
      removeLine.summary().diagnostics.latest.mutationResponseMergeRebaseDigest,
      plan.lifecycleProof.mergeRebaseDigest,
    );
    assert.deepEqual(taskLine.value(), {
      items: [{ id: "t2", title: "Second" }],
      total: 2,
    });
    assert.deepEqual(countsLine.value(), {
      items: [],
      total: 1,
    });
    assert.equal(taskLine.history().lifecycle.at(-1)?.event, "delivered");
    assert.equal(countsLine.history().lifecycle.at(-1)?.event, "delivered");
  } finally {
    await runtime.cleanup();
  }
});

test("remove tombstone reconciliation remains exact-restorable and preserves distinct lifecycle proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "remove-lifecycle-tombstone");
    const taskList = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks")
      .response(runtime.signals.resource.response.collection({
        itemId: (item) => item.id,
        items: (value) => value.items,
        replaceItems: (value, nextItems) => ({ ...value, items: [...nextItems] }),
      }))
      .list({
        load: () => ({
          items: [
            { id: "t1", title: "First", status: "active" },
            { id: "t2", title: "Second", status: "active" },
          ],
        }),
      });
    const taskLine = taskList.line({});
    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .remove({
        reconciles: [{
          family: taskList,
          params: () => ({}),
          fallback: "refetchRequired",
          collection: { kind: "item" },
        }],
        load: ({ taskId }) => ({
          id: taskId,
          title: "First",
          status: "deleted",
        }),
      })
      .line({ taskId: "t1" })
      .mutationResponse();

    const restoreAvailability = taskLine.history().availability.restoreExact;
    const restoreResult = taskLine.history().restoreExact();
    const verification = taskLine.history().verificationPackage();

    assert.equal(plan.executionArtifacts[0].kind, "exactCollectionTombstone");
    assert.equal(plan.lifecycleProof.count, 1);
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "item:t1");
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:item:t1/);
    assert.equal(restoreAvailability.kind, "available");
    assert.equal(restoreAvailability.mode, "SameRuntimeBranchExact");
    assert.equal(restoreResult.kind, "restored");
    assert.deepEqual(taskLine.value(), {
      items: [
        { id: "t1", title: "First", status: "active" },
        { id: "t2", title: "Second", status: "active" },
      ],
    });
    assert.equal(taskLine.history().lifecycle.at(-1)?.event, "restored");
    assert.equal(verification.typedDenials.restoreExact, null);
  } finally {
    await runtime.cleanup();
  }
});

test("remove detail invalidation remains exact-restorable and keeps lifecycle proof aligned with stale delivery", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "remove-lifecycle-detail-invalidation");
    const taskDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .detail({
        load: ({ taskId }) => ({ id: taskId, title: "First", status: "active" }),
      });
    const detailLine = taskDetail.line({ taskId: "t1" });
    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.summary()())
      .remove({
        reconciles: [{
          family: taskDetail,
          params: ({ taskId }) => ({ taskId }),
          fallback: "refetchRequired",
          detail: { kind: "invalidate" },
        }],
        load: () => 0,
      })
      .line({ taskId: "t1" })
      .mutationResponse();

    assert.equal(detailLine.freshness().kind, "stale");
    const restoreAvailability = detailLine.history().availability.restoreExact;
    const restoreResult = detailLine.history().restoreExact();
    const verification = detailLine.history().verificationPackage();

    assert.equal(plan.executionArtifacts[0].kind, "exactDetailInvalidation");
    assert.equal(plan.lifecycleProof.count, 1);
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "invalidation");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.locusKind, "invalidation");
    assert.equal(restoreAvailability.kind, "available");
    assert.equal(restoreAvailability.mode, "SameRuntimeBranchExact");
    assert.equal(restoreResult.kind, "restored");
    assert.deepEqual(detailLine.value(), { id: "t1", title: "First", status: "active" });
    assert.equal(detailLine.freshness().kind, "fresh");
    assert.equal(detailLine.history().lifecycle.at(-1)?.event, "restored");
    assert.equal(verification.typedDenials.restoreExact, null);
  } finally {
    await runtime.cleanup();
  }
});

test("remove detail replacement keeps exact restore and line-level merge proof aligned with the canonical deleted payload", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "remove-lifecycle-detail-replace");
    const taskDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .detail({
        load: ({ taskId }) => ({ id: taskId, title: "First", status: "active" }),
      });
    const detailLine = taskDetail.line({ taskId: "t1" });
    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .remove({
        reconciles: [{
          family: taskDetail,
          params: ({ taskId }) => ({ taskId }),
          fallback: "refetchRequired",
          detail: { kind: "replace" },
        }],
        load: ({ taskId }) => ({
          id: taskId,
          title: "First",
          status: "deleted",
          deletedAt: "2026-05-13T00:00:00Z",
        }),
      })
      .line({ taskId: "t1" })
      .mutationResponse();

    const restoreAvailability = detailLine.history().availability.restoreExact;
    const restoreResult = detailLine.history().restoreExact();
    const verification = detailLine.history().verificationPackage();

    assert.equal(plan.executionArtifacts[0].kind, "exactDetail");
    assert.equal(plan.lifecycleProof.count, 1);
    assert.equal(plan.lifecycleProof.entries[0].rollback.kind, "notApplicable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "detailResponse");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.locusKind, "detailResponse");
    assert.equal(restoreAvailability.kind, "available");
    assert.equal(restoreAvailability.mode, "SameRuntimeBranchExact");
    assert.equal(restoreResult.kind, "restored");
    assert.deepEqual(detailLine.value(), { id: "t1", title: "First", status: "active" });
    assert.equal(detailLine.history().lifecycle.at(-1)?.event, "restored");
    assert.equal(verification.typedDenials.restoreExact, null);
  } finally {
    await runtime.cleanup();
  }
});

test("fallback-only remove response plans keep typed lifecycle unavailability", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const taskList = runtime.signals.api({}).url("/tasks")
      .response(runtime.signals.resource.response.array({
        itemId: (item) => item.id,
      }))
      .list({
        load: () => [],
      });

    const plan = runtime.signals.api({}).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .remove({
        reconciles: [{
          family: taskList,
          params: () => ({}),
          fallback: "deletionUnavailable",
          collection: { kind: "delete" },
        }],
        load: ({ taskId }) => ({ id: taskId }),
      })
      .line({ taskId: "t1" })
      .mutationResponse();

    assert.equal(plan.lifecycleProof.count, 1);
    assert.equal(plan.lifecycleProof.entries[0].rollback.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "deletionUnavailable");
  } finally {
    await runtime.cleanup();
  }
});
