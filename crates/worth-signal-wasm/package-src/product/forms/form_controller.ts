import { FormDeclarationError } from "./form_errors.js";
import { materializeActionDeclarations } from "./actions/declarations.js";
import { reportRouteHandoffForExecution } from "./actions/route_handoff.js";
import { createActionRuntimeBindings } from "./actions/runtime_bindings.js";
import { findActionPlan, planActions } from "./actions/planning.js";
import { resolveResourceActionBinding } from "./actions/resource_action_binding.js";
import { resolveResourceEffectProfileBinding } from "./actions/resource_effect_profile_binding.js";
import { createActionAttemptStore } from "./actions/results.js";
import { createActionExecutionStore } from "./actions/execution.js";
import { attachmentTransferReadinessBlockers } from "./attachment_transfers/report.js";
import { createAttachmentPresentationStore } from "./attachments/store.js";
import { createCanonicalizationStore } from "./canonicalization.js";
import { collaborationFieldWriteBlocker, collaborationReadinessBlockers } from "./collaboration/artifacts.js";
import { materializeCollaborationDeclaration } from "./collaboration/declarations.js";
import { createCollaborationStore } from "./collaboration/store.js";
import { admissionCapabilityBlocker, admissionReadinessBlockers } from "./admission/artifacts.js";
import { materializeAdmissionDeclarations } from "./admission/declarations.js";
import { evaluateAdmission } from "./admission/execution.js";
import { createExitPresentationStore } from "./exit/store.js";
import { availabilityReadinessBlockers, availabilityEditBlocker, clearedFieldIds, omittedFieldIds } from "./availability/artifacts.js";
import { materializeAvailabilityDeclarations } from "./availability/declarations.js";
import { evaluateAvailability } from "./availability/execution.js";
import { materializeFieldDeclarations } from "./fields/declarations.js";
import { hostRequirementBlockers, readHostReport } from "./host/artifacts.js";
import { materializeHostBindings } from "./host/declarations.js";
import { createHandoffStore } from "./handoff/store.js";
import {
  routeAuthorityReadinessBlockers,
  routeAuthorityWriteBlocker,
} from "./route_authority/artifacts.js";
import { createRouteAuthorityStore } from "./route_authority/store.js";
import { createInteractionBindings } from "./interaction/controller_bindings.js";
import { createInteractionStore } from "./interaction/store.js";
import { applyControllerLocalNavigation as applyLocalStepNavigation } from "./navigation/controller_local_navigation.js";
import { createNavigationStore } from "./navigation/store.js";
import {
  materializeFormDeclarationRecord,
  readFieldContractDiagnostics,
  readFormDeclarationDiagnostics,
  readInputAdapterDiagnostics,
} from "./controller_declaration_diagnostics.js";
import { createFormDiagnosticsHistoryStore } from "./diagnostics/history.js";
import { createDiagnosticsControllerBindings } from "./diagnostics/controller_bindings.js";
import { createDerivedReportBindings } from "./derived_report_bindings.js";
import { createMeasurementSemanticContextReader } from "./measurement/controller_semantic_context.js";
import { materializeMeasurementDeclaration } from "./measurement/declarations.js";
import { createLayoutMeasurementStore } from "./measurement/store.js";
import { createMediaPresentationStore } from "./media/store.js";
import { createMessagePresentationStore } from "./messages/store.js";
import { createFormReportBindings } from "./controller_report_bindings.js";
import { createResourceDriftStore } from "./resource_drift/store.js";
import { createPresentationBindings } from "./presentation/controller_bindings.js";
import { materializePresentationDeclaration } from "./presentation/declarations.js";
import { createPresentationRuntimeBindings } from "./presentation/runtime_bindings.js";
import { createDeclaredPresentationScopeRegistry } from "./presentation/scope_registry.js";
import { createPresentationStore } from "./presentation/store.js";
import { createResourceMergeProjectionRegistry } from "./resource_merge/projection.js";
import { resourceMergeReadinessBlockers } from "./resource_merge/report.js";
import { createResourceMergeStore } from "./resource_merge/store.js";
import { createRecoveryBindings } from "./recovery/controller_bindings.js";
import { createFormReplayRestoreStore } from "./replay_restore/store.js";
import { dedupeReadinessBlockers } from "./readiness_artifacts.js";
import { createResourceSurfaceBindings } from "./resource_surface_bindings.js";
import { createFormResetStore } from "./reset/store.js";
import { createFormStateHistoryStore } from "./state_history.js";
import { createStateHistoryControllerBindings } from "./state_history/controller_bindings.js";
import { registerFormFields } from "./fields/registry.js";
import { applyFormFieldUpdates } from "./fields/bulk_updates.js";
import { evaluateSteps } from "./steps/artifacts.js";
import { materializeStepDeclarations } from "./steps/declarations.js";
import { materializeFormSourceAuthority, readSourceBootstrapArtifact } from "./sources/form_sources.js";
import { readResourceSourceReport, resourceSourceReadinessBlockers } from "./sources/resource_source_report.js";
import { createSourceCompatibilityStore, sourceCompatibilityBlockers } from "./sources/source_compatibility.js";
import { rawInputBlockers } from "./patching/patch_planning.js";
import { createReactiveFormBindings } from "./reactive_summary_bindings.js";
import { readFormFieldWritePosture, readFormReadiness } from "./form_runtime_policy.js";
import { validationReadinessBlockers } from "./validation/artifacts.js";
import { createAsyncValidationStore } from "./validation/async_execution.js";
import { materializeValidationDeclarations } from "./validation/declarations.js";
import { cloneFormValue, mergeDraft } from "./values/value_paths.js";
import { createFormControllerBootstrapFacade } from "./form_controller_bootstrap.js";

export function createFormController(signalNamespace, declaration, options = {}) {
  if (!declaration || typeof declaration !== "object") {
    throw new FormDeclarationError("signals.form(...) expects a declaration object");
  }
  if (!("source" in declaration)) {
    throw new FormDeclarationError("signals.form(...) requires a source value or signal");
  }
  if (typeof options.requireRouteFormsAuthorityArtifact !== "function") {
    throw new FormDeclarationError(
      "signals.form(...) requires a route authority validator from the signals runtime",
    );
  }
  const sourceAuthority = materializeFormSourceAuthority(declaration.source);
  const formDeclaration = materializeFormDeclarationRecord(declaration, sourceAuthority);
  let draft = {};
  const fieldDeclarations = materializeFieldDeclarations(declaration);
  const hostBindings = materializeHostBindings(declaration);
  const validationDeclarations = materializeValidationDeclarations(declaration, fieldDeclarations);
  const availabilityDeclarations = materializeAvailabilityDeclarations(declaration, fieldDeclarations);
  const admissionDeclarations = materializeAdmissionDeclarations(declaration, fieldDeclarations);
  const stepDeclarations = materializeStepDeclarations(declaration, fieldDeclarations);
  const actionDeclarations = materializeActionDeclarations(declaration, stepDeclarations, fieldDeclarations);
  const fieldHandles = {};
  const rawInputs = new Map();
  const parseFailures = new Map();
  const actionAttempts = createActionAttemptStore();
  const actionExecutions = createActionExecutionStore(actionAttempts);
  const asyncValidations = createAsyncValidationStore(validationDeclarations, fieldDeclarations);
  const exits = createExitPresentationStore();
  const handoffs = createHandoffStore();
  const routeAuthority = createRouteAuthorityStore();
  const attachments = createAttachmentPresentationStore();
  const canonicalizations = createCanonicalizationStore();
  const interactions = createInteractionStore();
  const navigation = createNavigationStore();
  const sourceCompatibility = createSourceCompatibilityStore(declaration.source);
  const collaborationDeclaration = materializeCollaborationDeclaration(declaration, fieldDeclarations);
  const collaborations = createCollaborationStore();
  const measurementPolicy = materializeMeasurementDeclaration(declaration);
  const layoutMeasurements = createLayoutMeasurementStore(measurementPolicy);
  const media = createMediaPresentationStore();
  const messages = createMessagePresentationStore();
  const diagnosticsHistory = createFormDiagnosticsHistoryStore();
  const stateHistory = createFormStateHistoryStore();
  const form = createFormControllerBootstrapFacade();
  const reactiveBindings = createReactiveFormBindings(
    signalNamespace,
    formDeclaration.formId,
    () => form,
  );
  const measurementSemanticCache = { value: null };
  const currentMeasurementSemanticContext = createMeasurementSemanticContextReader({
    cache: measurementSemanticCache,
    authoritativeSource,
    draft: () => draft,
    rawInputs,
    parseFailures,
    asyncValidations,
    sourceCompatibility,
    formRef: () => form,
  });
  const diagnosticsBindings = createDiagnosticsControllerBindings({
    formRef: () => form,
    fieldDeclarations,
    diagnosticsHistory,
    currentMeasurementSemanticContext,
  });
  const resourceMerges = createResourceMergeStore();
  const resourceDrifts = createResourceDriftStore();
  const resets = createFormResetStore();
  const replayRestores = createFormReplayRestoreStore();
  const presentationPolicy = materializePresentationDeclaration(declaration);
  const presentationSettlements = createPresentationStore();
  const presentationScopeRegistry = createDeclaredPresentationScopeRegistry(
    fieldDeclarations,
    stepDeclarations,
    actionDeclarations,
    availabilityDeclarations,
  );
  const resourceMergeRegistry = createResourceMergeProjectionRegistry(
    fieldDeclarations,
    stepDeclarations,
    availabilityDeclarations,
  );
  const interactionBindings = createInteractionBindings(interactions, fieldDeclarations);
  const presentationBindings = createPresentationBindings(
    () => form,
    () => syncSourceCompatibility(authoritativeSource()),
    presentationPolicy,
    actionDeclarations,
    stepDeclarations,
    presentationSettlements,
    handoffs,
    exits,
    attachments,
    media,
    presentationScopeRegistry,
  );
  const presentationRuntimeBindings = createPresentationRuntimeBindings({
    presentationPolicy,
    presentationBindings,
    exits,
    handoffs,
    attachments,
    media,
    messages,
    scopeRegistry: presentationScopeRegistry,
    collaborationDeclaration,
    collaborations,
    readResourceSource: () => (
      typeof form.resourceSource === "function" ? form.resourceSource() : null
    ),
  });
  const actionRuntimeBindings = createActionRuntimeBindings({
    formRef: () => form,
    actionAttempts,
    actionExecutions,
    asyncValidations,
    canonicalizations,
    sourceAuthority,
    source: declaration.source,
    fieldDeclarations,
    setDraft: updateDraft,
    applyControllerLocalNavigation(execution) {
      applyLocalStepNavigation(navigation, form, execution);
    },
    reportRouteHandoff(plan, execution) {
      reportRouteHandoffForExecution(form, plan, execution);
    },
    resets,
    replayRestores,
  });
  const recoveryBindings = createRecoveryBindings({
    formRef: () => form,
    source: declaration.source,
    writeDraft: updateDraft,
    resets,
    replayRestores,
  });
  const derivedReportBindings = createDerivedReportBindings({
    formRef: () => form,
    sourceDeclaration: declaration.source,
    syncSourceCompatibility,
    authoritativeSource,
    fieldDeclarations,
    rawInputs,
    parseFailures,
    asyncValidations,
    validationDeclarations, availabilityDeclarations, admissionDeclarations, stepDeclarations, actionDeclarations,
  });
  const resourceSurfaceBindings = createResourceSurfaceBindings({
    formRef: () => form,
    fieldDeclarations,
    signalNamespace,
    source: declaration.source,
    authoritativeSource,
    syncSourceCompatibility,
    latestCanonicalSourceDigest: () => canonicalizations.history().at(-1)?.sourceBasisDigest ?? null,
    draft: () => cloneFormValue(draft),
    effective: () => mergeDraft(authoritativeSource(), draft),
    resourceMerges,
    resourceMergeRegistry,
    resourceDrifts,
  });
  const stateHistoryBindings = createStateHistoryControllerBindings({
    formRef: () => form,
    stateHistory,
  });
  const fieldState = {
    writeDraft: updateDraft,
    recordRawInput: stateHistoryBindings.recordRawInput,
    recordDraftWrite: stateHistoryBindings.recordDraftWrite,
    interactions, rawInputs, parseFailures,
  };

  const formReportBindings = createFormReportBindings({
    formRef: () => form,
    fieldDeclarations,
    requireRouteFormsAuthorityArtifact: options.requireRouteFormsAuthorityArtifact,
    hostBindings,
    syncSourceCompatibility,
    authoritativeSource,
    exits,
    handoffs,
    routeAuthority,
    writeDraft: updateDraft,
    recordDraftWrite: stateHistoryBindings.recordDraftWrite,
    attachments,
    media,
    messages,
    collaborationDeclaration,
    collaborations,
    interactions,
    navigation,
    layoutMeasurements,
  });

  Object.assign(form, {
    source() {
      const currentSource = authoritativeSource();
      syncSourceCompatibility(currentSource);
      return cloneFormValue(currentSource);
    },
    sourceAuthority() { return sourceAuthority.diagnostics(); },
    declaration() { return readFormDeclarationDiagnostics(formDeclaration, sourceAuthority, fieldDeclarations); },
    fieldContract() { return readFieldContractDiagnostics(fieldDeclarations); },
    inputAdapters() { return readInputAdapterDiagnostics(fieldDeclarations); },
    draft() {
      syncSourceCompatibility(authoritativeSource());
      return cloneFormValue(draft);
    },
    effective() {
      const currentSource = authoritativeSource();
      syncSourceCompatibility(currentSource);
      return mergeDraft(currentSource, draft);
    },
    sourceAdmission() { return readSourceBootstrapArtifact(declaration.source, "sourceAdmission"); },
    draftRestore() { return readSourceBootstrapArtifact(declaration.source, "draftRestore"); },
    summarySignal() { return reactiveBindings.summarySignalHandle(); },
    ...resourceSurfaceBindings,
    ...formReportBindings,
    ...interactionBindings,
    presentation: presentationBindings.presentation,
    presentationLifecycle: presentationBindings.presentationLifecycle,
    reportPresentationLane: presentationBindings.reportPresentationLane,
    clearPresentationLane: presentationBindings.clearPresentationLane,
    acknowledgePresentation: presentationBindings.acknowledgePresentation,
    timeoutPresentation: presentationBindings.timeoutPresentation,
    presentationHistory: presentationBindings.presentationHistory,
    ...presentationRuntimeBindings,
    recordLayoutMeasurement(rows, options = {}) { syncSourceCompatibility(authoritativeSource()); return layoutMeasurements.record(currentMeasurementSemanticContext(), rows, options); },
    ...derivedReportBindings,
    actionPlan(actionId) {
      return findActionPlan(actionDeclarations, form, fieldDeclarations, actionId, declaration.source);
    },
    ...actionRuntimeBindings,
    ...recoveryBindings,
    sourceCompatibility() { return syncSourceCompatibility(authoritativeSource()); },
    sourceCompatibilityHistory() { syncSourceCompatibility(authoritativeSource()); return sourceCompatibility.history(); },
    stateHistory() { return stateHistory.history(); },
    actionReadiness(actionId) { return form.actionPlan(actionId).readiness; },
    ...diagnosticsBindings,
    fieldWritePosture(fieldId, capability = "edit") {
      return readFormFieldWritePosture({
        form,
        fieldId,
        capability,
        availabilityEditBlocker,
        admissionCapabilityBlocker,
        collaborationFieldWriteBlocker,
        routeAuthorityWriteBlocker,
        sourceCompatibilityBlockers,
      });
    },
    readiness() {
      return readFormReadiness({
        form,
        rawInputs,
        sourceCompatibilityBlockers,
        resourceSourceReadinessBlockers,
        resourceMergeReadinessBlockers,
        attachmentTransferReadinessBlockers,
        validationReadinessBlockers,
        availabilityReadinessBlockers,
        admissionReadinessBlockers,
        collaborationReadinessBlockers,
        routeAuthorityReadinessBlockers,
        hostRequirementBlockers,
        resolveResourceActionBinding,
        resolveResourceEffectProfileBinding,
        declarationSource: declaration.source,
        fieldDeclarations,
        actionDeclarations,
        dedupeReadinessBlockers,
        rawInputBlockers,
      });
    },
    setFields(values, options) { return applyFormFieldUpdates(form, fieldDeclarations, fieldState, values, options); },
    fields: fieldHandles,
    namespace: signalNamespace,
  });

  registerFormFields(form, fieldHandles, fieldDeclarations, fieldState, reactiveBindings);
  reactiveBindings.wrapControllerMutations(form);
  reactiveBindings.initialize();
  return Object.freeze(form);

  function authoritativeSource() { return canonicalizations.sourceFor(sourceAuthority.read()); }

  function updateDraft(nextDraft) {
    sourceCompatibility.noteDraft(nextDraft, authoritativeSource());
    draft = nextDraft;
  }

  function syncSourceCompatibility(rawSource) {
    const resolution = sourceCompatibility.reconcile(rawSource, draft);
    if (resolution.draft !== draft) {
      draft = resolution.draft;
    }
    return resolution.report;
  }
}
