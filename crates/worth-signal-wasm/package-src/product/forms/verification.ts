import { readRouteAuthorityContinuityAudit } from "./route_authority/continuity_audit.js";
import { digestFormDiagnosticsProof } from "./diagnostics/digests.js";
import { stableValueDigest } from "./values/value_paths.js";
import { digestFormDiagnosticsHistory } from "./diagnostics/history.js";
import { digestFormStateHistory } from "./state_history.js";

export function buildFormVerificationPackage(form, diagnosticsSnapshot) {
  const state = diagnosticsSnapshot.state;
  const source = form.source();
  const sourceAuthority = state.sourceAuthority;
  const sourceAdmission = form.sourceAdmission();
  const draftRestore = form.draftRestore();
  const resourceSource = state.resourceSource;
  const resourceDrift = state.resourceDrift;
  const attachmentTransfers = state.attachmentTransfers;
  const formDeclaration = state.declaration;
  const draft = form.draft();
  const effective = form.effective();
  const dirty = state.dirty;
  const patchPlan = state.patchPlan;
  const readiness = state.readiness;
  const validation = state.validation;
  const availability = state.availability;
  const admission = state.admission;
  const host = state.host;
  const inputCapabilities = state.inputCapabilities;
  const exit = state.exit;
  const handoff = state.handoff;
  const routeAuthority = state.routeAuthority;
  const attachments = state.attachments;
  const media = state.media;
  const messages = state.messages;
  const collaboration = state.collaboration;
  const interaction = state.interaction;
  const interactionHistory = interaction.history;
  const navigation = state.navigation;
  const navigationHistory = navigation.history;
  const accessibility = state.accessibility;
  const layout = state.layout;
  const layoutMeasurement = state.layoutMeasurement;
  const presentation = state.presentation;
  const presentationHistory = state.presentationHistory;
  const sourceCompatibility = state.sourceCompatibility;
  const steps = state.steps;
  const actions = state.actions;
  const routeAuthorityContinuity = readRouteAuthorityContinuityAudit(routeAuthority, steps, actions);
  const fieldContract = state.fieldContract;
  const inputAdapters = state.inputAdapters;
  const actionHistory = state.actionHistory;
  const actionExecutionHistory = state.actionExecutionHistory;
  const asyncValidationHistory = state.asyncValidationHistory;
  const canonicalizationHistory = state.canonicalizationHistory;
  const resetHistory = state.resetHistory;
  const stateHistory = state.stateHistory;
  const replayRestoreHistory = state.replayRestoreHistory;
  const sourceCompatibilityHistory = state.sourceCompatibilityHistory;
  const diagnosticsSummary = diagnosticsSnapshot.summary;
  const diagnosticsHistory = diagnosticsSnapshot.history;
  const diagnosticsDigest = digestFormDiagnosticsProof(
    diagnosticsSummary,
    diagnosticsSnapshot.diagnosticsStateDigest,
    diagnosticsHistory,
  );
  const canonicalizationDigests = canonicalizationHistory.map((artifact) => artifact.canonicalizationDigest);
  const canonicalizationResourceRollbackDigests = canonicalizationHistory.map(
    (artifact) => artifact.resourceLine?.rollback?.digest ?? null,
  );
  const canonicalizationMutationResponseDigests = canonicalizationHistory.map(
    (artifact) => artifact.resourceLine?.mutationResponse?.digest ?? null,
  );
  const actionHistoryDigests = historyDigestChain(actionHistory, ["resultDigest"]);
  const actionExecutionHistoryDigests = historyDigestChain(actionExecutionHistory, ["executionDigest"]);
  const asyncValidationHistoryDigests = historyDigestChain(asyncValidationHistory, ["lifecycleDigest"]);
  const resourceDriftHistoryDigests = historyDigestChain(resourceDrift.history, ["resultDigest"]);
  const interactionHistoryDigests = historyDigestChain(interactionHistory, ["interactionDigest", "intentDigest"]);
  const navigationHistoryDigests = historyDigestChain(navigationHistory, ["navigationDigest"]);
  const presentationHistoryDigests = historyDigestChain(
    presentationHistory,
    ["presentationDigest", "settlementDigest"],
  );
  const sourceCompatibilityHistoryDigests = historyDigestChain(
    sourceCompatibilityHistory,
    ["compatibilityDigest"],
  );
  const digests = Object.freeze({
    sourceAuthorityDigest: sourceAuthority.sourceAuthorityDigest,
    sourceAuthorityContractDigest: digestWithLabel("sourceAuthorityContractDigest", {
      kind: sourceAuthority.kind,
      sourceId: sourceAuthority.sourceId,
      explicit: sourceAuthority.explicit,
      contract: sourceAuthority.contract,
      identity: sourceAuthority.identity,
    }),
    sourceAdmissionDigest: digestWithLabel("sourceAdmissionDigest", sourceAdmission),
    draftRestoreDigest: digestWithLabel("draftRestoreDigest", draftRestore),
    resourceSourceDigest: resourceSource?.digest ?? null,
    resourceDriftDigest: resourceDrift.digest,
    resourceShapeDigest: resourceSource?.shape.digest ?? null,
    resourceLifecycleDigest: resourceSource?.lifecycle.digest ?? null,
    resourceSettlementDigest: resourceSource?.settlement.digest ?? null,
    resourceEffectProfileDigest: digestWithLabel(
      "resourceEffectProfileDigest",
      resourceSource?.effectProfile.profile ?? null,
    ),
    resourceExternalCompatibilityDigest: resourceSource?.externalCompatibility.digest ?? null,
    resourceTransferDigest: resourceSource?.transfer.digest ?? null,
    resourceVisibleBranchSelectionDigest: resourceSource?.visibleSelection.digest ?? stableValueDigest(null),
    resourceVerificationPackageDigest: resourceSource?.verification.packageDigest ?? null,
    resourceEffectCloseoutMatrixDigest: resourceSource?.effectProfile.closeoutMatrixDigest ?? null,
    resourceMutationResponseDigest: resourceSource?.mutationResponse?.digest ?? null,
    resourceMutationResponseConfirmationDigest:
      resourceSource?.mutationResponse?.confirmationDigest ?? null,
    resourceMutationResponseContractDigest:
      resourceSource?.mutationResponse?.contract.digest ?? null,
    resourceMutationResponseCompletionDigest:
      resourceSource?.mutationResponse?.completion.digest ?? null,
    resourceMutationResponseTargetOutcomeDigest:
      resourceSource?.mutationResponse?.targetOutcomeDigest ?? null,
    resourceMutationResponseCloseoutMatrixDigest:
      resourceSource?.verification.mutationResponseCloseoutMatrixDigest ?? null,
    sourceValueDigest: digestWithLabel("sourceValueDigest", source),
    formDeclarationDigest: digestWithLabel("formDeclarationDigest", formDeclaration),
    fieldContractDigest: digestWithLabel("fieldContractDigest", fieldContract),
    inputAdapterCapabilityDigest: digestWithLabel("inputAdapterCapabilityDigest", inputAdapters),
    hostFactDigest: host.digest,
    inputCapabilityDigest: inputCapabilities.digest,
    exitDigest: exit.digest,
    handoffDigest: handoff.digest,
    routeAuthorityDigest: routeAuthority.digest,
    routeAuthorityContinuityDigest: routeAuthorityContinuity.digest,
    attachmentDigest: attachments.digest,
    attachmentTransferDigest: attachmentTransfers.digest,
    mediaDigest: media.digest,
    messageDigest: messages.digest,
    collaborationDigest: collaboration.digest,
    collaborationEventDigest: collaboration.eventsDigest,
    interactionDigest: interaction.digest,
    interactionHistoryDigest: digestWithLabel("interactionHistoryDigest", interactionHistoryDigests),
    navigationDigest: navigation.digest,
    navigationHistoryDigest: digestWithLabel("navigationHistoryDigest", navigationHistoryDigests),
    accessibilityDigest: accessibility.digest,
    presentationOrderHintDigest: accessibility.orderDigest,
    layoutDigest: layout.digest,
    layoutMeasurementDigest: layoutMeasurement.digest,
    presentationDigest: presentation.digest,
    presentationSettlementAcknowledgementDigest: presentation.acknowledgements.digest,
    sourceCompatibilityDigest: digestWithLabel("sourceCompatibilityDigest", sourceCompatibility),
    draftDigest: digestWithLabel("draftDigest", draft),
    effectiveValueDigest: digestWithLabel("effectiveValueDigest", effective),
    semanticEqualityDigest: digestWithLabel("semanticEqualityDigest", dirty),
    patchPlanDigest: patchPlan.equivalenceDigest,
    readinessDigest: digestWithLabel("readinessDigest", readiness.blockers),
    validationDigest: digestWithLabel("validationDigest", validation),
    asyncValidationLifecycleDigest: digestWithLabel("asyncValidationLifecycleDigest", asyncValidationHistoryDigests),
    canonicalizationDigest: digestWithLabel("canonicalizationDigest", canonicalizationDigests),
    replayRestoreDigest: replayRestoreHistory.at(-1)?.replayRestoreDigest ?? digestWithLabel("replayRestoreDigest", null),
    resetRollbackDigest: digestWithLabel("resetRollbackDigest", canonicalizationResourceRollbackDigests),
    resetHistoryDigest: digestWithLabel("resetHistoryDigest", resetHistory.map((artifact) => artifact.resetDigest)),
    replayRestoreHistoryDigest: digestWithLabel(
      "replayRestoreHistoryDigest",
      replayRestoreHistory.map((artifact) => artifact.replayRestoreDigest),
    ),
    stateHistoryDigest: digestFormStateHistory(stateHistory),
    resourceDriftHistoryDigest: digestWithLabel("resourceDriftHistoryDigest", resourceDriftHistoryDigests),
    mutationResponseReconciliationDigest: digestWithLabel(
      "mutationResponseReconciliationDigest",
      canonicalizationMutationResponseDigests,
    ),
    sourceCompatibilityHistoryDigest: digestWithLabel(
      "sourceCompatibilityHistoryDigest",
      sourceCompatibilityHistoryDigests,
    ),
    presentationHistoryDigest: digestWithLabel("presentationHistoryDigest", presentationHistoryDigests),
    availabilityDependencyDigest: digestWithLabel("availabilityDependencyDigest", availability.dependencyBreadth),
    stepDeclarationProgressDigest: digestWithLabel("stepDeclarationProgressDigest", steps.artifacts),
    admissionPolicyDigest: digestWithLabel("admissionPolicyDigest", admission.dependencyBreadth),
    regulatedBindingDigest: digestWithLabel(
      "regulatedBindingDigest",
      admission.artifacts
        .filter((artifact) => artifact.binding !== undefined)
        .map((artifact) => artifact.binding.bindingDigest),
    ),
    actionCatalogDigest: actions.digests.catalogDigest,
    actionReadinessAdmissionDigest: actions.digests.readinessAdmissionDigest,
    actionPlanDigestSetDigest: actions.digests.planDigestSetDigest,
    submitPlanDigest: actions.digests.submitPlanDigest,
    actionLifecycleDigest: digestWithLabel("actionLifecycleDigest", actionHistoryDigests),
    actionExecutionLifecycleDigest: digestWithLabel("actionExecutionLifecycleDigest", actionExecutionHistoryDigests),
    diagnosticsHistoryDigest: digestFormDiagnosticsHistory(diagnosticsHistory),
    diagnosticsSummaryDigest: diagnosticsSummary.digest,
    diagnosticsDigest,
  });
  return Object.freeze({
    kind: "formVerification",
    sourceAuthority,
    routeAuthorityContinuity,
    digests,
    actionHistory: Object.freeze({
      attempts: actionHistory.length,
      digest: digests.actionLifecycleDigest,
    }),
    actionExecutionHistory: Object.freeze({
      operations: actionExecutionHistory.length,
      digest: digests.actionExecutionLifecycleDigest,
    }),
    asyncValidationHistory: Object.freeze({
      operations: asyncValidationHistory.length,
      digest: digests.asyncValidationLifecycleDigest,
    }),
    canonicalizationHistory: Object.freeze({
      operations: canonicalizationHistory.length,
      digest: digests.canonicalizationDigest,
    }),
    resetHistory: Object.freeze({
      operations: resetHistory.length,
      digest: digests.resetHistoryDigest,
    }),
    replayRestoreHistory: Object.freeze({
      operations: replayRestoreHistory.length,
      digest: digests.replayRestoreHistoryDigest,
    }),
    stateHistory: Object.freeze({
      operations: stateHistory.length,
      digest: digests.stateHistoryDigest,
    }),
    interactionHistory: Object.freeze({
      operations: interactionHistory.length,
      digest: digests.interactionHistoryDigest,
    }),
    navigationHistory: Object.freeze({
      operations: navigationHistory.length,
      digest: digests.navigationHistoryDigest,
    }),
    presentationHistory: Object.freeze({
      operations: presentationHistory.length,
      digest: digests.presentationHistoryDigest,
    }),
    diagnosticsHistory: Object.freeze({
      operations: diagnosticsHistory.length,
      digest: digests.diagnosticsHistoryDigest,
    }),
    sourceCompatibilityHistory: Object.freeze({
      operations: sourceCompatibilityHistory.length,
      digest: digests.sourceCompatibilityHistoryDigest,
    }),
    performanceEnvelope: formPerformanceEnvelope({
      sourceCompatibility,
      resourceSource,
      resourceDrift,
      attachmentTransfers,
      host,
      inputCapabilities,
      exit,
      handoff,
      routeAuthority,
      attachments,
      media,
      messages,
      collaboration,
      interaction,
      navigation,
      accessibility,
      layout,
      layoutMeasurement,
      presentation,
      validation,
      availability,
      admission,
      steps,
      actions,
      actionHistory,
      actionExecutionHistory,
      asyncValidationHistory,
      canonicalizationHistory,
      resetHistory,
      replayRestoreHistory,
      stateHistory,
      diagnosticsHistory,
      interactionHistory,
      navigationHistory,
      sourceCompatibilityHistory,
    }),
    packageDigest: digestDigestRecord(digests),
  });
}

function formPerformanceEnvelope(reports) {
  return Object.freeze({
    costBasis: "derivedFullReportScan",
    diagnosticsSummaryBreadth: "summaryShapedNotFullHistoryMaterialization",
    diagnosticsHistoryOperations: reports.diagnosticsHistory.length,
    actionHistoryAttempts: reports.actionHistory.length,
    actionExecutionOperations: reports.actionExecutionHistory.length,
    asyncValidationOperations: reports.asyncValidationHistory.length,
    canonicalizationOperations: reports.canonicalizationHistory.length,
    resetOperations: reports.resetHistory.length,
    replayRestoreOperations: reports.replayRestoreHistory.length,
    fieldWriteOperations: reports.stateHistory.filter((entry) => entry.entryKind === "draftWrite").length,
    rawInputOperations: reports.stateHistory.filter((entry) => entry.entryKind === "rawInput").length,
    interactionOperations: reports.interactionHistory.length,
    navigationOperations: reports.navigationHistory.length,
    sourceCompatibilityOperations: reports.sourceCompatibilityHistory.length,
    resourceSource: reports.resourceSource?.counters ?? null,
    resourceDrift: reports.resourceDrift.counters,
    hostFacts: reports.host.counters,
    inputCapabilities: reports.inputCapabilities.counters,
    exit: reports.exit.counters,
    handoff: reports.handoff.counters,
    routeAuthority: reports.routeAuthority.counters,
    attachments: reports.attachments.counters,
    attachmentTransfers: reports.attachmentTransfers.counters,
    media: reports.media.counters,
    messages: reports.messages.counters,
    collaboration: reports.collaboration.counters,
    interaction: reports.interaction.counters,
    navigation: reports.navigation.counters,
    accessibility: reports.accessibility.counters,
    layout: reports.layout.counters,
    layoutMeasurement: reports.layoutMeasurement.counters,
    presentation: reports.presentation.counters,
    sourceCompatibility: reports.sourceCompatibility.counters,
    validation: reports.validation.counters,
    availability: reports.availability.counters,
    admission: reports.admission.counters,
    steps: reports.steps.counters,
    actions: reports.actions.counters,
  });
}

function historyDigestChain(history, digestKeys) {
  return history.map((artifact) => artifactDigestToken(artifact, digestKeys));
}

function artifactDigestToken(artifact, digestKeys) {
  if (artifact === null || artifact === undefined) {
    return null;
  }
  for (const key of digestKeys) {
    const digest = artifact[key];
    if (typeof digest === "string") {
      return digest;
    }
  }
  return stableValueDigest(artifact);
}

function digestWithLabel(label, value) {
  try {
    return stableValueDigest(value);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new RangeError(`${label}: ${detail}`);
  }
}

function digestDigestRecord(record) {
  let hash = 2166136261;
  for (const key of Object.keys(record).sort()) {
    hash = digestHashChunk(hash, key);
    hash = digestHashChunk(hash, "\u0000");
    const value = record[key];
    hash = digestHashChunk(
      hash,
      typeof value === "string" ? value : stableValueDigest(value),
    );
    hash = digestHashChunk(hash, "\u0001");
  }
  return `f1a-${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

function digestHashChunk(hash, value) {
  let nextHash = hash;
  for (let index = 0; index < value.length; index += 1) {
    nextHash ^= value.charCodeAt(index);
    nextHash = Math.imul(nextHash, 16777619);
  }
  return nextHash;
}
