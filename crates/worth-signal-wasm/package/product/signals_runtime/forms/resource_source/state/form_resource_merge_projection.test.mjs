import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../../resource_runtime/runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../../resource_runtime/runtime_fixture/real_resource_signals.mjs";

test("signals.form projects resource merge conflicts into fields sections messages readiness and verification", async () => {
  const runtime = await createRealRequestRuntime();
  let restoreResource = null;
  try {
    const { signals } = runtime;
    createBranchHead(signals, "forms-resource-merge-conflict");
    const line = createDetailFieldLine(signals).line({ profileId: "p1" });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-merge-conflict" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
      availability: ({ section }) => ({
        details: section("details", ["title"], ["title"], () => true),
      }),
    });

    form.fields.title.set("Local");
    await form.executeAction("submit");
    const effect = line.diagnostics().lastEffect;
    const resourceWithConflictHistory = runtime.mod.createResourceNamespace(null, {
      history() {
        return {
          plan_merge_policy_preview_with_proof(request) {
            return createConflictPreviewEnvelope(
              signals.history().plan_merge_policy_preview_with_proof(request),
              request,
            );
          },
        };
      },
    });
    const originalResource = signals.resource;
    restoreResource = () => {
      signals.resource = originalResource;
    };
    signals.resource = { ...originalResource, branch: resourceWithConflictHistory.branch };

    const preview = form.previewResourceMerge({
      source_branch_id: effect.optimistic.branchId,
      target_branch_id: 0,
    });

    assert.equal(preview.status, "conflict");
    assert.equal(preview.conflictCount, 1);
    assert.deepEqual(preview.projectedFields, ["title"]);
    assert.deepEqual(preview.projectedSections, ["details"]);
    assert.equal(preview.messages[0].code, "resource.merge.conflict");
    assert.equal(preview.messages[0].target, "title");
    assert.equal(preview.blockers[0].kind, "resource:mergeConflict");
    assert.equal(preview.blockers[0].field, "title");
    assert.equal(preview.blockers[0].section, "details");

    assert.equal(form.resourceMerge().current.status, "conflict");
    assert.equal(form.resourceMerge().summary.fieldCount, 1);
    assert.equal(form.resourceMerge().summary.sectionCount, 1);
    assert.equal(form.messages().summary.semanticVisibleCount, 1);
    assert.equal(form.visibleMessages().at(-1).code, "resource.merge.conflict");
    assert.equal(
      form.readiness().blockers.some((blocker) => blocker.kind === "resource:mergeConflict"),
      true,
    );
    assert.equal(
      form.actionPlan("submit").recoveryActions.some((action) => action.kind === "rollbackLastResourceEffect"),
      true,
    );
    assert.equal(
      form.actionPlan("submit").recoveryActions.some((action) => action.kind === "restoreExactResourceSource"),
      true,
    );
    const submitAttempt = form.attemptAction("submit");
    assert.equal(
      submitAttempt.recoveryActions.some((action) => action.kind === "rollbackLastResourceEffect"),
      true,
    );
    assert.equal(
      submitAttempt.recoveryActions.some((action) => action.kind === "restoreExactResourceSource"),
      true,
    );
    assert.equal(form.steps().artifacts.length, 0);
    assert.equal(form.verification().digests.resourceMergeDigest, form.resourceMerge().digest);
    assert.equal(typeof form.verification().digests.resourceMergeHistoryDigest, "string");
    assert.equal(form.verification().performanceEnvelope.resourceMerge.conflictPreviews, 1);
  } finally {
    restoreResource?.();
    await runtime.cleanup();
  }
});

test("signals.form projects mapping-unavailable resource merge posture without inventing field conflicts", async () => {
  const runtime = await createRealRequestRuntime();
  let restoreResource = null;
  try {
    const { signals, signalsMod: mod } = runtime;
    createBranchHead(signals, "forms-resource-merge-mapping");
    const line = createUnmappedCollectionLine(runtime);
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-merge-mapping" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    await line.patch(mod.resourcePatch.itemAspect({
      itemId: "demo:1",
      aspect: "title",
      value: "Unmapped conflict",
    }));
    const effect = line.diagnostics().lastEffect;
    const resourceWithConflictHistory = runtime.mod.createResourceNamespace(null, {
      history() {
        return {
          plan_merge_policy_preview_with_proof(request) {
            return createConflictPreviewEnvelope(
              signals.history().plan_merge_policy_preview_with_proof(request),
              request,
              "unmapped",
            );
          },
        };
      },
    });
    const originalResource = signals.resource;
    restoreResource = () => {
      signals.resource = originalResource;
    };
    signals.resource = { ...originalResource, branch: resourceWithConflictHistory.branch };

    const preview = form.previewResourceMerge({
      source_branch_id: effect.optimistic.branchId,
      target_branch_id: 0,
    });

    assert.equal(preview.status, "unavailable");
    assert.equal(preview.conflictCount, 1);
    assert.deepEqual(preview.projectedFields, []);
    assert.equal(preview.blockers[0].kind, "resource:mergeMappingUnavailable");
    assert.equal(preview.messages[0].code, "resource.merge.mapping_unavailable");
    assert.equal(form.visibleMessages().at(-1).code, "resource.merge.mapping_unavailable");
  } finally {
    restoreResource?.();
    await runtime.cleanup();
  }
});

test("signals.form marks stored merge previews stale when the backing resource effect changes", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    createBranchHead(signals, "forms-resource-merge-stale");
    const line = createDetailFieldLine(signals).line({ profileId: "p1" });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-merge-stale" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    form.fields.title.set("Local");
    await form.executeAction("submit");
    const effect = line.diagnostics().lastEffect;
    form.previewResourceMerge({
      source_branch_id: effect.optimistic.branchId,
      target_branch_id: 0,
    });

    form.fields.title.set("Local again");
    await form.executeAction("submit");

    assert.equal(form.resourceMerge().current.stale, true);
    assert.equal(
      form.readiness().blockers.some((blocker) => blocker.kind === "resource:mergeConflict"),
      false,
    );
    assert.equal(
      form.visibleMessages().some((message) => message.code === "resource.merge.conflict"),
      false,
    );
    assert.equal(form.resourceMerge().summary.stale, true);
  } finally {
    await runtime.cleanup();
  }
});

test("signals.form reports unavailable merge preview posture when no resource effect exists yet", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    createBranchHead(signals, "forms-resource-merge-no-effect");
    const line = createDetailFieldLine(signals).line({ profileId: "p1" });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-merge-no-effect" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    const preview = form.previewResourceMerge({
      source_branch_id: signals.history().current_branch().id,
      target_branch_id: 0,
    });

    assert.equal(preview.status, "unavailable");
    assert.match(preview.reason, /requires a current resource line effect/);
    assert.equal(form.readiness().canSubmit, false);
    assert.equal(
      form.readiness().blockers.some((blocker) => blocker.kind === "resource:mergeConflict"),
      false,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("signals.form reports unavailable merge preview posture when the form source is not a resource line", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const form = signals.form({
      source: { title: "Local only" },
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    const preview = form.previewResourceMerge({
      source_branch_id: 0,
      target_branch_id: 0,
    });

    assert.equal(preview.status, "unavailable");
    assert.equal(preview.reason, "form source is not a resource line");
    assert.equal(preview.sourceKind, "form");
    assert.equal(form.resourceMerge().current.reason, "form source is not a resource line");
  } finally {
    await runtime.cleanup();
  }
});

test("signals.form verification stays bounded across repeated resource merge previews", async () => {
  const runtime = await createRealRequestRuntime();
  let restoreResource = null;
  try {
    const { signals } = runtime;
    createBranchHead(signals, "forms-resource-merge-history");
    const line = createDetailFieldLine(signals).line({ profileId: "p1" });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-merge-history" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
      availability: ({ section }) => ({
        details: section("details", ["title"], ["title"], () => true),
      }),
    });

    form.fields.title.set("Local");
    await form.executeAction("submit");
    const effect = line.diagnostics().lastEffect;
    const resourceWithConflictHistory = runtime.mod.createResourceNamespace(null, {
      history() {
        return {
          plan_merge_policy_preview_with_proof(request) {
            return createConflictPreviewEnvelope(
              signals.history().plan_merge_policy_preview_with_proof(request),
              request,
            );
          },
        };
      },
    });
    const originalResource = signals.resource;
    restoreResource = () => {
      signals.resource = originalResource;
    };
    signals.resource = { ...originalResource, branch: resourceWithConflictHistory.branch };

    for (let iteration = 0; iteration < 24; iteration += 1) {
      const preview = form.previewResourceMerge({
        source_branch_id: effect.optimistic.branchId,
        target_branch_id: 0,
      });
      assert.equal(preview.status, "conflict");
    }

    const verification = form.verification();
    assert.equal(form.resourceMerge().history.length, 24);
    assert.equal(verification.performanceEnvelope.resourceMerge.previews, 24);
    assert.equal(typeof verification.digests.resourceMergeDigest, "string");
    assert.equal(typeof verification.digests.resourceMergeHistoryDigest, "string");
    assert.equal(typeof verification.packageDigest, "string");
  } finally {
    restoreResource?.();
    await runtime.cleanup();
  }
});

function createDetailFieldLine(signals) {
  const response = signals.resource.response.detail()({
    title: "title",
  });
  return signals.api({
    effects: signals.resource.effects.branchNative(),
  })
    .url("/profiles/:profileId")
    .response(response)
    .detail({
      load: ({ profileId }) => ({ id: profileId, title: "Loaded" }),
    });
}

function createUnmappedCollectionLine(runtime) {
  const { signals, signalsMod: mod } = runtime;
  const resource = signals.resource;
  const family = resource.collection({
    params: mod.resourceParams(),
    normalizeParams: ({ workspaceId }) =>
      mod.resourceParamIdentity({ workspaceId }, workspaceId),
    requestContext: mod.resourceRequestContext({
      correlationId: "trace-demo",
      branchId: "branch-demo",
      basisId: "basis-1",
    }),
    effects: resource.effects.branchNative(),
    itemIdentity: (item) => item.id,
    reconcile: mod.resourceCollectionShape({
      items: (value) => value.items,
      replaceItems: (value, nextItems) => ({ ...value, items: [...nextItems] }),
      aspects: mod.resourceItemAspects({
        title: {
          read: (item) => item.title,
          write: (item, title) => ({ ...item, title: String(title) }),
        },
      }),
    }),
    load: () => ({
      items: [{ id: "demo:1", title: "Loaded" }],
    }),
  });
  return family.line({ workspaceId: "demo" });
}

function createConflictPreviewEnvelope(baseEnvelope, request, target = "title") {
  return {
    ...baseEnvelope,
    plan: {
      ...baseEnvelope.plan,
      source_branch_id: request.source_branch_id,
      target_branch_id: request.target_branch_id,
      resolution_plan: {
        source_branch_id: request.source_branch_id,
        target_branch_id: request.target_branch_id,
        divergence: "ConflictingOutputs",
        records: [{
          source_node: `resource.effect.source.${target}`,
          target_node: `resource.effect.target.${target}`,
          required_resolution: ["Manual"],
          supported_strategies: ["Manual"],
        }],
      },
    },
  };
}
