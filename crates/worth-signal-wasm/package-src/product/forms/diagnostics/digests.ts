import { stableValueDigest } from "../values/value_paths.js";
import { digestFormDiagnosticsHistory } from "./history.js";

export function digestFormDiagnosticsProof(summary, diagnosticsStateDigest, history) {
  return stableValueDigest({
    diagnosticsStateDigest,
    summary,
    diagnosticsHistoryDigest: digestFormDiagnosticsHistory(history),
  });
}

export function readFormDiagnosticsStateDigestInput(state) {
  return Object.freeze({
    declaration: state.declaration,
    sourceAuthority: state.sourceAuthority,
    fieldContract: state.fieldContract,
    inputAdapters: state.inputAdapters,
    dirty: state.dirty,
    patchPlan: state.patchPlan,
    readiness: state.readiness,
    validation: state.validation,
    availability: state.availability,
    admission: state.admission,
    resourceSource: state.resourceSource,
    resourceDrift: state.resourceDrift,
    attachmentTransfers: state.attachmentTransfers,
    host: state.host,
    inputCapabilities: state.inputCapabilities,
    exit: state.exit,
    handoff: state.handoff,
    routeAuthority: state.routeAuthority,
    attachments: state.attachments,
    media: state.media,
    messages: state.messages,
    collaboration: state.collaboration,
    interaction: state.interaction,
    navigation: state.navigation,
    accessibility: state.accessibility,
    layout: state.layout,
    layoutMeasurement: state.layoutMeasurement,
    presentation: state.presentation,
    sourceCompatibility: state.sourceCompatibility,
    steps: state.steps,
    actions: state.actions,
    actionHistory: state.actionHistory,
    actionExecutionHistory: state.actionExecutionHistory,
    presentationHistory: state.presentationHistory,
    asyncValidationHistory: state.asyncValidationHistory,
    canonicalizationHistory: state.canonicalizationHistory,
    resetHistory: state.resetHistory,
    stateHistory: state.stateHistory.map((entry) => entry.stateHistoryDigest),
    replayRestoreHistory: state.replayRestoreHistory,
    sourceCompatibilityHistory: state.sourceCompatibilityHistory,
  });
}
