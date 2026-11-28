import assert from "node:assert/strict";
import test from "node:test";

import { createBranchHead } from "../runtime_fixture/real_resource_signals.mjs";
import { createRealRequestRuntime } from "../runtime_fixture/real_request_runtime.mjs";
import { normalizeForProof } from "./resource_verification_package_helpers.mjs";

test("mutation response exact reconciliation keeps rollback posture honest", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    createBranchHead(signals, "mutation-response-branch-effect-closeout");
    const taskDetail = signals.api({
      effects: signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId").detail({
      load: ({ taskId }) => ({ id: taskId, status: "draft" }),
    });
    const residentLine = taskDetail.line({ taskId: "task:1" });
    const saveTask = signals.api({
      effects: signals.resource.effects.branchNative(),
    }).url("/tasks/:taskId")
      .response(signals.resource.response.detail()())
      .update({
        reconciles: [{
          family: taskDetail,
          params: ({ taskId }) => ({ taskId }),
          fallback: "refetchRequired",
          detail: { kind: "replace" },
        }],
        load: ({ taskId }) => ({ id: taskId, status: "published" }),
      });

    const saveLine = saveTask.line({
      taskId: "task:1",
      body: {},
    });
    const plan = saveLine.mutationResponse();
    const effect = residentLine.diagnostics().lastEffect;
    const rollbackResult = await residentLine.history().rollbackLastEffect();
    const residentVerification = residentLine.history().verificationPackage();
    const writeVerification = saveLine.history().verificationPackage();

    assert.equal(plan.executionArtifacts[0].effectId, effect.effectId);
    assert.match(plan.lifecycleProof.replayExactDigest, /available:SameRuntimeSignalExact/);
    assert.match(plan.lifecycleProof.restoreExactDigest, /available:SameRuntimeBranchExact/);
    assert.match(plan.lifecycleProof.mergeRebaseDigest, /unavailable:line/);
    assert.deepEqual(rollbackResult, {
      kind: "unavailable",
      reason: "noOpenEffect",
      detail:
        "resource effect rollback is unavailable because the line has no open resource effect",
      effectId: null,
      basisCurrentId: null,
      basisAdvanceCount: 0,
    });
    assert.deepEqual(residentLine.value(), { id: "task:1", status: "published" });
    assert.equal(residentLine.history().lifecycle.at(-1)?.event, "delivered");
    assert.deepEqual(normalizeForProof(residentVerification.typedDenials.restoreExact), null);
    assert.equal(
      saveLine.summary().diagnostics.latest.mutationResponseReplayExactDigest,
      plan.lifecycleProof.replayExactDigest,
    );
    assert.equal(
      saveLine.summary().diagnostics.latest.mutationResponseRestoreExactDigest,
      plan.lifecycleProof.restoreExactDigest,
    );
    assert.equal(
      writeVerification.mutationResponse.plan.executionArtifacts[0].effectId,
      effect.effectId,
    );
  } finally {
    await runtime.cleanup();
  }
});
