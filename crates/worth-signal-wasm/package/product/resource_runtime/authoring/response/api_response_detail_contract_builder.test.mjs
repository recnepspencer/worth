import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../runtime_fixture/real_request_runtime.mjs";

function stripSymbolProperties(value) {
  if (Array.isArray(value)) {
    return value.map(stripSymbolProperties);
  }
  if (value !== null && typeof value === "object") {
    const normalized = {};
    for (const key of Object.keys(value)) {
      normalized[key] = stripSymbolProperties(value[key]);
    }
    return normalized;
  }
  return value;
}

test("detail response contracts own the detail finalizer lane", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const response = runtime.signals.resource.response.detail()();
    assert.equal(response.kind, "detail");
    assert.equal(response.lensProof.topology, "detail");
    assert.equal(
      response.lensProof.capabilityRows.some(
        (row) => row.locus === "detailResponse" && row.patchScope === "line",
      ),
      true,
    );

    const route = runtime.signals.api({}).url("/users/:userId").response(response);
    const user = route.detail({
      load: ({ userId }) => ({ id: userId, name: "First" }),
    });
    assert.deepEqual(user.line({ userId: "u1" }).value(), {
      id: "u1",
      name: "First",
    });

    assert.throws(
      () => route.list({ load: () => [] }),
      /detail response lane; use detail/,
    );
    assert.throws(
      () => route.paged({ accumulatePage: (_existing, next) => next, load: () => [] }),
      /detail response lane; use detail/,
    );
    const saveUser = route.update({
      load: ({ userId, body }) => ({ id: userId, name: body.name }),
    });
    const saveLine = saveUser.line({
      userId: "u1",
      body: { name: "Updated" },
    });
    assert.deepEqual(saveLine.value(), {
      id: "u1",
      name: "Updated",
    });
    assert.deepEqual(stripSymbolProperties(saveLine.mutationResponse()), {
      version: "resource-mutation-response-plan-v1",
      source: 'api.url("/users/:userId").response(...).update(...)',
      planId: `${saveLine.descriptor().family.familyId}:${saveLine.descriptor().canonicalParams.canonicalKey}:PUT:0:0:0:0:1`,
      route: "/users/:userId",
      method: "PUT",
      line: {
        familyId: saveLine.descriptor().family.familyId,
        runtimeLineId: saveLine.descriptor().runtimeLineId,
        canonicalKey: saveLine.descriptor().canonicalParams.canonicalKey,
      },
      request: {
        correlationId: null,
        branchId: null,
        basisId: null,
        requestPath: "/users/u1",
        url: "/users/u1",
      },
      submittedTargets: [],
      response: {
        topology: "detail",
        readResponseLensSource: "resource.response.detail<T>()",
        readResponseLensDigest: response.lensProof.compiledLensDigest,
        mutationResponseLensDigest:
          saveLine.mutationResponse().response.mutationResponseLensDigest,
        declaredFieldDigest: "mutation-response-declared-fields|not-applicable",
        unknownFieldPosture: {
          kind: "notApplicable",
          fields: [],
          digest: "mutation-response-unknown-fields|not-applicable",
        },
        payloadDigest: 'mutation-response-payload|{"id":"u1","name":"Updated"}',
      },
      confirmation: {
        kind: "preservedOptimisticTruth",
        detail:
          "mutation response classified as preservedOptimisticTruth with 0 exact target(s) 0 fallback target(s) and 0 diagnostic fact(s)",
        exactTargetCount: 0,
        fallbackTargetCount: 0,
        diagnosticCount: 0,
        fallbackKinds: [],
        digest:
          "mutation-response-confirmation|preservedOptimisticTruth|exact:0|fallbacks:none|diagnostics:mutation-response-diagnostics|none",
      },
      lifecycleProof: {
        entries: [],
        count: 0,
        replayExactDigest: "mutation-response-replay-exact|none",
        restoreExactDigest: "mutation-response-restore-exact|none",
        rollbackDigest: "mutation-response-rollback|none",
        mergeRebaseDigest: "mutation-response-merge-rebase|none",
        digest:
          "mutation-response-lifecycle|mutation-response-rollback|none|mutation-response-merge-rebase|none|mutation-response-replay-exact|none|mutation-response-restore-exact|none",
      },
      diagnostics: {
        entries: [],
        count: 0,
        digest: "mutation-response-diagnostics|none",
      },
      identityMigration: null,
      targets: [],
      targetCount: 0,
      atomicity: "zeroTargets",
      reconciliationAtomicity: "allOrNone",
      partialAdmission: "notNeeded",
      targetDigest: "mutation-response-targets|none",
      fallbackDigest: "mutation-response-fallbacks|none",
      executionArtifacts: [],
      executionDigest: "mutation-response-execution|none",
      counters: {
        planningBreadth: 1,
        responseExtractionBreadth: 1,
        payloadFieldExtractionBreadth: 0,
        targetLookupBreadth: 0,
        targetFanoutBreadth: 0,
        topologyTraversalBreadth: 0,
        reconstructionBreadth: 0,
        fallbackBreadth: 0,
        executionBreadth: 0,
        diagnosticExtractionBreadth: 0,
        targetBasisSnapshotBreadth: 0,
        staleTargetDenialBreadth: 0,
        partialPolicyBreadth: 0,
        identityResponseExtractionBreadth: 0,
        identityMigrationTargetFanoutBreadth: 0,
        identityMigrationStaleDenialBreadth: 0,
        identityMigrationExecutionBreadth: 0,
        identityMigrationLifecycleProofBreadth: 0,
        confirmationClassificationBreadth: 0,
        lifecycleProofBreadth: 0,
      },
    });
    assert.deepEqual(
      saveLine.history().lifecycle.slice(-2).map((entry) => entry.event),
      ["materialized", "mutationResponsePlanned"],
    );
    assert.deepEqual(
      stripSymbolProperties(saveLine.history().lifecycle.at(-1).mutationResponsePlan),
      stripSymbolProperties(saveLine.mutationResponse()),
    );
    assert.equal(
      saveLine.history().verificationPackage().mutationResponse.plan.targetCount,
      0,
    );
    assert.equal(
      saveLine.history().verificationPackage().mutationResponse.planCount,
      1,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("collection response lanes deny detail(...) but admit response-owned write planning", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const response = runtime.signals.resource.response.array({
      itemId: (item) => item.id,
    });
    const route = runtime.signals.api({}).url("/tasks").response(response);

    assert.throws(
      () => route.detail({ load: () => [] }),
      /collection response lane; use list/,
    );

    const saveTasks = route.update({
      load: ({ body }) => body.tasks,
    });
    const saveLine = saveTasks.line({
      body: {
        tasks: [{ id: "t1", title: "First" }],
      },
    });

    assert.deepEqual(saveLine.value(), [{ id: "t1", title: "First" }]);
    assert.equal(saveLine.request().method, "PUT");
    assert.equal(saveLine.mutationResponse().response.topology, "directArray");
    assert.equal(
      saveLine.history().verificationPackage().mutationResponse.plan.targetCount,
      0,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("response-owned write planning records multi-target fallback artifacts before any later reconciliation phase exists", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const response = runtime.signals.resource.response.detail()();
    const route = runtime.signals.api({}).url("/users/:userId").response(response);
    const userRead = runtime.signals.api({}).url("/users/:userId").detail({
      load: ({ userId }) => ({ id: userId, name: "First" }),
    });
    const auditRead = runtime.signals.api({}).url("/user-audit/:userId").detail({
      load: ({ userId }) => ({ id: userId, events: 0 }),
    });

    const saveUser = route.update({
      reconciles: [
        {
          family: userRead,
          params: ({ userId }) => ({ userId }),
          fallback: "refetchRequired",
        },
        {
          family: auditRead,
          params: ({ userId }) => ({ userId }),
          fallback: "deliveryAwaited",
        },
      ],
      load: ({ userId, body }) => ({ id: userId, name: body.name }),
    });

    const residentUserLine = userRead.line({ userId: "u1" });
    const saveLine = saveUser.line({
      userId: "u1",
      body: { name: "Updated" },
    });
    const plan = saveLine.mutationResponse();

    assert.equal(plan.targetCount, 2);
    assert.equal(plan.atomicity, "allOrNone");
    assert.equal(
      plan.targetDigest,
      `mutation-response-targets|mutationTarget1:detail:${residentUserLine.descriptor().family.familyId}:${residentUserLine.descriptor().canonicalParams.canonicalKey}:refetchRequired,mutationTarget2:detail:${auditRead.line({ userId: "u1" }).descriptor().family.familyId}:${auditRead.line({ userId: "u1" }).descriptor().canonicalParams.canonicalKey}:deliveryAwaited`,
    );
    assert.equal(
      plan.fallbackDigest,
      `mutation-response-fallbacks|mutationTarget1:refetchRequired:none:${residentUserLine.descriptor().canonicalParams.canonicalKey},mutationTarget2:deliveryAwaited:none:${auditRead.line({ userId: "u1" }).descriptor().canonicalParams.canonicalKey}`,
    );
    assert.equal(plan.executionArtifacts.length, 2);
    assert.equal(plan.executionArtifacts[0].kind, "fallback");
    assert.equal(plan.executionArtifacts[1].fallback, "deliveryAwaited");
    assert.equal(plan.targets[0].line.residency, "resident");
    assert.equal(plan.targets[1].line.residency, "declared");
    assert.equal(plan.counters.targetLookupBreadth, 2);
    assert.equal(plan.counters.targetFanoutBreadth, 2);
    assert.equal(plan.counters.fallbackBreadth, 2);
    assert.equal(plan.counters.executionBreadth, 2);
  } finally {
    await runtime.cleanup();
  }
});

test("mutation response target digests distinguish resolved target lines, not just declared families", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const response = runtime.signals.resource.response.detail()();
    const route = runtime.signals.api({}).url("/users/:userId").response(response);
    const userRead = runtime.signals.api({}).url("/users/:userId").detail({
      load: ({ userId }) => ({ id: userId, name: "First" }),
    });

    const saveUser = route.update({
      reconciles: [
        {
          family: userRead,
          params: ({ userId }) => ({ userId }),
          fallback: "refetchRequired",
        },
      ],
      load: ({ userId, body }) => ({ id: userId, name: body.name }),
    });

    const planA = saveUser.line({
      userId: "u1",
      body: { name: "Updated A" },
    }).mutationResponse();
    const planB = saveUser.line({
      userId: "u2",
      body: { name: "Updated B" },
    }).mutationResponse();

    assert.notEqual(planA.targetDigest, planB.targetDigest);
    assert.match(
      planA.targetDigest,
      /mutationTarget1:detail:__resourceFamily\.detail\.\d+:\/users\/u1:refetchRequired/,
    );
    assert.match(
      planB.targetDigest,
      /mutationTarget1:detail:__resourceFamily\.detail\.\d+:\/users\/u2:refetchRequired/,
    );
  } finally {
    await runtime.cleanup();
  }
});

test("mutation response planning denies accessor-backed payloads before visible line truth changes", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const response = runtime.signals.resource.response.detail()();
    const route = runtime.signals.api({}).url("/users/:userId").response(response);
    let getterCalls = 0;

    const saveUser = route.update({
      load: ({ userId, body }) => {
        const payload = { id: userId };
        Object.defineProperty(payload, "name", {
          enumerable: true,
          configurable: true,
          get() {
            getterCalls += 1;
            return body.name;
          },
        });
        return payload;
      },
    });

    const line = saveUser.line({ userId: "u1", body: { name: "Unsafe" } });
    // The refusal is a rejected settlement, not a thrown error: the same
    // outcome an asynchronous load gets, and the getter was never invoked.
    assert.equal(line.status().kind, "rejected");
    assert.match(line.status().message, /accessor-backed property "name"/);
    assert.equal(line.value(), null);
    assert.equal(line.mutationResponse(), null);
    assert.equal(getterCalls, 0);
  } finally {
    await runtime.cleanup();
  }
});

test("response-owned write planning denies malformed reconcile fallback declarations", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const response = runtime.signals.resource.response.detail()();
    const userRead = runtime.signals.api({}).url("/users/:userId").detail({
      load: ({ userId }) => ({ id: userId }),
    });

    assert.throws(
      () =>
        runtime.signals.api({}).url("/users/:userId").response(response).update({
          reconciles: [
            {
              family: userRead,
              params: ({ userId }) => ({ userId }),
              fallback: "laterMaybe",
            },
          ],
          load: ({ userId, body }) => ({ id: userId, name: body.name }),
        }),
      /fallback must be one of deletionUnavailable, placementUnavailable, refetchRequired, deliveryAwaited, partialReconciliation, unsupportedTarget/,
    );
  } finally {
    await runtime.cleanup();
  }
});
