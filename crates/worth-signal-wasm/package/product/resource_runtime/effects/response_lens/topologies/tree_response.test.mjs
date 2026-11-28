import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("tree responses lower item replacement through recursive tree loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "recursive-tree");
    let rootReplacementCount = 0;
    let singleNodeReplacementCount = 0;
    const response = createTaskTreeResponse(signals, {
      replaceRoots(value, roots) {
        rootReplacementCount += 1;
        return { ...value, roots };
      },
      replaceChildren(node, nextChildren) {
        return { ...node, children: nextChildren };
      },
      replaceNode(value, path, itemId, nextNode) {
        singleNodeReplacementCount += 1;
        return {
          ...value,
          roots: replaceTreeNode(value.roots, path, itemId, nextNode),
        };
      },
    });
    assert.equal(response.lensProof.topology, "recursiveTree");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "recursiveTree" && row.patchScope === "item",
    ), true);

    const tasks = createTaskTreeApi(signals, response, "/tree", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced", children: [] },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.equal(readTreeNode(line.value(), ["root", "task:1"]).title, "Replaced");
    assert.equal(rootReplacementCount, 0);
    assert.equal(singleNodeReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "recursiveTree",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.tree<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "recursiveTree");
    assert.equal(itemEffect.locusProof.locus, "recursiveTree");
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "tree-descendant-path",
      lookupBreadth: 1,
      traversal: "single-descendant-path",
      traversalBreadth: 1,
      reconstruction: "replaceNode",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-tree",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", title: "Delivered", children: [] },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(readTreeNode(line.value(), ["root", "task:1"]).title, "Delivered");
    assert.equal(deliveryEffect.locus.kind, "recursiveTree");
    assert.equal(deliveryEffect.locusProof.locus, "recursiveTree");
    assert.deepEqual(deliveryEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.deepEqual(deliveryEffect.optimistic.confirmation, {
      kind: "consumedCanonicalServerTruth",
      previousEffectId: itemEffect.effectId,
      previousPlanId: itemEffect.plan.planId,
      previousBranchId: itemEffect.optimistic.branchId,
      previousSnapshotId: itemEffect.optimistic.snapshotId,
      locusMatches: true,
      valueChanged: true,
      detail:
        "server delivery consumed canonical server truth after a pending speculative resource effect",
    });
    assert.equal(singleNodeReplacementCount, 2);

    await line.patch(tasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Aspect",
    }));
    const aspectEffect = line.diagnostics().lastEffect;
    assert.equal(aspectEffect.locus.kind, "itemAspect");
    assert.equal(aspectEffect.locusProof.locus, "itemAspect");
    assert.deepEqual(aspectEffect.locusProof.cost, itemEffect.locusProof.cost);
    assert.equal(readTreeNode(line.value(), ["root", "task:1"]).title, "Aspect");
    assert.equal(singleNodeReplacementCount, 5);
  } finally {
    await runtime.cleanup();
  }
});

test("tree branch-native inverse capture stays on declared descendant path", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    createBranchHead(signals, "recursive-tree-path-local-inverse");
    const response = createTaskTreeResponse(signals, {
      roots: () => [{
        id: "root",
        title: "Root",
        children: [
          { id: "task:1", title: "First", children: [] },
          { id: "off-path", title: "Off Path", children: [] },
        ],
      }],
      children(node) {
        if (node.id === "off-path") {
          throw new Error("off-path descendants must not be traversed");
        }
        return node.children;
      },
    });
    const tasks = createTaskTreeApi(signals, response, "/tree-path-local-inverse", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});

    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Path Local", children: [] },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.equal(readTreeNode(line.value(), ["root", "task:1"]).title, "Path Local");
    assert.equal(effect.locus.kind, "recursiveTree");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "tree-descendant-path",
      lookupBreadth: 1,
      traversal: "single-descendant-path",
      traversalBreadth: 1,
      reconstruction: "replaceNode",
      reconstructionBreadth: 1,
    });
  } finally {
    await runtime.cleanup();
  }
});

test("tree broad replacements preserve recursive tree topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTreeResponse(signals);
    const tasks = createTaskTreeApi(signals, response, "/tree-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      roots: [{ id: "root:2", title: "Broad", children: [] }],
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "recursiveTree");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-tree-roots",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceRoots",
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

test("tree broad replacements deny duplicated recursive ids before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTreeResponse(signals);
    const tasks = createTaskTreeApi(signals, response, "/tree-broad-denial");
    const line = tasks.line({});

    assertTreePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.replace({
      roots: [{
        id: "root",
        title: "Root",
        children: [{ id: "root", title: "Duplicate", children: [] }],
      }],
    })), /duplicated tree node id "root"/);
  } finally {
    await runtime.cleanup();
  }
});

test("tree responses deny invalid descendant lookup before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTreeResponse(signals, {
      nodeForItem: () => [],
    });
    const tasks = createTaskTreeApi(signals, response, "/bad-tree-lookup");
    const line = tasks.line({});

    assertTreePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced", children: [] },
    })), /non-empty descendant path/);
  } finally {
    await runtime.cleanup();
  }
});

test("tree responses deny malformed child boundaries before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTreeResponse(signals, {
      children: (node) => node.id === "root"
        ? { id: "task:1", title: "Task", children: [] }
        : node.children,
    });
    const tasks = createTaskTreeApi(signals, response, "/bad-tree-children");
    const line = tasks.line({});

    assertTreePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced", children: [] },
    })), /children\("root"\) to be an array of tree nodes/);
  } finally {
    await runtime.cleanup();
  }
});

test("tree responses deny duplicate descendant ids before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTreeResponse(signals, {
      roots: () => [{
        id: "root",
        title: "Root",
        children: [
          { id: "task:1", title: "First", children: [] },
          { id: "task:1", title: "Duplicate", children: [] },
        ],
      }],
    });
    const tasks = createTaskTreeApi(signals, response, "/duplicate-tree");
    const line = tasks.line({});

    assertTreePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced", children: [] },
    })), /duplicated tree node id "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

test("tree replaceNode must preserve descendant identity", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskTreeResponse(signals, {
      replaceNode(value, path, itemId, nextNode) {
        return {
          ...value,
          roots: replaceTreeNode(value.roots, path, itemId, {
            ...nextNode,
            id: "task:2",
          }),
        };
      },
    });
    const tasks = createTaskTreeApi(signals, response, "/corrupt-tree");
    const line = tasks.line({});

    assertTreePatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced", children: [] },
    })), /preserve tree node "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

function assertTreePatchDeniedWithoutSideEffects(line, patchAction, errorPattern) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, errorPattern);
  assert.deepEqual(line.value(), beforeValue);
  assert.equal(line.diagnostics().lastEffect, beforeEffect);
}

function createTaskTreeResponse(signals, overrides = {}) {
  return signals.resource.response.tree()({
    itemId: (node) => node.id,
    roots: overrides.roots ?? ((value) => value.roots),
    children: overrides.children ?? ((node) => node.children),
    replaceChildren: overrides.replaceChildren ?? ((node, nextChildren) => ({
      ...node,
      children: nextChildren,
    })),
    replaceRoots: overrides.replaceRoots ?? ((value, roots) => ({ ...value, roots })),
    nodeForItem: overrides.nodeForItem ?? (
      (itemId) => itemId === "root" ? ["root"] : ["root", itemId]
    ),
    replaceNode: overrides.replaceNode ?? (
      (value, path, itemId, nextNode) => ({
        ...value,
        roots: replaceTreeNode(value.roots, path, itemId, nextNode),
      })
    ),
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskTreeApi(signals, response, url, apiOptions = {}) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({
      load: () => ({
        roots: [{
          id: "root",
          title: "Root",
          children: [{ id: "task:1", title: "First", children: [] }],
        }],
      }),
    });
}

function replaceTreeNode(nodes, path, itemId, nextNode) {
  const [head, ...tail] = path;
  return nodes.map((node) => {
    if (node.id !== head) {
      return node;
    }
    if (tail.length === 0 && node.id === itemId) {
      return nextNode;
    }
    return {
      ...node,
      children: replaceTreeNode(node.children, tail, itemId, nextNode),
    };
  });
}

function readTreeNode(value, path) {
  let nodes = value.roots;
  let foundNode = null;
  for (const segment of path) {
    foundNode = nodes.find((node) => node.id === segment) ?? null;
    if (foundNode === null) {
      return null;
    }
    nodes = foundNode.children;
  }
  return foundNode;
}
