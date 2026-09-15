import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createTreeTasks } from "../../../runtime_fixture/tree_collection_runtime_fixture.mjs";

test("create responses can insert tree items through declared append placement", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const treeTasks = createTreeTasks(runtime, "/tree-tasks", {
      nodeForItem: (itemId) => itemId === "root" ? ["root"] : ["root", itemId],
    });
    const line = treeTasks.line({});

    const plan = runtime.signals.api({}).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: treeTasks,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "append" },
        }],
        load: ({ body }) => body,
      })
      .line({
        body: { id: "task:2", title: "Second", children: [] },
      })
      .mutationResponse();

    assert.deepEqual(line.value(), {
      roots: [{
        id: "root",
        title: "Root",
        children: [
          { id: "task:1", title: "First", children: [] },
          { id: "task:2", title: "Second", children: [] },
        ],
      }],
    });
    assert.equal(plan.executionArtifacts[0].kind, "exactCollectionInsert");
    assert.equal(plan.executionArtifacts[0].deliveryKind, "patch");
    assert.equal(plan.confirmation.kind, "consumedCanonicalTruth");
    assert.deepEqual(line.diagnostics().lastEffect.locusProof.cost, {
      lookup: "tree-descendant-path",
      lookupBreadth: 1,
      traversal: "single-descendant-path",
      traversalBreadth: 1,
      reconstruction: "replaceChildrenOrRoots",
      reconstructionBreadth: 1,
    });
  } finally {
    await runtime.cleanup();
  }
});

test("create responses deny tree insert when the declared parent path does not exist", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const treeTasks = createTreeTasks(runtime, "/tree-tasks", {
      nodeForItem: (itemId) => ["missing-parent", itemId],
    });
    const line = treeTasks.line({});
    const before = structuredClone(line.value());

    const created = runtime.signals.api({}).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: treeTasks,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "append" },
        }],
        load: ({ body }) => body,
      })
      .line({
        body: { id: "task:2", title: "Second", children: [] },
      });

    // The denial settles the write line rejected with the reason; the target
    // tree is untouched.
    assert.equal(created.status().kind, "rejected");
    assert.match(
      created.status().message,
      /tree parent path "missing-parent" to resolve an existing parent node/,
    );
    assert.equal(created.mutationResponse(), null);

    assert.deepEqual(line.value(), before);
    assert.equal(line.diagnostics().lastEffect, null);
  } finally {
    await runtime.cleanup();
  }
});
