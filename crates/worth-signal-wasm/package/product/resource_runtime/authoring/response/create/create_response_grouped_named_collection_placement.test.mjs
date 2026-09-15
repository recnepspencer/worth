import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";

test("create responses can insert grouped items while preserving empty sibling groups", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const groupedTasks = runtime.signals.api({}).url("/grouped-tasks")
      .response(runtime.signals.resource.response.grouped()({
        itemId: (task) => task.id,
        groupId: (task) => task.group,
        groupForItem: () => "todo",
        groups: (value) => value.groups,
        replaceGroups: (value, groups) => ({ ...value, groups }),
        replaceGroupItem: (value, groupId, itemId, nextItem) => ({
          ...value,
          groups: Object.fromEntries(
            Object.entries(value.groups).map(([key, items]) => [
              key,
              key === groupId
                ? items.map((item) => item.id === itemId ? nextItem : item)
                : items,
            ]),
          ),
        }),
      }))
      .list({
        load: () => ({
          groups: {
            todo: [{ id: "task:1", group: "todo", title: "First" }],
            done: [],
          },
        }),
      });
    const line = groupedTasks.line({});

    const plan = runtime.signals.api({}).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: groupedTasks,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "append" },
        }],
        load: ({ body }) => body,
      })
      .line({
        body: { id: "task:2", group: "todo", title: "Second" },
      })
      .mutationResponse();

    assert.deepEqual(line.value(), {
      groups: {
        todo: [
          { id: "task:1", group: "todo", title: "First" },
          { id: "task:2", group: "todo", title: "Second" },
        ],
        done: [],
      },
    });
    assert.equal(plan.executionArtifacts[0].kind, "exactCollectionInsert");
    assert.deepEqual(line.diagnostics().lastEffect.locusProof.cost, {
      lookup: "group-key-item-id",
      lookupBreadth: 1,
      traversal: "single-group",
      traversalBreadth: 1,
      reconstruction: "replaceGroups",
      reconstructionBreadth: 1,
    });
  } finally {
    await runtime.cleanup();
  }
});

test("create responses can insert named collection items while preserving empty sibling collections", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const namedTasks = runtime.signals.api({}).url("/named-tasks")
      .response(runtime.signals.resource.response.named()({
        itemId: (task) => task.id,
        collectionId: (task) => task.collection,
        collectionForItem: () => "backlog",
        collections: (value) => value.collections,
        replaceCollections: (value, collections) => ({ ...value, collections }),
        replaceCollectionItem: (value, collectionId, itemId, nextItem) => ({
          ...value,
          collections: Object.fromEntries(
            Object.entries(value.collections).map(([key, items]) => [
              key,
              key === collectionId
                ? items.map((item) => item.id === itemId ? nextItem : item)
                : items,
            ]),
          ),
        }),
      }))
      .list({
        load: () => ({
          collections: {
            backlog: [{ id: "task:1", collection: "backlog", title: "First" }],
            active: [],
          },
        }),
      });
    const line = namedTasks.line({});

    const plan = runtime.signals.api({}).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: namedTasks,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "prepend" },
        }],
        load: ({ body }) => body,
      })
      .line({
        body: { id: "task:0", collection: "backlog", title: "Zeroth" },
      })
      .mutationResponse();

    assert.deepEqual(line.value(), {
      collections: {
        backlog: [
          { id: "task:0", collection: "backlog", title: "Zeroth" },
          { id: "task:1", collection: "backlog", title: "First" },
        ],
        active: [],
      },
    });
    assert.equal(plan.executionArtifacts[0].kind, "exactCollectionInsert");
    assert.deepEqual(line.diagnostics().lastEffect.locusProof.cost, {
      lookup: "collection-key-item-id",
      lookupBreadth: 1,
      traversal: "single-named-collection",
      traversalBreadth: 1,
      reconstruction: "replaceCollections",
      reconstructionBreadth: 1,
    });
  } finally {
    await runtime.cleanup();
  }
});

test("create responses deny grouped insert when lookup group disagrees with nextItem group", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const groupedTasks = runtime.signals.api({}).url("/grouped-tasks")
      .response(runtime.signals.resource.response.grouped()({
        itemId: (task) => task.id,
        groupId: (task) => task.group,
        groupForItem: (itemId) => itemId === "task:2" ? "done" : "todo",
        groups: (value) => value.groups,
        replaceGroups: (value, groups) => ({ ...value, groups }),
        replaceGroupItem: (value, groupId, itemId, nextItem) => ({
          ...value,
          groups: Object.fromEntries(
            Object.entries(value.groups).map(([key, items]) => [
              key,
              key === groupId
                ? items.map((item) => item.id === itemId ? nextItem : item)
                : items,
            ]),
          ),
        }),
      }))
      .list({
        load: () => ({
          groups: {
            todo: [{ id: "task:1", group: "todo", title: "First" }],
            done: [],
          },
        }),
      });
    const line = groupedTasks.line({});

    const createTask = runtime.signals.api({}).url("/tasks")
      .response(runtime.signals.resource.response.detail()())
      .create({
        reconciles: [{
          family: groupedTasks,
          params: () => ({}),
          fallback: "placementUnavailable",
          collection: { kind: "insert", placement: "append" },
        }],
        load: ({ body }) => body,
      });
    const created = createTask.line({
      body: { id: "task:2", group: "todo", title: "Second" },
    });

    // The denial settles the write line rejected with the reason; the target
    // collection is untouched.
    assert.equal(created.status().kind, "rejected");
    assert.match(
      created.status().message,
      /nextItem group id "todo" to match grouped lookup group id "done"/,
    );
    assert.equal(created.mutationResponse(), null);

    assert.deepEqual(line.value(), {
      groups: {
        todo: [{ id: "task:1", group: "todo", title: "First" }],
        done: [],
      },
    });
  } finally {
    await runtime.cleanup();
  }
});
