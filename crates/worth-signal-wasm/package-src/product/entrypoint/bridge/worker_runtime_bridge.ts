import {
  createBridgeBrowserHistoryIngressReport,
  createBridgeBrowserHistoryStory,
  normalizeBridgeBrowserHistoryIngress,
  normalizeBridgeBrowserHistoryWriteback,
  createBridgeBrowserHistoryWritebackReport,
} from "./worker_runtime_bridge_router_boundary.js";
import {
  attachErrorListener,
  attachMessageListener,
  deserializeError,
  normalizeOptionalWasmUrl,
  normalizeWorkerUrl,
} from "./worker_runtime_bridge_support.js";
import { createWorkerRuntimeBridgeHistory } from "./worker_runtime_bridge_history.js";
import { createWorkerRuntimeWasmBootstrapMessage } from "./worker_runtime_wasm_bootstrap.js";

export function createWorkerRuntimeBridge(options = {}) {
  return new WorkerRuntimeBridge(options);
}

class WorkerRuntimeBridge {
  #nextRequestId = 0;
  #pending = new Map();
  #history;
  #worker;
  #terminated = false;
  #terminating = null;

  constructor(options) {
    if (typeof globalThis.Worker !== "function") {
      throw new TypeError(
        "createWorkerRuntimeBridge requires a global Worker constructor",
      );
    }
    this.#worker = options.workerUrl
      ? new Worker(normalizeWorkerUrl(options.workerUrl), {
          type: "module",
        })
      : new Worker(
          new URL("./worker_runtime_bridge_worker.js", import.meta.url),
          { type: "module" },
        );
    this.#history = createWorkerRuntimeBridgeHistory(
      (method, ...args) => this.#request(method, ...args),
    );
    attachMessageListener(this.#worker, (message) => {
      const pending = this.#pending.get(message?.id);
      if (!pending) {
        return;
      }
      this.#pending.delete(message.id);
      if (message.ok) {
        pending.resolve(message.value);
        return;
      }
      pending.reject(deserializeError(message.error));
    });
    attachErrorListener(this.#worker, (event) => {
      const error = deserializeError({
        name: "WorkerRuntimeBridgeWorkerError",
        message: event.message || "Worker runtime bridge worker failed",
        stack: null,
      });
      this.#rejectAll(error);
    });
    // Worker top-level init waits for this before constructing the runtime.
    this.#worker.postMessage(
      createWorkerRuntimeWasmBootstrapMessage(
        normalizeOptionalWasmUrl(options.wasmUrl),
      ),
    );
  }

  bootstrapRecord() {
    return this.#request("bootstrapRecord");
  }

  workerRuntimeShellLock() {
    return this.#request("workerRuntimeShellLock");
  }

  publishPortableGraph(publication) {
    return this.#request("publishPortableGraph", publication);
  }

  applyTransaction(transactionOps) {
    return this.#request("applyTransaction", transactionOps);
  }

  applyTransactionProjection(request) {
    return this.#request("applyTransactionProjection", request);
  }

  localTruthCommand(envelope) {
    return this.#request("localTruthCommand", envelope);
  }

  attachObservationDelivery(request) {
    return this.#request("attachObservationDelivery", request);
  }

  detachObservationDelivery(request) {
    return this.#request("detachObservationDelivery", request);
  }

  why(id) {
    return this.#request("why", id);
  }

  latestFlow() {
    return this.#request("latestFlow");
  }

  latestObservation() {
    return this.#request("latestObservation");
  }

  health() {
    return this.#request("health");
  }

  performanceSummary() {
    return this.#request("performanceSummary");
  }

  latestFailure() {
    return this.#request("latestFailure");
  }

  latestRollback() {
    return this.#request("latestRollback");
  }

  latestInvalidationPlanningEstimate() {
    return this.#request("latestInvalidationPlanningEstimate");
  }

  latestInvalidationTraceRecords() {
    return this.#request("latestInvalidationTraceRecords");
  }

  recentHistory() {
    return this.#request("recentHistory");
  }

  currentBranch() { return this.#history.currentBranch(); }
  branches() { return this.#history.branches(); }
  createBranch(name) { return this.#history.createBranch(name); }
  workerBranchBasis(branchId) { return this.#history.workerBranchBasis(branchId); }
  forkBranch(request) { return this.#history.forkBranch(request); }
  applyTransactionToBranch(request) { return this.#history.applyTransactionToBranch(request); }
  retireBranch(request) { return this.#history.retireBranch(request); }
  retireBranches(request) { return this.#history.retireBranches(request); }
  closeoutEffectBranch(request) { return this.#history.closeoutEffectBranch(request); }
  switchBranch(branchId) { return this.#history.switchBranch(branchId); }
  planMergeBranches(source, target) { return this.#history.planMergeBranches(source, target); }
  planMergeBranchesWithProof(source, target) { return this.#history.planMergeBranchesWithProof(source, target); }
  mergeBranches(source, target) { return this.#history.mergeBranches(source, target); }
  mergeBranchesWithProof(source, target) { return this.#history.mergeBranchesWithProof(source, target); }
  planMergePolicyPreview(request) { return this.#history.planMergePolicyPreview(request); }
  planMergePolicyPreviewWithProof(request) { return this.#history.planMergePolicyPreviewWithProof(request); }
  mergeBranchesPolicyPreview(request) { return this.#history.mergeBranchesPolicyPreview(request); }
  mergeBranchesPolicyPreviewWithProof(request) { return this.#history.mergeBranchesPolicyPreviewWithProof(request); }
  replayForBranch(branchId) { return this.#history.replayForBranch(branchId); }
  branchSnapshotId(branchId) { return this.#history.branchSnapshotId(branchId); }
  branchSnapshotEnvelope(branchId) { return this.#history.branchSnapshotEnvelope(branchId); }
  branchSnapshotArtifact(branchId) { return this.#history.branchSnapshotArtifact(branchId); }
  branchSnapshotEnvelopeArtifact(branchId) { return this.#history.branchSnapshotEnvelopeArtifact(branchId); }
  branchSnapshotEnvelopeWire(branchId) { return this.#history.branchSnapshotEnvelopeWire(branchId); }
  branchSnapshotEnvelopePortableWire(branchId) { return this.#history.branchSnapshotEnvelopePortableWire(branchId); }
  restoreBranchSnapshotArtifact(branchId, snapshot) { return this.#history.restoreBranchSnapshotArtifact(branchId, snapshot); }
  restoreBranchSnapshotWire(branchId, snapshot) { return this.#history.restoreBranchSnapshotWire(branchId, snapshot); }
  restoreBranchSnapshotPortableWire(branchId, snapshot) { return this.#history.restoreBranchSnapshotPortableWire(branchId, snapshot); }
  restoreBranchSnapshotById(branchId, snapshotId) { return this.#history.restoreBranchSnapshotById(branchId, snapshotId); }
  branchStateProof(branchId) { return this.#history.branchStateProof(branchId); }

  replayFor(id) {
    return this.#request("replayFor", id);
  }

  lineageFor(id) {
    return this.#request("lineageFor", id);
  }

  readVersions(ids) {
    return this.#request("readVersions", ids);
  }

  evaluateDirty() {
    return this.#request("evaluateDirty");
  }

  exportDefinitions() {
    return this.#request("exportDefinitions");
  }

  exportWorkerRuntimeEnvelope() {
    return this.#request("exportWorkerRuntimeEnvelope");
  }

  exportWorkerRuntimeEnvelopeWire() {
    return this.#request("exportWorkerRuntimeEnvelopeWire");
  }

  discardRestoreToken(token) {
    return this.#request("discardRestoreToken", token);
  }

  exportWorkerRuntimeEnvelopePortableWire() {
    return this.#request("exportWorkerRuntimeEnvelopePortableWire");
  }

  exportWorkerSnapshotEnvelope() {
    return this.#request("exportWorkerSnapshotEnvelope");
  }

  exportWorkerSnapshotEnvelopeArtifact() {
    return this.#request("exportWorkerSnapshotEnvelopeArtifact");
  }

  exportWorkerSnapshotEnvelopeWire() {
    return this.#request("exportWorkerSnapshotEnvelopeWire");
  }

  exportWorkerSnapshotEnvelopePortableWire() {
    return this.#request("exportWorkerSnapshotEnvelopePortableWire");
  }

  restoreSnapshotEnvelope(snapshot) {
    return this.#request("restoreSnapshotEnvelope", snapshot);
  }

  restoreSnapshotEnvelopeWire(snapshot) {
    return this.#request("restoreSnapshotEnvelopeWire", snapshot);
  }

  restoreSnapshotEnvelopePortableWire(snapshot) {
    return this.#request("restoreSnapshotEnvelopePortableWire", snapshot);
  }

  admitWorkerRuntimeEnvelopeImportWire(envelope) {
    return this.#request("admitWorkerRuntimeEnvelopeImportWire", envelope);
  }

  admitWorkerRuntimeEnvelopeImportPortableWire(envelope) {
    return this.#request("admitWorkerRuntimeEnvelopeImportPortableWire", envelope);
  }

  runtimeProofReport() {
    return this.#request("runtimeProofReport");
  }

  admitHostCapabilityIngress(batch) {
    return this.#request("admitHostCapabilityIngress", batch);
  }

  async admitBrowserHistoryIngress(ingress) {
    const normalizedIngress = normalizeBridgeBrowserHistoryIngress(
      ingress,
      "workerRuntimeBridge.admitBrowserHistoryIngress(...)",
    );
    const report = await this.#request("admitBrowserHistoryIngress", normalizedIngress);
    return createBridgeBrowserHistoryIngressReport(normalizedIngress, report);
  }

  async applyBrowserHistoryWriteback(writeback) {
    const normalizedWriteback = normalizeBridgeBrowserHistoryWriteback(
      writeback,
      "workerRuntimeBridge.applyBrowserHistoryWriteback(...)",
    );
    return createBridgeBrowserHistoryWritebackReport(
      normalizedWriteback,
      async (ingress) => this.#request("admitBrowserHistoryIngress", ingress),
      async () => this.#request("readDiagnosticsSummary"),
    );
  }

  browserHistoryStory(initialReport) {
    return createBridgeBrowserHistoryStory(initialReport);
  }

  issueHostEffectRequest(request) {
    return this.#request("issueHostEffectRequest", request);
  }

  admitHostEffectAcknowledgement(acknowledgement) {
    return this.#request("admitHostEffectAcknowledgement", acknowledgement);
  }

  certifyMainThreadHostBridge() {
    return this.#request("certifyMainThreadHostBridge");
  }

  deliverLatestObservation() {
    return this.#request("deliverLatestObservation");
  }

  deliverOutputs(request) {
    return this.#request("deliverOutputs", request);
  }

  readSignals(request) {
    return this.#request("readSignals", request);
  }

  readDiagnosticsSummary() {
    return this.#request("readDiagnosticsSummary");
  }

  readDiagnosticsHistory() {
    return this.#request("readDiagnosticsHistory");
  }

  async terminate() {
    if (this.#terminated) {
      return;
    }
    if (this.#terminating !== null) {
      await this.#terminating;
      return;
    }
    this.#terminating = this.#terminateWhenIdle();
    try {
      await this.#terminating;
    } finally {
      this.#terminating = null;
    }
  }

  #request(method, ...args) {
    if (this.#terminated) {
      return Promise.reject(new Error("Worker runtime bridge terminated"));
    }
    const id = ++this.#nextRequestId;
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve, reject, method });
      this.#worker.postMessage({ id, method, args });
    });
  }

  async #terminateWhenIdle() {
    await this.#waitForIdle();
    this.#worker.terminate();
    this.#terminated = true;
    const pendingMethods = [...new Set([...this.#pending.values()].map((pending) => pending.method))];
    const suffix = pendingMethods.length === 0
      ? ""
      : ` while requests were pending: ${pendingMethods.join(", ")}`;
    this.#rejectAll(new Error(`Worker runtime bridge terminated${suffix}`));
  }

  async #waitForIdle() {
    let idlePasses = 0;
    for (let attempts = 0; attempts < 50 && idlePasses < 2; attempts += 1) {
      if (this.#pending.size === 0) {
        idlePasses += 1;
      } else {
        idlePasses = 0;
      }
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }

  #rejectAll(error) {
    for (const pending of this.#pending.values()) {
      pending.reject(error);
    }
    this.#pending.clear();
  }
}
