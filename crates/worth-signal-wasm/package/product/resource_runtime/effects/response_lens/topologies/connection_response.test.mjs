import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../runtime_fixture/real_request_runtime.mjs";
import { createBranchHead } from "../../../runtime_fixture/real_resource_signals.mjs";

test("connection responses lower item replacement through connection loci", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals, signalsMod } = runtime;
    createBranchHead(signals, "connection-response");
    let fullNodeReplacementCount = 0;
    let singleNodeReplacementCount = 0;
    const response = createTaskConnectionResponse(signals, {
      replaceNodes(value, nextNodes) {
        fullNodeReplacementCount += 1;
        return { ...value, edges: nextNodes.map(createTaskEdge) };
      },
      replaceNode(value, itemId, nextNode) {
        singleNodeReplacementCount += 1;
        return {
          ...value,
          edges: replaceConnectionNode(value.edges, itemId, nextNode),
        };
      },
    });
    assert.equal(response.lensProof.topology, "connection");
    assert.equal(response.lensProof.capabilityRows.some(
      (row) => row.locus === "connection" && row.patchScope === "item",
    ), true);

    const tasks = createTaskConnectionApi(signals, response, "/connection", {
      effects: signals.resource.effects.branchNative(),
    });
    const line = tasks.line({});
    await line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    }));
    const itemEffect = line.diagnostics().lastEffect;

    assert.equal(readTask(line.value(), "task:1").title, "Replaced");
    assert.equal(fullNodeReplacementCount, 0);
    assert.equal(singleNodeReplacementCount, 1);
    assert.deepEqual(itemEffect.locus, {
      kind: "connection",
      itemId: "task:1",
    });
    assert.equal(itemEffect.locusProof.lensSource, "resource.response.connection<T>()(...)");
    assert.equal(itemEffect.locusProof.topology, "connection");
    assert.equal(itemEffect.locusProof.locus, "connection");
    assert.deepEqual(itemEffect.locusProof.cost, {
      lookup: "connection-edge-item-id",
      lookupBreadth: 1,
      traversal: "single-connection-edge",
      traversalBreadth: 1,
      reconstruction: "replaceNode",
      reconstructionBreadth: 1,
    });
    assert.equal(itemEffect.optimistic.rollback.kind, "effectBranchRetirementAvailable");
    assert.equal(itemEffect.profile.rebase, "unavailable");

    line.deliver(signalsMod.resourceDelivery.patch({
      packetId: "pkt-connection",
      basisId: null,
      patch: tasks.patch.item({
        itemId: "task:1",
        nextItem: { id: "task:1", title: "Delivered" },
      }),
    }));
    const deliveryEffect = line.diagnostics().lastEffect;
    assert.equal(readTask(line.value(), "task:1").title, "Delivered");
    assert.equal(deliveryEffect.locus.kind, "connection");
    assert.deepEqual(deliveryEffect.locusProof.cost, itemEffect.locusProof.cost);
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
    assert.equal(readTask(line.value(), "task:1").title, "Aspect");
    assert.equal(singleNodeReplacementCount, 5);
  } finally {
    await runtime.cleanup();
  }
});

test("connection broad replacements preserve connection topology proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskConnectionResponse(signals);
    const tasks = createTaskConnectionApi(signals, response, "/connection-broad");
    const line = tasks.line({});

    await line.patch(tasks.patch.replace({
      edges: [createTaskEdge({ id: "task:2", title: "Broad" })],
      pageInfo: { hasNextPage: false },
    }));
    const effect = line.diagnostics().lastEffect;

    assert.deepEqual(effect.locus, { kind: "broadResponse" });
    assert.equal(effect.locusProof.topology, "connection");
    assert.equal(effect.locusProof.locus, "broadResponse");
    assert.deepEqual(effect.locusProof.cost, {
      lookup: "whole-connection-edges",
      lookupBreadth: 0,
      traversal: "whole-response",
      traversalBreadth: 1,
      reconstruction: "replaceNodes",
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

test("connection broad replacements deny malformed edges before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskConnectionResponse(signals);
    const tasks = createTaskConnectionApi(signals, response, "/connection-bad-broad");
    const line = tasks.line({});

    assertConnectionPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.replace({
      edges: { node: { id: "task:1", title: "Bad" } },
      pageInfo: { hasNextPage: false },
    })), /edges\(value\) to return an array of edges/);
  } finally {
    await runtime.cleanup();
  }
});

test("connection responses deny invalid item edge index before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskConnectionResponse(signals, {
      edgeIndexForItem: () => 1,
    });
    const tasks = createTaskConnectionApi(signals, response, "/connection-bad-index");
    const line = tasks.line({});

    assertConnectionPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /index 1 to reference an existing edge/);
  } finally {
    await runtime.cleanup();
  }
});

test("connection responses deny duplicated node ids before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskConnectionResponse(signals, {
      edgeIndexForItem: () => 0,
    });
    const tasks = createTaskConnectionApi(
      signals,
      response,
      "/connection-duplicated-node",
      {},
      () => ({
        edges: [
          createTaskEdge({ id: "task:1", title: "First" }),
          createTaskEdge({ id: "task:1", title: "Duplicate" }),
        ],
        pageInfo: { hasNextPage: false },
      }),
    );
    const line = tasks.line({});

    assertConnectionPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /duplicated connection node id "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

test("connection responses deny mismatched indexed edges before effects", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskConnectionResponse(signals, {
      edgeIndexForItem: () => 0,
    });
    const tasks = createTaskConnectionApi(
      signals,
      response,
      "/connection-mismatched-index",
      {},
      () => ({
        edges: [
          createTaskEdge({ id: "task:2", title: "Wrong" }),
          createTaskEdge({ id: "task:1", title: "First" }),
        ],
        pageInfo: { hasNextPage: false },
      }),
    );
    const line = tasks.line({});

    assertConnectionPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /edge node "task:2" to match requested itemId "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

test("connection replaceNode must preserve requested node identity", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const response = createTaskConnectionResponse(signals, {
      replaceNode(value, itemId, nextNode) {
        return {
          ...value,
          edges: replaceConnectionNode(
            value.edges,
            itemId,
            { ...nextNode, id: "task:2" },
          ),
        };
      },
    });
    const tasks = createTaskConnectionApi(signals, response, "/connection-corrupt");
    const line = tasks.line({});

    assertConnectionPatchDeniedWithoutSideEffects(line, () => line.patch(tasks.patch.item({
      itemId: "task:1",
      nextItem: { id: "task:1", title: "Replaced" },
    })), /preserve connection node "task:1"/);
  } finally {
    await runtime.cleanup();
  }
});

function createTaskConnectionResponse(signals, overrides = {}) {
  return signals.resource.response.connection()({
    itemId: (task) => task.id,
    edges: overrides.edges ?? ((value) => value.edges),
    node: overrides.node ?? ((edge) => edge.node),
    edgeIndexForItem: overrides.edgeIndexForItem ?? readTaskConnectionEdgeIndex,
    replaceNodes: overrides.replaceNodes ?? (
      (value, nextNodes) => ({
        ...value,
        edges: nextNodes.map(createTaskEdge),
      })
    ),
    replaceNode: overrides.replaceNode ?? (
      (value, itemId, nextNode) => ({
        ...value,
        edges: replaceConnectionNode(value.edges, itemId, nextNode),
      })
    ),
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
}

function createTaskConnectionApi(
  signals,
  response,
  url,
  apiOptions = {},
  load = () => ({
    edges: [createTaskEdge({ id: "task:1", title: "First" })],
    pageInfo: { hasNextPage: false },
  }),
) {
  return signals.api({
    effects: signals.resource.effects.pessimistic(),
    ...apiOptions,
  }).url(url)
    .response(response)
    .list({ load });
}

function readTaskConnectionEdgeIndex(value, itemId) {
  const edgeIndex = value.edges.findIndex((edge) => edge.node.id === itemId);
  return edgeIndex === -1 ? null : edgeIndex;
}

function createTaskEdge(node) {
  return { cursor: `cursor:${node.id}`, node };
}

function replaceConnectionNode(edges, itemId, nextNode) {
  return edges.map((edge) =>
    edge.node.id === itemId
      ? { ...edge, node: nextNode }
      : edge
  );
}

function readTask(value, itemId) {
  return value.edges.find((edge) => edge.node.id === itemId)?.node;
}

function assertConnectionPatchDeniedWithoutSideEffects(
  line,
  patchAction,
  errorPattern,
) {
  const beforeValue = line.value();
  const beforeEffect = line.diagnostics().lastEffect;

  assert.throws(patchAction, errorPattern);
  assert.deepEqual(line.value(), beforeValue);
  assert.equal(line.diagnostics().lastEffect, beforeEffect);
}
