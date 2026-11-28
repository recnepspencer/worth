import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("save response plans carry rollback posture and merge proof for exact detail targets", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "mutation-response-detail-proof");
    const readFamily = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/users/:userId")
      .detail({
        load: ({ userId }) => ({ id: userId, name: "First" }),
      });
    readFamily.line({ userId: "u1" });
    const saveUser = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/users/:userId")
      .response(runtime.signals.resource.response.detail()())
      .update({
        reconciles: [
          {
            family: readFamily,
            params: ({ userId }) => ({ userId }),
            fallback: "refetchRequired",
            detail: { kind: "replace" },
          },
        ],
        load: ({ userId, body }) => ({ id: userId, name: body.name }),
      });

    const plan = saveUser.line({
      userId: "u1",
      body: { name: "Updated" },
    }).mutationResponse();
    const proof = plan.lifecycleProof.entries[0];

    assert.equal(proof.effectId, plan.executionArtifacts[0].effectId);
    assert.equal(proof.replayExact.kind, "available");
    assert.equal(proof.replayExact.mode, "SameRuntimeSignalExact");
    assert.equal(proof.restoreExact.kind, "available");
    assert.equal(proof.restoreExact.mode, "SameRuntimeBranchExact");
    assert.equal(proof.rollback.kind, "notApplicable");
    assert.equal(proof.rollback.mode, null);
    assert.equal(proof.rollback.branchId, null);
    assert.equal(proof.mergeRebase.kind, "unavailable");
    assert.equal(proof.mergeRebase.granularity, "line");
    assert.equal(proof.mergeRebase.locusKind, "line");
    assert.match(plan.lifecycleProof.rollbackDigest, /notApplicable/);
    assert.match(plan.lifecycleProof.replayExactDigest, /reconciliation:mutationTarget1:.*:available:SameRuntimeSignalExact/);
    assert.match(plan.lifecycleProof.restoreExactDigest, /reconciliation:mutationTarget1:.*:available:SameRuntimeBranchExact/);
    assert.ok(plan.lifecycleProof.rollbackDigest.includes(proof.effectId));
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:line/);
    assert.ok(plan.lifecycleProof.mergeRebaseDigest.includes(proof.effectId));
    assert.equal(plan.counters.lifecycleProofBreadth, 1);
  } finally {
    await runtime.cleanup();
  }
});

test("save response plans name granular field and summary merge proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "mutation-response-granular-proof");
    const detailFields = runtime.signals.resource.detailFields({
      name: {
        read: (value) => value.name,
        write: (value, name) => ({ ...value, name }),
      },
    });
    const profileRead = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/profiles/:profileId")
      .detail({
        reconcile: detailFields,
        load: ({ profileId }) => ({ id: profileId, name: "First" }),
      });
    profileRead.line({ profileId: "p1" });
    const profileSave = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/profiles/:profileId")
      .response(runtime.signals.resource.response.detail()({ name: "name" }))
      .update({
        reconciles: [
          {
            family: profileRead,
            params: ({ profileId }) => ({ profileId }),
            fallback: "refetchRequired",
            detail: { kind: "field", field: "name" },
          },
        ],
        load: ({ body }) => ({ name: body.name }),
      });
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
    })
      .url("/task-search")
      .response(collectionResponse)
      .list({
        load: () => ({
          items: [{ id: "t1", title: "First" }],
          total: 1,
        }),
      });
    taskList.line({});
    const statsSave = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/task-search/stats")
      .response(runtime.signals.resource.response.detail()({ total: "total" }))
      .update({
        reconciles: [
          {
            family: taskList,
            params: () => ({}),
            fallback: "refetchRequired",
            summary: { kind: "summary", summary: "total" },
          },
        ],
        load: ({ body }) => ({ total: body.total }),
      });

    const fieldPlan = profileSave.line({
      profileId: "p1",
      body: { name: "Renamed" },
    }).mutationResponse();
    const summaryLine = statsSave.line({ body: { total: 2 } });
    const summaryPlan = summaryLine.mutationResponse();

    assert.equal(
      fieldPlan.lifecycleProof.entries[0].mergeRebase.granularity,
      "field:name",
    );
    assert.equal(
      fieldPlan.lifecycleProof.entries[0].mergeRebase.locusKind,
      "detailField",
    );
    assert.match(
      fieldPlan.lifecycleProof.mergeRebaseDigest,
      /unavailable:field:name:detailField/,
    );
    assert.equal(
      summaryPlan.lifecycleProof.entries[0].mergeRebase.granularity,
      "summary:total",
    );
    assert.equal(
      summaryLine.summary().diagnostics.latest.mutationResponseReplayExactDigest,
      summaryPlan.lifecycleProof.replayExactDigest,
    );
    assert.equal(
      summaryLine.summary().diagnostics.latest.mutationResponseRestoreExactDigest,
      summaryPlan.lifecycleProof.restoreExactDigest,
    );
    assert.equal(
      summaryLine.summary().diagnostics.latest.mutationResponseRollbackDigest,
      summaryPlan.lifecycleProof.rollbackDigest,
    );
    assert.equal(
      summaryLine.summary().diagnostics.latest.mutationResponseMergeRebaseDigest,
      summaryPlan.lifecycleProof.mergeRebaseDigest,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("save response plans name detail JSON path and region merge proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "mutation-response-detail-granular-proof");
    const jsonPathRead = runtime.signals.resource.detailJsonPaths({
      title: { path: ["document", "title"] },
    });
    const jsonPathFamily = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/workflow-title/:workflowId")
      .detail({
        reconcile: jsonPathRead,
        load: ({ workflowId }) => ({
          id: workflowId,
          document: { title: "First" },
        }),
      });
    jsonPathFamily.line({ workflowId: "wf-1" });
    const saveTitle = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/workflow-title/:workflowId")
      .response(runtime.signals.resource.response.detailJsonPaths()({
        title: { path: ["document", "title"] },
      }))
      .update({
        reconciles: [
          {
            family: jsonPathFamily,
            params: ({ workflowId }) => ({ workflowId }),
            fallback: "refetchRequired",
            detail: { kind: "jsonPath", path: "title" },
          },
        ],
        load: ({ workflowId, body }) => ({
          id: workflowId,
          document: { title: body.title },
        }),
      });
    const regionRead = runtime.signals.resource.detailRegions({
      graph: {
        read: (value) => value.graph,
        write: (value, graph) => ({ ...value, graph }),
        identityBoundary: "outside",
        mergeGranularity: "region-subtree",
        cost: {
          traversalBreadth: 2,
          reconstructionBreadth: 2,
        },
      },
    });
    const regionFamily = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/workflow-graph/:workflowId")
      .detail({
        reconcile: regionRead,
        load: ({ workflowId }) => ({
          id: workflowId,
          graph: { nodes: [{ id: "n1" }] },
        }),
      });
    regionFamily.line({ workflowId: "wf-1" });
    const saveGraph = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/workflow-graph/:workflowId")
      .response(runtime.signals.resource.response.detailRegions()({
        graph: {
          read: (value) => value.graph,
          write: (value, graph) => ({ ...value, graph }),
          identityBoundary: "outside",
          mergeGranularity: "region-subtree",
          cost: {
            traversalBreadth: 2,
            reconstructionBreadth: 2,
          },
        },
      }))
      .update({
        reconciles: [
          {
            family: regionFamily,
            params: ({ workflowId }) => ({ workflowId }),
            fallback: "refetchRequired",
            detail: { kind: "region", region: "graph" },
          },
        ],
        load: ({ workflowId, body }) => ({
          id: workflowId,
          graph: { nodes: body.nodes },
        }),
      });

    const jsonPathPlan = saveTitle.line({
      workflowId: "wf-1",
      body: { title: "Renamed" },
    }).mutationResponse();
    const regionPlan = saveGraph.line({
      workflowId: "wf-1",
      body: { nodes: [{ id: "n2" }] },
    }).mutationResponse();

    assert.equal(
      jsonPathPlan.lifecycleProof.entries[0].mergeRebase.granularity,
      "jsonPath:title",
    );
    assert.equal(
      jsonPathPlan.lifecycleProof.entries[0].mergeRebase.locusKind,
      "detailJsonPath",
    );
    assert.equal(
      regionPlan.lifecycleProof.entries[0].mergeRebase.granularity,
      "region:graph:region-subtree",
    );
    assert.equal(
      regionPlan.lifecycleProof.entries[0].mergeRebase.locusKind,
      "detailRegion",
    );
    assert.match(
      jsonPathPlan.lifecycleProof.mergeRebaseDigest,
      /unavailable:jsonPath:title/,
    );
    assert.match(
      regionPlan.lifecycleProof.mergeRebaseDigest,
      /unavailable:region:graph:region-subtree/,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("save response plans name collection item merge proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "mutation-response-collection-proof");
    const collectionResponse = runtime.signals.resource.response.array({
      itemId: (item) => item.id,
    });
    const taskList = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/tasks")
      .response(collectionResponse)
      .list({
        load: () => [{ id: "t1", title: "First" }],
      });
    taskList.line({});
    const saveTask = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    })
      .url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .update({
        reconciles: [
          {
            family: taskList,
            params: () => ({}),
            fallback: "refetchRequired",
            collection: { kind: "item" },
          },
        ],
        load: ({ taskId, body }) => ({ id: taskId, title: body.title }),
      });

    const plan = saveTask.line({
      taskId: "t1",
      body: { title: "Updated" },
    }).mutationResponse();

    assert.equal(plan.executionArtifacts[0].kind, "exactCollectionItem");
    assert.equal(plan.lifecycleProof.entries[0].replayExact.kind, "available");
    assert.equal(plan.lifecycleProof.entries[0].restoreExact.kind, "available");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "item:t1");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.locusKind, "membership");
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:item:t1/);
    assert.equal(plan.counters.lifecycleProofBreadth, 1);
  } finally {
    await runtime.cleanup();
  }
});

test("fallback-only save response plans carry typed lifecycle unavailability", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const readFamily = runtime.signals.api({}).url("/users/:userId").detail({
      load: ({ userId }) => ({ id: userId, name: "First" }),
    });
    const saveUser = runtime.signals.api({}).url("/users/:userId")
      .response(runtime.signals.resource.response.detail()())
      .update({
        reconciles: [
          {
            family: readFamily,
            params: ({ userId }) => ({ userId }),
            fallback: "deliveryAwaited",
          },
        ],
        load: ({ userId, body }) => ({ id: userId, name: body.name }),
      });

    const plan = saveUser.line({
      userId: "u1",
      body: { name: "Queued" },
    }).mutationResponse();

    assert.equal(plan.lifecycleProof.entries[0].rollback.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[0].replayExact.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[0].restoreExact.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "deliveryAwaited");
    assert.equal(plan.counters.lifecycleProofBreadth, 1);
  } finally {
    await runtime.cleanup();
  }
});
