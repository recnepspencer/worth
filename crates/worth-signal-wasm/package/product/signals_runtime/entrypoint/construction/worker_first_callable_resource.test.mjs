import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { loadSignalsModule } from "../../module_loading/load_signals_module.mjs";

async function settleWorkerResourceLine() {
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
}

test("default worker-first root materializes native and compatibility resource families, including scoped collection families", async () => {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const {
    createSignals,
    cleanup,
    resourceParams,
    resourceParamIdentity,
  } = await loadSignalsModule({ rawSurface: "real" });

  try {
    const workerSignals = await createSignals();
    const compatibilitySignals = await createSignals({ deployment: "mainThreadCompatibility" });

    assert.deepEqual(
      workerSignals.resource.effects.branchNative(),
      compatibilitySignals.resource.effects.branchNative(),
    );
    assert.deepEqual(
      workerSignals.resource.mutationResponses,
      compatibilitySignals.resource.mutationResponses,
    );

    const nativeDetail = workerSignals.resource.detail({
      params: resourceParams(),
      normalizeParams: ({ taskId }) => resourceParamIdentity({ taskId }, taskId),
      load: ({ taskId }) => ({ id: taskId, title: `Task:${taskId}` }),
    });
    const nativeDetailLine = nativeDetail.line({ taskId: "task-1" });
    await settleWorkerResourceLine();
    assert.deepEqual(nativeDetailLine.value(), {
      id: "task-1",
      title: "Task:task-1",
    });

    const scopedCollection = workerSignals.scope("wizard").resource.collection({
      params: resourceParams(),
      normalizeParams: ({ workspaceId }) =>
        resourceParamIdentity({ workspaceId }, workspaceId),
      itemIdentity: (item) => item.id,
      load: ({ workspaceId }) => [{
        id: `${workspaceId}:1`,
        title: `Workspace:${workspaceId}`,
      }],
    });
    const scopedCollectionLine = scopedCollection.line({ workspaceId: "demo" });
    await settleWorkerResourceLine();
    assert.deepEqual(scopedCollectionLine.value(), [{
      id: "demo:1",
      title: "Workspace:demo",
    }]);

    const nativePaged = workerSignals.resource.paged({
      params: resourceParams(),
      normalizeParams: ({ feedId }) => resourceParamIdentity({ feedId }, feedId),
      itemIdentity: (item) => item.id,
      accumulatePage: (existing, next) => [...existing, ...next],
      load: ({ feedId }) => [{ id: `${feedId}:1`, title: `Feed:${feedId}` }],
    });
    const nativePagedLine = nativePaged.line({ feedId: "feed" });
    await settleWorkerResourceLine();
    assert.deepEqual(nativePagedLine.value(), [{
      id: "feed:1",
      title: "Feed:feed",
    }]);

    const compatibilityDetail = workerSignals.resource.compatibility.detail({
      version: "worth-resource-external-v1",
      family: "detail",
      definitionId: "external-task-detail",
      requestContract: "native-v1",
      reconciliationContract: "none",
      declaration: {
        params: resourceParams(),
        normalizeParams: ({ taskId }) => resourceParamIdentity({ taskId }, taskId),
        load: ({ taskId }) => ({ id: taskId, title: `External:${taskId}` }),
      },
    });
    const compatibilityDetailLine = compatibilityDetail.line({ taskId: "task-2" });
    await settleWorkerResourceLine();
    assert.deepEqual(compatibilityDetailLine.value(), {
      id: "task-2",
      title: "External:task-2",
    });
    assert.deepEqual(compatibilityDetailLine.descriptor().compatibility, {
      kind: "externalDefinition",
      version: "worth-resource-external-v1",
      definitionId: "external-task-detail",
      requestContract: "native-v1",
      reconciliationContract: "none",
    });

    const compatibilityPaged = workerSignals.scope("wizard").resource.compatibility.paged({
      version: "worth-resource-external-v1",
      family: "paged",
      definitionId: "external-feed",
      requestContract: "native-v1",
      reconciliationContract: "none",
      declaration: {
        params: resourceParams(),
        normalizeParams: ({ feedId }) => resourceParamIdentity({ feedId }, feedId),
        itemIdentity: (item) => item.id,
        accumulatePage: (existing, next) => [...existing, ...next],
        load: ({ feedId }) => [{ id: `${feedId}:compat`, title: `Compat:${feedId}` }],
      },
    });
    const compatibilityPagedLine = compatibilityPaged.line({ feedId: "paged" });
    await settleWorkerResourceLine();
    assert.deepEqual(compatibilityPagedLine.value(), [{
      id: "paged:compat",
      title: "Compat:paged",
    }]);

    const deliveryPacket = workerSignals.resource.compatibility.delivery.basisRefresh({
      packetId: "packet-1",
      basisId: "basis-1",
      nextBasisId: "basis-2",
    });
    assert.equal(deliveryPacket.kind, "basisRefresh");

    const workerBranch = await workerSignals.history().create_branch("feature/resource-branch");
    const compatibilityBranch = compatibilitySignals.history().create_branch("feature/resource-branch");
    await workerSignals.history().switch_branch(workerBranch.id);
    compatibilitySignals.history().switch_branch(compatibilityBranch.id);
    const workerTasks = createBranchTasks(workerSignals);
    const compatibilityTasks = createBranchTasks(compatibilitySignals);
    const workerLine = workerTasks.line({});
    const compatibilityLine = compatibilityTasks.line({});
    await settleWorkerResourceLine();
    await workerLine.patch(workerTasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Worker-First Resource Edit",
    }));
    await compatibilityLine.patch(compatibilityTasks.patch.itemAspect({
      itemId: "task:1",
      aspect: "title",
      value: "Worker-First Resource Edit",
    }));
    const workerEffect = workerLine.diagnostics().lastEffect;
    const compatibilityEffect = compatibilityLine.diagnostics().lastEffect;
    await workerLine.effects().reject(workerEffect.effectId, {
      responseId: "worker-resource-plan-closeout",
    });
    await compatibilityLine.effects().reject(compatibilityEffect.effectId, {
      responseId: "compatibility-resource-plan-closeout",
    });

    workerLine.free();
    compatibilityLine.free();
    nativeDetailLine.free();
    scopedCollectionLine.free();
    nativePagedLine.free();
    compatibilityDetailLine.free();
    compatibilityPagedLine.free();
    await workerSignals.terminate();
    compatibilitySignals.free();
  } finally {
    await cleanup();
    globalThis.Worker = previousWorker;
  }
});

function createBranchTasks(signals) {
  const response = signals.resource.response.array({
    itemId: (task) => task.id,
    aspects: signals.resource.response.objectAspects()({
      title: "title",
    }),
  });
  return signals.api({ effects: signals.resource.effects.branchNative() })
    .url("/branch-execution-tasks")
    .response(response)
    .list({
      load: () => [{ id: "task:1", title: "Loaded" }],
    });
}

function comparableResourceEffectArtifact(artifact) {
  return {
    kind: artifact.kind,
    reason: artifact.reason ?? null,
    conflictCount: artifact.conflictCount ?? null,
    conflicts: artifact.conflicts ?? null,
  };
}
