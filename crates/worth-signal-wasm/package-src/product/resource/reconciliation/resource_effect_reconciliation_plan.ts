const RESOURCE_EFFECT_RECONCILIATION_PLAN = Symbol(
  "WORTHSignal.resourceEffectReconciliationPlan",
);

const NARROW_PATCH_KINDS = new Set([
  "field",
  "region",
  "jsonPath",
  "item",
  "itemAspect",
  "insert",
  "delete",
  "summary",
]);

function planResourceEffectReconciliation(request) {
  requireReconciliationRequest(request);
  const patch = request.serverPatch ?? request.effect.patchIntent;
  const isNarrow = NARROW_PATCH_KINDS.has(patch.kind);
  if (!isNarrow && patch.kind !== "replace") {
    throw reconciliationDenial(
      "unsupportedLocus",
      request.effect.effectId,
      `resource reconciliation has no materializer for patch kind ${patch.kind}`,
    );
  }
  const sameLocusOpenEffects = request.sameLocusOpenEffects.filter(
    (effect) => effect.effectId !== request.effect.effectId,
  );
  const conflict = resolveConflict(
    request.effect,
    sameLocusOpenEffects,
    request.serverRevision,
  );
  return Object.freeze({
    [RESOURCE_EFFECT_RECONCILIATION_PLAN]:
      "resourceEffectReconciliationPlan",
    effectId: request.effect.effectId,
    locusKey: request.effect.locusKey,
    baseValue: request.effect.baseValue,
    canonicalValue: request.canonicalValue,
    patch,
    materialization: Object.freeze({
      kind: isNarrow
        ? "resourceLocusMaterialization"
        : "broadResponseMaterialization",
      wholeNodeReplacementForbidden: isNarrow,
      changedLocusCount: 1,
      fallbackBreadth: 0,
    }),
    conflict,
    counters: Object.freeze({
      effectLookupCount: 1,
      locusLookupCount: 1,
      conflictCandidateCount: sameLocusOpenEffects.length,
      reconstructionCount: 1,
      changedLocusCount: 1,
      fallbackBreadth: 0,
    }),
  });
}

function executePlannedResourceEffectReconciliation(plan, applyPatch) {
  if (
    !plan
    || plan[RESOURCE_EFFECT_RECONCILIATION_PLAN]
      !== "resourceEffectReconciliationPlan"
  ) {
    throw new TypeError(
      "resource reconciliation execution requires a sealed plan",
    );
  }
  if (plan.conflict.kind === "superseded") {
    return Object.freeze({
      kind: "superseded",
      effectId: plan.effectId,
      canonicalValue: plan.canonicalValue,
      materialization: plan.materialization,
      conflict: plan.conflict,
      counters: plan.counters,
    });
  }
  const outcome = applyPatch(plan.patch, plan.canonicalValue);
  return Object.freeze({
    kind: "materialized",
    effectId: plan.effectId,
    canonicalValue: outcome.nextValue,
    materialization: plan.materialization,
    conflict: plan.conflict,
    counters: Object.freeze({
      ...plan.counters,
      downstreamInvalidationCount: outcome.valueChanged ? 1 : 0,
    }),
  });
}

function requireReconciliationRequest(request) {
  if (!request?.effect || typeof request.effect.effectId !== "string") {
    throw new TypeError("resource reconciliation requires an effect envelope");
  }
  if (typeof request.applyPatch !== "function") {
    throw new TypeError("resource reconciliation requires a locus materializer");
  }
}

function resolveConflict(effect, sameLocusEffects, serverRevision) {
  if (sameLocusEffects.length === 0) {
    return Object.freeze({ kind: "none", competingEffectIds: Object.freeze([]) });
  }
  const candidate = Object.freeze({
    ...effect,
    serverRevision: serverRevision ?? effect.serverRevision ?? null,
  });
  const competing = [...sameLocusEffects].sort(compareConflictAuthority);
  const winner = [...competing, candidate].sort(compareConflictAuthority).at(-1);
  return Object.freeze({
    kind: winner.effectId === effect.effectId ? "won" : "superseded",
    winnerEffectId: winner.effectId,
    competingEffectIds: Object.freeze(competing.map((entry) => entry.effectId)),
    serverRevision: serverRevision ?? null,
    policy: serverRevision === null
      ? "stableAdmissionSequence"
      : "serverRevisionThenAdmissionSequence",
  });
}

function compareConflictAuthority(left, right) {
  const leftRevision = left.serverRevision ?? -1;
  const rightRevision = right.serverRevision ?? -1;
  return leftRevision - rightRevision
    || left.admissionSequence - right.admissionSequence;
}

function reconciliationDenial(reason, effectId, detail) {
  const error = new TypeError(detail);
  error.name = "ResourceEffectReconciliationDenial";
  error.code = reason;
  error.effectId = effectId;
  return error;
}

export {
  executePlannedResourceEffectReconciliation,
  planResourceEffectReconciliation,
};
