import { buildHostCapabilityTransportReport } from "../host_capability_reports.js";
import { normalizeWorkerRuntimeEnvelope } from "./bridge/worker_runtime_envelope_normalization.js";

export function createWorkerFirstAdaptersFacade(workerFirstSession) {
  return Object.freeze({
    exportDefinitions() {
      return workerFirstSession.bridge.exportDefinitions();
    },
    async exportRuntimeEnvelope() {
      const [runtimeEnvelope, runtimeEnvelopeRestoreToken, runtimeEnvelopePortableWire] =
        await Promise.all([
          workerFirstSession.bridge.exportWorkerRuntimeEnvelope(),
          workerFirstSession.bridge.exportWorkerRuntimeEnvelopeWire(),
          workerFirstSession.bridge.exportWorkerRuntimeEnvelopePortableWire(),
        ]);
      return Object.freeze({
        ...normalizeWorkerRuntimeEnvelope(runtimeEnvelope),
        runtimeEnvelopeRestoreToken,
        runtimeEnvelopeRestoreMode: "SameRuntimeExact",
        runtimeEnvelopePortableWire,
      });
    },
    replaceRuntimeEnvelope(envelope) {
      const portableWire = envelope?.runtimeEnvelopePortableWire;
      if (typeof portableWire !== "string") {
        throw new TypeError(
          "worker-first adapters.replaceRuntimeEnvelope(...) requires an artifact returned by exportRuntimeEnvelope()",
        );
      }
      return workerFirstSession.bridge.admitWorkerRuntimeEnvelopeImportPortableWire(portableWire);
    },
    restoreExactRuntimeEnvelope(envelope) {
      const restoreToken = envelope?.runtimeEnvelopeRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "worker-first adapters.restoreExactRuntimeEnvelope(...) requires an artifact returned by exportRuntimeEnvelope()",
        );
      }
      return workerFirstSession.bridge.admitWorkerRuntimeEnvelopeImportWire(restoreToken);
    },
    discardExactRuntimeEnvelope(envelope) {
      const restoreToken = envelope?.runtimeEnvelopeRestoreToken;
      if (typeof restoreToken !== "string") {
        throw new TypeError(
          "worker-first adapters.discardExactRuntimeEnvelope(...) requires an artifact returned by exportRuntimeEnvelope()",
        );
      }
      return workerFirstSession.bridge.discardRestoreToken(restoreToken);
    },
    runtimeProofReport() {
      return workerFirstSession.bridge.runtimeProofReport();
    },
    async hostCapabilityTransportReport(envelope) {
      const definitions =
        envelope?.definitions
        ?? (await workerFirstSession.bridge.exportDefinitions());
      return buildHostCapabilityTransportReport(definitions?.unavailableCallbacks);
    },
  });
}
