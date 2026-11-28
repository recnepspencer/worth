import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("create response plans carry lifecycle proof for exact placement plus exact identity migration", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "create-lifecycle-proof");
    const taskList = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks")
      .response(runtime.signals.resource.response.array({
        itemId: (item) => item.id,
      }))
      .list({
        load: () => [{ id: "task:existing", title: "Existing" }],
      });
    const draftDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .detail({
        load: ({ taskId }) => ({ id: taskId, title: "Draft" }),
      });
    taskList.line({});
    draftDetail.line({ taskId: "tmp-lifecycle-create-1" });

    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: taskList,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "append" },
        }],
        identity: {
          submitted: ({ body }) => body.id,
          response: (value) => value.id,
          canonical: (value, responseIdentity) => responseIdentity ?? value.id,
          targets: [{
            family: draftDetail,
            params: ({ body }) => ({ taskId: body.id }),
            canonicalParams: (_params, _value, canonicalIdentity) => ({
              taskId: canonicalIdentity,
            }),
            fallback: "identityMigrationUnavailable",
          }],
        },
        load: ({ body }) => ({ id: `task:${body.id}`, title: body.title }),
      })
      .line({
        body: { id: "tmp-lifecycle-create-1", title: "Created" },
      })
      .mutationResponse();

    assert.equal(plan.lifecycleProof.count, 2);
    assert.equal(plan.lifecycleProof.entries[0].entryKind, "reconciliation");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "item:task:tmp-lifecycle-create-1");
    assert.equal(plan.lifecycleProof.entries[1].entryKind, "identityMigration");
    assert.equal(plan.lifecycleProof.entries[1].rollback.kind, "identityMigrationUnavailable");
    assert.equal(plan.lifecycleProof.entries[1].mergeRebase.kind, "identityMigrationUnavailable");
    assert.equal(plan.counters.lifecycleProofBreadth, 2);
    assert.equal(plan.identityMigration.counters.lifecycleProofBreadth, 1);
    assert.match(plan.lifecycleProof.rollbackDigest, /identityMigrationUnavailable/);
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:item:task:tmp-lifecycle-create-1/);
  } finally {
    await runtime.cleanup();
  }
});

test("create response lifecycle proof keeps exact placement and typed identity fallback distinct", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "create-lifecycle-fallback");
    const taskList = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks")
      .response(runtime.signals.resource.response.array({
        itemId: (item) => item.id,
      }))
      .list({
        load: () => [],
      });
    const taskDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(runtime.signals.resource.response.detail()())
      .detail({
        load: ({ taskId }) => ({ id: taskId, title: "Draft" }),
      });
    taskList.line({});
    taskDetail.line({ taskId: "tmp-lifecycle-create-2" });
    taskDetail.line({ taskId: "task:tmp-lifecycle-create-2" });

    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: taskList,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "append" },
        }],
        identity: {
          submitted: ({ body }) => body.id,
          response: (value) => value.id,
          canonical: (value, responseIdentity) => responseIdentity ?? value.id,
          atomicity: "partialAllowed",
          targets: [{
            family: taskDetail,
            params: ({ body }) => ({ taskId: body.id }),
            canonicalParams: (_params, _value, canonicalIdentity) => ({
              taskId: canonicalIdentity,
            }),
            fallback: "identityMigrationUnavailable",
          }],
        },
        load: ({ body }) => ({ id: `task:${body.id}`, title: body.title }),
      })
      .line({
        body: { id: "tmp-lifecycle-create-2", title: "Created" },
      })
      .mutationResponse();

    assert.equal(plan.lifecycleProof.count, 2);
    assert.equal(plan.lifecycleProof.entries[0].entryKind, "reconciliation");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[1].entryKind, "identityMigration");
    assert.equal(plan.lifecycleProof.entries[1].rollback.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[1].mergeRebase.kind, "fallbackUnavailable");
    assert.equal(plan.lifecycleProof.entries[1].mergeRebase.granularity, "identityMigrationUnavailable");
    assert.equal(plan.confirmation.kind, "partialCanonicalTruth");
    assert.deepEqual(plan.confirmation.fallbackKinds, ["identityMigrationUnavailable"]);
  } finally {
    await runtime.cleanup();
  }
});

test("create response lifecycle proof carries effect-backed detail-child migration entries under branch-native effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "create-lifecycle-detail-child");
    const taskRegions = runtime.signals.resource.detailRegions({
      children: {
        read: (value) => value.children,
        write: (value, children) => ({ ...value, children }),
        identityBoundary: "inside",
        mergeGranularity: "child-list",
        cost: {
          traversalBreadth: 1,
          reconstructionBreadth: 1,
        },
      },
    });
    const taskDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .detail({
        reconcile: taskRegions,
        load: ({ taskId }) => ({
          id: taskId,
          title: "Task",
          children: [{ id: "tmp-child-lifecycle-1", title: "Draft child" }],
        }),
      });
    taskDetail.line({ taskId: "task-lifecycle-1" });

    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId/children")
      .response(runtime.signals.resource.response.detailRegions()(taskRegions))
      .create({
        identity: {
          submitted: ({ body }) => body.id,
          response: (value) => value.children.at(-1)?.id ?? value.id,
          canonical: (value, responseIdentity) => responseIdentity ?? value.id,
          targets: [{
            family: taskDetail,
            params: ({ taskId }) => ({ taskId }),
            fallback: "identityMigrationUnavailable",
            detailChild: {
              kind: "detailChild",
              region: "children",
            },
          }],
        },
        load: ({ taskId, body }) => ({
          id: taskId,
          title: "Task",
          children: [{ id: `child:${body.id}`, title: body.title }],
        }),
      })
      .line({
        taskId: "task-lifecycle-1",
        body: { id: "tmp-child-lifecycle-1", title: "Draft child" },
      })
      .mutationResponse();

    assert.equal(plan.lifecycleProof.count, 1);
    assert.equal(plan.lifecycleProof.entries[0].entryKind, "identityMigration");
    assert.equal(plan.lifecycleProof.entries[0].effectId, plan.identityMigration.targets[0].execution.effectId);
    assert.equal(plan.lifecycleProof.entries[0].rollback.kind, "notApplicable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "unavailable");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.locusKind, "detailRegion");
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.granularity, "region:children:child-list");
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:region:children:child-list:detailRegion/);
  } finally {
    await runtime.cleanup();
  }
});

test("create response lifecycle proof denies exact detail-child migration under all-or-none atomicity when a sibling target falls back", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    createBranchHead(runtime.signals, "create-lifecycle-detail-child-atomicity");
    const taskRegions = runtime.signals.resource.detailRegions({
      children: {
        read: (value) => value.children,
        write: (value, children) => ({ ...value, children }),
        identityBoundary: "inside",
        mergeGranularity: "child-list",
        cost: {
          traversalBreadth: 1,
          reconstructionBreadth: 1,
        },
      },
    });
    const taskDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .detail({
        reconcile: taskRegions,
        load: ({ taskId }) => ({
          id: taskId,
          title: "Task",
          children: [{ id: "tmp-child-lifecycle-atomicity", title: "Draft child" }],
        }),
      });
    const auditDetail = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/task-audit/:taskId")
      .detail({
        load: ({ taskId }) => ({ id: taskId, title: "Audit" }),
      });
    const taskLine = taskDetail.line({ taskId: "task-lifecycle-atomicity" });
    auditDetail.line({ taskId: "tmp-child-lifecycle-atomicity" });
    auditDetail.line({ taskId: "child:tmp-child-lifecycle-atomicity" });

    const plan = runtime.signals.api({
      effects: runtime.signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId/children")
      .response(runtime.signals.resource.response.detailRegions()(taskRegions))
      .create({
        identity: {
          submitted: ({ body }) => body.id,
          response: (value) => value.children.at(-1)?.id ?? value.id,
          canonical: (value, responseIdentity) => responseIdentity ?? value.id,
          targets: [{
            family: taskDetail,
            params: ({ taskId }) => ({ taskId }),
            fallback: "identityMigrationUnavailable",
            detailChild: {
              kind: "detailChild",
              region: "children",
            },
          }, {
            family: auditDetail,
            params: ({ body }) => ({ taskId: body.id }),
            canonicalParams: (_params, _value, canonicalIdentity) => ({
              taskId: canonicalIdentity,
            }),
            fallback: "identityMigrationUnavailable",
          }],
        },
        load: ({ taskId, body }) => ({
          id: taskId,
          title: "Task",
          children: [{ id: `child:${body.id}`, title: body.title }],
        }),
      })
      .line({
        taskId: "task-lifecycle-atomicity",
        body: {
          id: "tmp-child-lifecycle-atomicity",
          title: "Draft child",
        },
      })
      .mutationResponse();

    assert.equal(plan.identityMigration.partialAdmission, "denied");
    assert.equal(plan.identityMigration.exactTargetCount, 0);
    assert.equal(plan.identityMigration.targets[0].outcome, "fallback");
    assert.equal(plan.identityMigration.targets[0].execution.kind, "fallback");
    assert.equal(
      plan.identityMigration.targets[0].execution.fallback,
      "identityMigrationUnavailable",
    );
    assert.equal(plan.lifecycleProof.entries[0].mergeRebase.kind, "fallbackUnavailable");
    assert.deepEqual(taskLine.value().children, [{
      id: "tmp-child-lifecycle-atomicity",
      title: "Draft child",
    }]);
    assert.notEqual(taskLine.diagnostics().lastPatchedRegion, "children");
  } finally {
    await runtime.cleanup();
  }
});
