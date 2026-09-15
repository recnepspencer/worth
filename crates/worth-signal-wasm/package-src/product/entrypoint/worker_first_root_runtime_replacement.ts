export function createWorkerFirstRootRuntimeReplacement(deps) {
  return Object.freeze({
    async replaceRuntimeEnvelope(envelope) {
      const portableWire = envelope?.runtimeEnvelopePortableWire;
      if (typeof portableWire !== "string") {
        throw new TypeError(
          "worker-first root adapters().replaceRuntimeEnvelope(...) requires an artifact returned by adapters.exportRuntimeEnvelope()",
        );
      }
      await deps.ready();
      await deps.authoredRuntime.settlePendingPublications();
      deps.requireActive("adapters.replaceRuntimeEnvelope");
      const report = await deps.bridge.admitWorkerRuntimeEnvelopeImportPortableWire(portableWire);
      if (isWorkerRuntimeEnvelopeImportAdmitted(report)) {
        deps.invalidateActiveImport(
          "worker-first root adapters().replaceRuntimeEnvelope(...) replaced the active imported graph runtime",
        );
        deps.authoredRuntime.invalidate(
          "worker-first root adapters().replaceRuntimeEnvelope(...) replaced the worker-owned runtime",
        );
        await deps.observations.clearContext(deps.bridge);
        await deps.hostCapabilities.replayCurrentIngress();
        // The live diagnostics describe the runtime that now exists, not the
        // one that was replaced.
        await deps.publishDiagnosticsChanged();
      }
      return report;
    },
    async restoreExactRuntimeEnvelope(envelope) {
      const restoreToken = envelope?.runtimeEnvelopeRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "worker-first root adapters().restoreExactRuntimeEnvelope(...) requires an artifact returned by adapters.exportRuntimeEnvelope()",
        );
      }
      await deps.ready();
      await deps.authoredRuntime.settlePendingPublications();
      deps.requireActive("adapters.restoreExactRuntimeEnvelope");
      const report = await deps.bridge.admitWorkerRuntimeEnvelopeImportWire(restoreToken);
      if (isWorkerRuntimeEnvelopeImportAdmitted(report)) {
        deps.invalidateActiveImport(
          "worker-first root adapters().restoreExactRuntimeEnvelope(...) replaced the active imported graph runtime",
        );
        deps.authoredRuntime.invalidate(
          "worker-first root adapters().restoreExactRuntimeEnvelope(...) replaced the worker-owned runtime",
        );
        await deps.observations.clearContext(deps.bridge);
        await deps.hostCapabilities.replayCurrentIngress();
        await deps.publishDiagnosticsChanged();
      }
      return report;
    },
  });
}

function isWorkerRuntimeEnvelopeImportAdmitted(report) {
  return report?.importOutcome === "Admitted" || report?.importOutcome === "AdmittedExact";
}
