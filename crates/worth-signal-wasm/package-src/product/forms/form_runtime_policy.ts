export function readFormFieldWritePosture({
  form,
  fieldId,
  capability,
  availabilityEditBlocker,
  admissionCapabilityBlocker,
  collaborationFieldWriteBlocker,
  routeAuthorityWriteBlocker,
  sourceCompatibilityBlockers,
}) {
  form.field(fieldId);
  const availabilityBlocker = availabilityEditBlocker(form.availability(), fieldId);
  const admissionBlocker = admissionCapabilityBlocker(form.admission(), fieldId, capability);
  const collaborationBlocker = collaborationFieldWriteBlocker(form.collaboration(), fieldId, capability);
  const routeAuthorityBlocker = routeAuthorityWriteBlocker(form.routeAuthority(), fieldId);
  const blockers = [
    ...sourceCompatibilityBlockers(form.sourceCompatibility()),
    availabilityBlocker,
    admissionBlocker,
    collaborationBlocker,
    routeAuthorityBlocker,
  ].filter(Boolean);
  return Object.freeze({
    field: fieldId,
    capability,
    canWrite: blockers.length === 0,
    blockers: Object.freeze(blockers),
    reason: blockers[0]?.reason ?? "field write admitted",
  });
}

export function readFormReadiness({
  form,
  rawInputs,
  sourceCompatibilityBlockers,
  resourceSourceReadinessBlockers,
  attachmentTransferReadinessBlockers,
  validationReadinessBlockers,
  availabilityReadinessBlockers,
  admissionReadinessBlockers,
  collaborationReadinessBlockers,
  routeAuthorityReadinessBlockers,
  hostRequirementBlockers,
  resolveResourceActionBinding,
  resolveResourceEffectProfileBinding,
  declarationSource,
  fieldDeclarations,
  actionDeclarations,
  dedupeReadinessBlockers,
  rawInputBlockers,
}) {
  const patchPlan = form.patchPlan();
  const blockers = rawInputBlockers(rawInputs);
  blockers.push(...sourceCompatibilityBlockers(form.sourceCompatibility()));
  blockers.push(...resourceSourceReadinessBlockers(form.resourceSource()));
  blockers.push(...attachmentTransferReadinessBlockers(form.attachmentTransfers()));
  blockers.push(...validationReadinessBlockers(form.validation()));
  blockers.push(...availabilityReadinessBlockers(form.availability()));
  blockers.push(...admissionReadinessBlockers(form.admission()));
  blockers.push(...collaborationReadinessBlockers(form.collaboration(), patchPlan));
  blockers.push(...routeAuthorityReadinessBlockers(form.routeAuthority()));
  const submitAction = actionDeclarations.find((entry) => entry.id === "submit");
  if (submitAction) {
    blockers.push(...hostRequirementBlockers(form.host(), submitAction.hostRequirements, "submit"));
    blockers.push(
      ...resolveResourceActionBinding(
        submitAction,
        declarationSource,
        "submit",
        fieldDeclarations,
        patchPlan,
      ).blockers,
    );
    blockers.push(
      ...resolveResourceEffectProfileBinding(
        submitAction,
        form.resourceSource(),
        "submit",
      ).blockers,
    );
  }
  if (patchPlan.empty) {
    blockers.push({
      kind: "unchanged",
      reason: "form has no semantic changes to submit",
    });
  }
  const dedupedBlockers = dedupeReadinessBlockers(blockers);
  return Object.freeze({
    canSubmit: dedupedBlockers.length === 0,
    blockers: dedupedBlockers,
    patchPlan,
  });
}
