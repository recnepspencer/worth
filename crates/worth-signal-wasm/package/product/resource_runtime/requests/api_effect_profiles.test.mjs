import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../runtime_fixture/real_request_runtime.mjs";

test("typed resource effect profiles inherit through api scopes and route settings", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const api = signals.api({
      effects: signals.resource.effects.branchNative(),
    }).scope({
      effects: ({ workspaceId }) =>
        workspaceId === "sensitive"
          ? signals.resource.effects.sensitive()
          : signals.resource.effects.serverCanonical(),
    });
    const inherited = api.url("/workspaces/:workspaceId/tasks").detail({
      load: ({ workspaceId }) => ({ id: workspaceId }),
    });
    const routeOwned = api.url("/workspaces/:workspaceId/audit")
      .effects(signals.resource.effects.pessimistic())
      .detail({
        load: ({ workspaceId }) => ({ id: workspaceId }),
      });

    const inheritedLine = inherited.line({ workspaceId: "demo" });
    const sensitiveLine = inherited.line({ workspaceId: "sensitive" });
    const routeLine = routeOwned.line({ workspaceId: "demo" });

    assert.equal(inheritedLine.request().effects.name, "serverCanonical");
    assert.equal(
      inheritedLine.request().sources.effects.source,
      "apiScope[1].effects",
    );
    assert.equal(sensitiveLine.request().effects.name, "sensitive");
    assert.equal(routeLine.request().effects.name, "pessimistic");
    assert.deepEqual(routeLine.request().sources.effects, {
      source: "endpoint.effects",
      overridden: true,
    });
    assert.equal(routeLine.diagnostics().request.effects.name, "pessimistic");
    assert.equal(
      routeLine.diagnosticsSummary().request.effects.name,
      "pessimistic",
    );
    assert.equal(
      routeLine.history().verificationPackage().requestPosture.effectsName,
      "pessimistic",
    );
  } finally {
    await runtime.cleanup();
  }
});

test("resource effect profiles deny forged and impossible posture combinations", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;

    assert.throws(
      () => signals.api({ effects: { name: "fake" } }),
      /requires a profile created with resource\.effects\.\*\(\)/,
    );
    assert.throws(
      () =>
        signals.resource.effects.custom({
          name: "bad",
          optimism: "branchSpeculative",
          confirmation: "serverCanonical",
          rollback: "unavailable",
          rebase: "unavailable",
          preimage: "none",
        }),
      /cannot enable branch speculation with unavailable rollback/,
    );
    assert.throws(
      () =>
        signals.api({}).url("/tasks")
          .effects({ name: "fake" })
          .detail({ load: () => ({ id: "t1" }) }),
      /requires a profile created with resource\.effects\.\*\(\)/,
    );
  } finally {
    await runtime.cleanup();
  }
});
