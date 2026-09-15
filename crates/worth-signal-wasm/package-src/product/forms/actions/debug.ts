import { stableValueDigest } from "../values/value_paths.js";

// Cost: O(action history) to filter this action's attempts and executions.
// `verification` and `digest` are computed on first access and memoized on
// the report: `form.verification()` digests every retained history family,
// so building it eagerly made every debug read (and every React render that
// takes one) O(total form history) even when only `pending` or
// `latestExecution` was consulted.
export function readActionDebug(form, actionId) {
  const plan = form.actionPlan(actionId);
  const attempts = Object.freeze(
    form.actionHistory().filter((attempt) => attempt.action === actionId),
  );
  const executions = Object.freeze(
    form.actionExecutionHistory().filter((execution) => (
      execution.action === actionId || execution.targetAction === actionId
    )),
  );
  const latestAttempt = attempts.at(-1) ?? null;
  const latestExecution = executions.at(-1) ?? null;
  const blockers = Object.freeze(latestAttempt?.blockers ?? plan.readiness.blockers);
  const latestReason = latestExecution?.reason
    ?? latestAttempt?.reason
    ?? blockers[0]?.reason
    ?? (plan.status === "accepted" ? "action plan accepted" : "action plan is not ready");
  const debug = {
    kind: "actionDebug",
    action: actionId,
    canRun: plan.readiness.canRun,
    pending: latestExecution?.resultKind === "pending",
    latestReason,
    blockers,
    plan,
    latestAttempt,
    latestExecution,
    attempts,
    executions,
  };
  let verification = null;
  let digest = null;
  Object.defineProperty(debug, "verification", {
    enumerable: true,
    get() {
      if (verification === null) {
        const formVerification = form.verification();
        verification = Object.freeze({
          packageDigest: formVerification.packageDigest,
          actionPlanDigest: plan.planDigest,
          actionLifecycleDigest: formVerification.digests.actionLifecycleDigest,
          actionExecutionLifecycleDigest: formVerification.digests.actionExecutionLifecycleDigest,
        });
      }
      return verification;
    },
  });
  Object.defineProperty(debug, "digest", {
    enumerable: true,
    get() {
      if (digest === null) {
        digest = stableValueDigest({
          action: debug.action,
          canRun: debug.canRun,
          pending: debug.pending,
          latestReason: debug.latestReason,
          blockerKinds: debug.blockers.map((blocker) => blocker.kind),
          planDigest: debug.plan.planDigest,
          latestAttemptDigest: debug.latestAttempt?.resultDigest ?? null,
          latestExecutionDigest: debug.latestExecution?.executionDigest ?? null,
          attemptDigests: debug.attempts.map((attempt) => attempt.resultDigest),
          executionDigests: debug.executions.map((execution) => execution.executionDigest),
          verification: debug.verification,
        });
      }
      return digest;
    },
  });
  return Object.freeze(debug);
}
