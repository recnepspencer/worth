import { freezeObject } from "../graph_support.js";
import { buildHostCapabilityTransportReport } from "../host_capability_reports.js";
import { assertWorkerFirstHandleOwnership } from "./worker_first_handle_ownership.js";

export function createRootDiagnosticsFacade(rootSession) {
  return freezeObject({
    why(id) {
      return rootSession.why(id);
    },
    health() {
      rootSession.requireActiveDiagnostics("diagnostics.health");
      return diagnosticsContextOrLive(rootSession).health;
    },
    summaryNow() {
      rootSession.requireActiveDiagnostics("diagnostics.summaryNow");
      return diagnosticsContextOrLive(rootSession).diagnosticsSummary;
    },
    historyNow() {
      rootSession.requireActiveDiagnostics("diagnostics.historyNow");
      return diagnosticsContextOrLive(rootSession).diagnosticsHistory;
    },
    latestFlow() {
      rootSession.requireActiveDiagnostics("diagnostics.latestFlow");
      return diagnosticsContextOrLive(rootSession).latestFlow;
    },
    latestObservation() {
      rootSession.requireActiveDiagnostics("diagnostics.latestObservation");
      return diagnosticsContextOrLive(rootSession).latestObservation;
    },
    latestHostCapabilityEvent() {
      rootSession.requireActiveDiagnostics("diagnostics.latestHostCapabilityEvent");
      return rootSession.latestHostCapabilityEvent();
    },
    recentHostCapabilityEvents() {
      rootSession.requireActiveDiagnostics("diagnostics.recentHostCapabilityEvents");
      return rootSession.recentHostCapabilityEvents();
    },
    performanceSummary() {
      rootSession.requireActiveDiagnostics("diagnostics.performanceSummary");
      return diagnosticsContextOrLive(rootSession).performanceSummary;
    },
    latestFailure() {
      rootSession.requireActiveDiagnostics("diagnostics.latestFailure");
      return diagnosticsContextOrLive(rootSession).latestFailure;
    },
    latestRollback() {
      rootSession.requireActiveDiagnostics("diagnostics.latestRollback");
      return diagnosticsContextOrLive(rootSession).latestRollback;
    },
    latestInvalidationPlanningEstimate() {
      rootSession.requireActiveDiagnostics("diagnostics.latestInvalidationPlanningEstimate");
      return diagnosticsContextOrLive(rootSession).latestInvalidationPlanningEstimate;
    },
    latestInvalidationTraceRecords() {
      rootSession.requireActiveDiagnostics("diagnostics.latestInvalidationTraceRecords");
      return diagnosticsContextOrLive(rootSession).latestInvalidationTraceRecords;
    },
    recentHistory() {
      rootSession.requireActiveDiagnostics("diagnostics.recentHistory");
      return diagnosticsContextOrLive(rootSession).recentHistory;
    },
    hostCapabilityReport() {
      // Host-side evidence: the report is built from this root's own
      // authored callbacks and the denials its detached handles recorded,
      // never from the worker. It stays readable after terminate() so the
      // denials a detached handle produced can be inspected afterwards.
      return rootSession.hostCapabilityReport();
    },
    subscribe(listener) {
      return rootSession.subscribeDiagnostics(listener);
    },
  });
}

export function createRootAdaptersFacade(rootSession) {
  return freezeObject({
    exportDefinitions() {
      return rootSession.currentImportContext().runtimeDefinitionEnvelope;
    },
    exportRuntimeEnvelope() {
      return rootSession.currentImportContext().requireRuntimeEnvelopeArtifact();
    },
    async replaceRuntimeEnvelope(envelope) {
      return rootSession.replaceRuntimeEnvelope(envelope);
    },
    async restoreExactRuntimeEnvelope(envelope) {
      return rootSession.restoreExactRuntimeEnvelope(envelope);
    },
    discardExactRuntimeEnvelope(envelope) {
      const restoreToken = envelope?.runtimeEnvelopeRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "adapters.discardExactRuntimeEnvelope expects an artifact returned by adapters.exportRuntimeEnvelope()",
        );
      }
      return rootSession.bridge().discardRestoreToken(restoreToken);
    },
    runtimeProofReport() {
      return rootSession.currentImportContext().runtimeProofReport;
    },
    hostCapabilityTransportReport(envelope) {
      const definitions =
        envelope?.definitions ?? rootSession.currentImportContext().runtimeDefinitionEnvelope;
      return buildHostCapabilityTransportReport(definitions?.unavailableCallbacks);
    },
    free() {},
    [Symbol.dispose]() {},
  });
}

export function createRootSpecialistFacade(rootSession) {
  return freezeObject({
    evaluateDirty() {
      return rootSession.evaluateDirty();
    },
    evaluate_dirty() {
      return rootSession.evaluateDirty();
    },
    graphSummary() {
      return rootSession.currentImportContext().diagnosticsSummary;
    },
    graph_summary() {
      return rootSession.currentImportContext().diagnosticsSummary;
    },
    readVersions(ids) {
      return readVersionSummaries(rootSession, ids, "specialist.readVersions");
    },
    read_versions(ids) {
      return readVersionSummaries(rootSession, ids, "specialist.read_versions");
    },
    free() {},
    [Symbol.dispose]() {},
  });
}

export function readRootSignalValue(rootSession, target) {
  assertWorkerFirstHandleOwnership(rootSession, target, "worker-first root read(...)");
  const id = normalizeRootReadableTargetId(target);
  return rootSession.readSignalValue(id);
}

function diagnosticsContextOrLive(rootSession) {
  return rootSession.peekActiveImportContext() ?? rootSession.peekLiveRootDiagnostics();
}

function normalizeRootReadableTargetId(target) {
  if (typeof target === "string" && target.length > 0) {
    return target;
  }
  if (
    target &&
    typeof target === "object" &&
    typeof target.id === "string" &&
    target.id.length > 0
  ) {
    return target.id;
  }
  if (
    typeof target === "function" &&
    typeof target.id === "string" &&
    target.id.length > 0
  ) {
    return target.id;
  }
  throw new TypeError(
    "worker-first root read(...) requires a worker-first signal handle or canonical signal id",
  );
}

function readVersionSummaries(rootSession, ids, operation) {
  if (!Array.isArray(ids)) {
    throw new TypeError(`${operation}(...) requires an array of signal ids`);
  }
  const context = rootSession.currentImportContext();
  return ids.map((id) => {
    if (typeof id !== "string" || !context.versionById.has(id)) {
      throw new TypeError(
        `${operation}(...) requires signal ids from the active imported graph`,
      );
    }
    return context.versionById.get(id);
  });
}
