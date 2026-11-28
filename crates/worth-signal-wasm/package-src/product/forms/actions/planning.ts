import { FormDeclarationError } from "../form_errors.js";
import { hostRequirementBlockers } from "../host/artifacts.js";
import { controllerLocalNavigationBlockers } from "../navigation/semantics.js";
import { routeAuthorityReadinessBlockers } from "../route_authority/artifacts.js";
import { stableValueDigest } from "../values/value_paths.js";
import { resolveActionPatchArtifact } from "./action_patch_scope.js";
import { resolveResourceEffectProfileBinding } from "./resource_effect_profile_binding.js";
import {
  resolveResourceActionBinding,
} from "./resource_action_binding.js";
import {
  actionCatalogEntry,
  actionCounters,
  actionReportDigests,
  actionSummary,
  createActionProofBinding,
} from "./planning_artifacts.js";
import { recoveryActionsForBlockers } from "./recovery.js";

export function planActions(actionDeclarations, form, fieldDeclarations, sourceDeclaration) {
  const patchPlan = form.patchPlan();
  const validation = form.validation();
  const availability = form.availability();
  const admission = form.admission();
  const readiness = form.readiness();
  const host = form.host();
  const resourceSource = form.resourceSource();
  const steps = form.steps();
  const routeAuthority = form.routeAuthority();
  const navigation = form.navigation();
  const binding = createActionProofBinding(form, fieldDeclarations);
  const basePlans = actionDeclarations.map((declaration) =>
    actionPlan(declaration, {
      patchPlan,
      validation,
      availability,
      admission,
      readiness,
      host,
      resourceSource,
      sourceDeclaration,
      fieldDeclarations,
      steps,
      routeAuthority,
      navigation,
      binding,
    }),
  );
  const plans = Object.freeze(basePlans.map((plan) =>
    withRecoveryActions(plan, basePlans, {
      patchPlan,
      resourceSource,
    }),
  ));
  return Object.freeze({
    catalog: Object.freeze(plans.map(actionCatalogEntry)),
    plans: Object.freeze(plans),
    host,
    summary: actionSummary(plans),
    counters: actionCounters(actionDeclarations, plans),
    digests: actionReportDigests(plans),
  });
}

export function findActionPlan(actionDeclarations, form, fieldDeclarations, actionId, sourceDeclaration) {
  const plan = planActions(actionDeclarations, form, fieldDeclarations, sourceDeclaration)
    .plans
    .find((entry) => entry.id === actionId);
  if (!plan) {
    throw new FormDeclarationError("form action is not declared", { actionId });
  }
  return plan;
}

function actionPlan(declaration, context) {
  const resolvedPatch = resolveActionPatchArtifact(
    declaration,
    context.fieldDeclarations,
    context.patchPlan,
  );
  const resourceEffectProfile = resolveResourceEffectProfileBinding(
    declaration,
    context.resourceSource,
    declaration.id,
  );
  const resourceAction = resolveResourceActionBinding(
    declaration,
    context.sourceDeclaration,
    declaration.id,
    context.fieldDeclarations,
    resolvedPatch.patch,
  );
  const resourceRecoveryBlockers = resourceRecoveryActionBlockers(
    declaration,
    context.resourceSource,
  );
  const blockers = actionReadinessBlockers(
    declaration,
    context,
    resolvedPatch.patch,
    [...resourceAction.blockers, ...resourceRecoveryBlockers],
    resourceEffectProfile.blockers,
  );
  const status = blockers.length === 0 ? "accepted" : "denied";
  const effectiveEffectPolicy = routeAuthorityEffectPolicy(declaration, context.routeAuthority);
  const actionSchemaDigest = stableValueDigest(declaration.schema);
  const effectDigest = stableValueDigest({
    effectPolicy: effectiveEffectPolicy,
    hostEffect: declaration.hostEffect,
    resourceAction,
    resourceEffectProfile,
    step: declaration.step,
  });
  const proof = Object.freeze({
    sourceDigest: context.binding.sourceDigest,
    draftDigest: context.binding.draftDigest,
    effectiveDigest: context.binding.effectiveDigest,
    patchDigest: declaration.patchPolicy === "ignore"
      ? context.patchPlan.equivalenceDigest
      : resolvedPatch.patch.equivalenceDigest,
    schemaDigest: context.binding.schemaDigest,
    actionSchemaDigest,
    effectDigest,
    bindingDigest: context.binding.bindingDigest,
  });
  const planSeed = {
    id: declaration.id,
    kind: declaration.kind,
    patchPolicy: declaration.patchPolicy,
    admissionCapability: declaration.admissionCapability,
    destructive: declaration.destructive,
    idempotency: declaration.idempotency,
    effectPolicy: effectiveEffectPolicy,
    hostEffect: declaration.hostEffect,
    hostRequirements: declaration.hostRequirements,
    resourceAction,
    resourceEffectProfile,
    actionSchemaDigest,
    effectDigest,
    step: declaration.step,
    blockers,
    proof,
  };
  const planDigest = stableValueDigest(planSeed);
  return Object.freeze({
    ...actionCatalogEntry(declaration),
    effectPolicy: effectiveEffectPolicy,
    resourceAction,
    resourceEffectProfile,
    status,
    resultKind: status,
    readiness: Object.freeze({
      action: declaration.id,
      canRun: blockers.length === 0,
      blockers: Object.freeze(blockers),
    }),
    recoveryActions: Object.freeze([]),
    patch: resolvedPatch.patch,
    validation: Object.freeze({
      summary: context.validation.summary,
      artifactCount: context.validation.artifacts.length,
    }),
    availability: Object.freeze({
      summary: context.availability.summary,
      artifactCount: context.availability.artifacts.length,
    }),
    admission: Object.freeze({
      summary: context.admission.summary,
      artifactCount: context.admission.artifacts.length,
    }),
    host: Object.freeze({
      requirements: declaration.hostRequirements,
      blockers: hostRequirementBlockers(context.host, declaration.hostRequirements, declaration.id),
      digest: context.host.digest,
    }),
    proof,
    planDigest,
    regulatedActionBindings: regulatedActionBindings(declaration, context.admission, planDigest),
    diagnostics: Object.freeze({
      deniedBeforeEffects: blockers.length > 0,
      consumesLoweredPlan: true,
      routeSemantics: routeSemantics(declaration),
      repeatedAttemptPolicy: declaration.idempotency,
    }),
  });
}

function withRecoveryActions(plan, allPlans, context) {
  return Object.freeze({
    ...plan,
    recoveryActions: recoveryActionsForBlockers(plan.readiness.blockers, {
      canAcceptCanonicalValue: context.patchPlan.semanticDirty === true,
      resourceSource: context.resourceSource,
      availableActions: allPlans,
    }),
  });
}

function actionReadinessBlockers(declaration, context, patch, resourceActionBlockers, resourceEffectBlockers) {
  const scopedFieldSet = scopedActionFieldSet(declaration);
  const blockers = [
    ...hostRequirementBlockers(context.host, declaration.hostRequirements, declaration.id),
    ...resourceActionBlockers,
    ...resourceEffectBlockers,
    ...routeAuthorityReadinessBlockers(context.routeAuthority),
  ];
  if (declaration.step?.routeCoupled === true) {
    if (!routeAuthorityAllowsRouteCoupledBehavior(context.routeAuthority)) {
      blockers.push(Object.freeze({
        kind: "action:deferred",
        action: declaration.id,
        reason: routeAuthorityUnavailableReason(
          context.routeAuthority,
          "route-coupled step action",
        ),
      }));
    }
  }
  if (declaration.step?.routeCoupled !== true && declaration.kind === "step") {
    blockers.push(
      ...controllerLocalNavigationBlockers(
        declaration.step,
        {
          currentStepId: context.navigation.summary.currentStepId,
          localStepIds: context.navigation.summary.localStepIds,
          visitedStepIds: context.navigation.summary.visitedStepIds,
          skippedStepIds: context.navigation.summary.skippedStepIds,
          localSteps: context.steps.artifacts.filter((step) => step.routeCoupled !== true),
        },
        declaration.id,
      ),
    );
  }
  blockers.push(
    ...context.readiness.blockers.filter((blocker) => {
      if (blocker.kind === "host:offline" || blocker.kind === "host:unavailable") {
        return false;
      }
      if (resourceLifecycleActionRecoversBlocker(declaration.resourceAction, blocker)) {
        return false;
      }
      if (blocker.kind === "unchanged" && declaration.patchPolicy !== "requiresNonEmpty") {
        return false;
      }
      if (!blockerAppliesToScopedFields(blocker, scopedFieldSet)) {
        return false;
      }
      return blocker.action === declaration.id || blocker.action === undefined;
    }),
  );
  if (
    declaration.patchPolicy === "requiresNonEmpty"
    && patch.empty
    && !context.readiness.blockers.some((blocker) => blocker.kind === "unchanged")
  ) {
    blockers.push(Object.freeze({
      kind: "unchanged",
      action: declaration.id,
      reason: `action "${declaration.id}" has no semantic changes within its declared patch scope`,
    }));
  }
  return dedupeReadinessBlockers(blockers);
}

function resourceLifecycleActionRecoversBlocker(resourceAction, blocker) {
  if (resourceAction?.kind === "refresh" || resourceAction?.kind === "revalidate") {
    return blocker.kind === "resource:stale"
      || blocker.kind === "resource:deliveryBasisDrift"
      || blocker.kind === "resource:rejected"
      || blocker.kind === "resource:timedOut";
  }
  if (resourceAction?.kind === "replayExact" || resourceAction?.kind === "restoreExact") {
    return blocker.kind === "resource:stale";
  }
  if (resourceAction?.kind === "rollbackLastEffect") {
    return blocker.kind === "resource:mergeConflict";
  }
  return false;
}

function resourceRecoveryActionBlockers(declaration, resourceSource) {
  if (declaration.resourceAction?.kind !== "rollbackLastEffect") {
    return Object.freeze([]);
  }
  if (resourceSource?.effects?.targetedRejectionAvailable === true) {
    return Object.freeze([]);
  }
  return Object.freeze([Object.freeze({
    kind: "resource:actionUnavailable",
    action: declaration.id,
    reason: "declared rollbackLastEffect action requires an open resource effect that can be rejected by identity",
  })]);
}

function dedupeReadinessBlockers(blockers) {
  const seen = new Set();
  const deduped = [];
  for (const blocker of blockers) {
    const digest = stableValueDigest(blocker);
    if (seen.has(digest)) {
      continue;
    }
    seen.add(digest);
    deduped.push(blocker);
  }
  return Object.freeze(deduped);
}

function scopedActionFieldSet(declaration) {
  if (
    declaration.resourceAction?.kind === "patchPlan"
    && Array.isArray(declaration.resourceAction.fields)
    && declaration.resourceAction.fields.length > 0
  ) {
    return new Set(declaration.resourceAction.fields);
  }
  return null;
}

function blockerAppliesToScopedFields(blocker, scopedFieldSet) {
  if (scopedFieldSet === null) {
    return true;
  }
  if (typeof blocker.field === "string") {
    return scopedFieldSet.has(blocker.field);
  }
  if (Array.isArray(blocker.fields)) {
    return blocker.fields.some((fieldId) => scopedFieldSet.has(fieldId));
  }
  return true;
}

function regulatedActionBindings(declaration, admission, planDigest) {
  return Object.freeze(
    admission.artifacts
      .filter((artifact) => (
        artifact.scope === "action" &&
        artifact.ownerId === declaration.id &&
        artifact.binding !== undefined
      ))
      .map((artifact) => {
        const binding = {
          admissionId: artifact.id,
          capability: artifact.capability,
          posture: artifact.posture,
          actorDigest: artifact.actorDigest,
          policyDigest: artifact.policyDigest,
          admissionBindingDigest: artifact.binding.bindingDigest,
          actionPlanDigest: planDigest,
        };
        return Object.freeze({
          ...binding,
          attestationDigest: stableValueDigest(binding),
        });
      }),
  );
}

function routeSemantics(declaration) {
  if (declaration.kind !== "step") {
    return "notStepNavigation";
  }
  return declaration.step?.routeCoupled === true
    ? "routeAuthorityRequired"
    : "controllerLocalOnly";
}

function routeAuthorityEffectPolicy(declaration, routeAuthority) {
  if (declaration.step?.routeCoupled === true && routeAuthorityAllowsRouteCoupledBehavior(routeAuthority)) {
    return "deferred";
  }
  return declaration.effectPolicy;
}

import {
  routeAuthorityAllowsRouteCoupledBehavior,
  routeAuthorityUnavailableReason,
} from "../route_authority/handoff.js";
