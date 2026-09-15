import {
  buildCompatibilityDefinition,
  buildGraphExportDefinition,
  freezeObject,
  requireKnownInput,
  requireKnownOutput,
} from "../graph_support.js";
import {
  inspectWorkerFirstRootGraphDiagnostics,
  inspectWorkerFirstRootGraphHistory,
} from "./worker_first_root_graph_inspection.js";
import {
  buildImportedInputSignalRecord,
  buildImportedSignalRecord,
  normalizeImportedGraphSessionOptions,
} from "./sessions/support/worker_first_imported_graph_support.js";
import {
  buildImportedGraphMutationContext,
  buildImportedGraphOperationalContract,
  buildImportedGraphSnapshotArtifact,
  planImportedGraphMutation,
} from "../imported_graph_surface_support.js";
import { materializeWorkerCachedValue } from "./sessions/support/worker_cached_value.js";

export function createWorkerFirstRootImportedGraph(rootSession, options) {
  return new WorkerFirstRootImportedGraph(rootSession, options).api();
}

class WorkerFirstRootImportedGraph {
  #rootSession;
  #definition;
  #snapshot;
  #graphId;
  #inputDescriptors;
  #outputDescriptors;
  #operationalContract;
  #mutationContext;
  #trackedInputIds;
  #trackedOutputIds;
  #inputs;
  #outputs;
  #cachedInputs;
  #cachedOutputs;
  #cachedDiagnosticsSummary;
  #cachedDiagnosticsHistory;
  #ready;
  #readyPromise;
  #readyError;
  #terminated;
  #invalidatedMessage;

  constructor(rootSession, options) {
    const normalized = normalizeImportedGraphSessionOptions(options);
    this.#rootSession = rootSession;
    this.#definition = normalized.definition;
    this.#snapshot = normalized.snapshot;
    this.#graphId = normalized.definition.id;
    this.#inputDescriptors = normalized.definition.inputDescriptors;
    this.#outputDescriptors = normalized.definition.descriptors;
    this.#operationalContract = buildImportedGraphOperationalContract(normalized.definition);
    this.#mutationContext = buildImportedGraphMutationContext(
      normalized.definition,
      normalized.snapshot,
    );
    this.#trackedInputIds = normalized.trackedInputIds;
    this.#trackedOutputIds = normalized.trackedOutputIds;
    this.#cachedInputs = new Map();
    this.#cachedOutputs = new Map();
    this.#cachedDiagnosticsSummary = null;
    this.#cachedDiagnosticsHistory = null;
    this.#ready = false;
    this.#readyError = null;
    this.#terminated = false;
    this.#invalidatedMessage = null;
    this.#inputs = buildImportedInputSignalRecord(
      rootSession,
      this.#inputDescriptors,
      "inputName",
      (descriptor) => descriptor.sourceId,
      (descriptor) => this.#readCachedInput(descriptor.sourceId),
      (mutation) => this.apply(mutation),
    );
    this.#outputs = buildImportedSignalRecord(
      rootSession,
      this.#outputDescriptors,
      "outputName",
      (descriptor) => descriptor.publishedId,
      (descriptor) => this.#readCachedOutput(descriptor.publishedId),
    );
    this.#readyPromise = this.#initialize().catch((error) => {
      this.#readyError = error;
      throw error;
    });
  }

  api() {
    return freezeObject({
      id: this.#graphId,
      inputs: this.#inputs,
      outputs: this.#outputs,
      ready: () => this.ready(),
      contract: () => this.#definition.contract,
      contractHistory: () => this.#snapshot.contractHistory,
      importPosture: () => this.importPosture(),
      operationalContract: () => this.#operationalContract,
      input: (name) => this.input(name),
      output: (name) => this.output(name),
      read: () => this.read(),
      readInputs: () => this.readInputs(),
      writeInputs: (values) => this.writeInputs(values),
      writeInput: (name, value) => this.writeInput(name, value),
      patchInputs: (patches) => this.patchInputs(patches),
      patchInput: (name, patch) => this.patchInput(name, patch),
      resetInputs: (names) => this.resetInputs(names),
      resetInput: (name) => this.resetInput(name),
      apply: (mutation) => this.apply(mutation),
      summary: () => this.#definition.summary,
      inputDescriptors: () => this.#inputDescriptors,
      descriptors: () => this.#outputDescriptors,
      inspectDiagnostics: () => this.inspectDiagnostics(),
      inspectHistory: () => this.inspectHistory(),
      exportCompatibilityDefinition: () => this.exportCompatibilityDefinition(),
      exportDefinition: () => this.exportDefinition(),
      exportSnapshot: () => this.exportSnapshot(),
      terminate: () => this.terminate(),
    });
  }

  invalidate(message) {
    if (this.#invalidatedMessage !== null || this.#terminated) {
      return;
    }
    this.#invalidatedMessage = message;
    this.#cachedInputs.clear();
    this.#cachedOutputs.clear();
    this.#cachedDiagnosticsSummary = null;
    this.#cachedDiagnosticsHistory = null;
  }

  isInvalidated() {
    return this.#invalidatedMessage !== null || this.#terminated;
  }

  invalidatedError(operation) {
    return new TypeError(
      `worker-first imported graph ${this.#graphId} ${operation}() cannot be used because ${this.#invalidatedMessage ?? "it was terminated"}`,
    );
  }

  ready() {
    return this.#readyPromise;
  }

  input(name) {
    this.#requireActive("input");
    return requireKnownInput(this.#inputs, this.#graphId, name);
  }

  output(name) {
    this.#requireActive("output");
    return requireKnownOutput(this.#outputs, this.#graphId, name);
  }

  readInputs() {
    this.#requireHydrated("readInputs");
    const snapshot = Object.create(null);
    for (const descriptor of this.#inputDescriptors) {
      snapshot[descriptor.inputName] = this.#readCachedInput(descriptor.sourceId);
    }
    return freezeObject(snapshot);
  }

  read() {
    this.#requireHydrated("read");
    const snapshot = Object.create(null);
    for (const descriptor of this.#outputDescriptors) {
      snapshot[descriptor.outputName] = this.#readCachedOutput(descriptor.publishedId);
    }
    return freezeObject(snapshot);
  }

  async writeInputs(values) {
    return this.apply({ writes: values });
  }

  async writeInput(name, value) {
    return this.apply({ writes: { [name]: value } });
  }

  async patchInputs(patches) {
    return this.apply({ patches });
  }

  async patchInput(name, patch) {
    return this.apply({ patches: { [name]: patch } });
  }

  async resetInputs(names = this.#mutationContext.defaultResetInputNames) {
    return this.apply({ reset: names });
  }

  async resetInput(name) {
    return this.apply({ reset: [name] });
  }

  async apply(mutation) {
    await this.ready();
    this.#requireActive("apply");
    const transactionOps = planImportedGraphMutation({
      label: `worker-first imported graph ${this.#graphId}`,
      graphId: this.#graphId,
      inputDescriptorsByName: this.#mutationContext.inputDescriptorsByName,
      inputAuthoritiesByName: this.#mutationContext.inputAuthoritiesByName,
      initialValuesBySourceId: this.#mutationContext.initialValuesBySourceId,
      readCurrentValue: (sourceId, inputName) => this.#readCachedInput(sourceId, inputName),
      mutation,
    });
    return this.#rootSession.applyImportMutation(this, transactionOps, this.#trackedOutputIds);
  }

  inspectDiagnostics() { this.#requireHydrated("inspectDiagnostics"); return inspectWorkerFirstRootGraphDiagnostics(this); }
  inspectHistory() { this.#requireHydrated("inspectHistory"); return inspectWorkerFirstRootGraphHistory(this); }

  async refreshFromRootRuntime() {
    if (this.#terminated || this.#invalidatedMessage !== null || !this.#ready) {
      return;
    }
    await this.#refreshCaches();
  }

  exportCompatibilityDefinition() {
    this.#requireActive("exportCompatibilityDefinition");
    return buildCompatibilityDefinition(
      this.#graphId,
      this.#definition.summary,
      this.#inputDescriptors,
      this.#outputDescriptors,
      this.#definition.compatibility.definitions,
    );
  }

  exportDefinition() {
    this.#requireActive("exportDefinition");
    return buildGraphExportDefinition(
      this.#graphId,
      this.#definition.summary,
      this.#definition.contract,
      this.#operationalContract,
      this.#inputDescriptors,
      this.#outputDescriptors,
      this.#definition.compatibility.definitions,
    );
  }

  exportSnapshot() {
    this.#requireActive("exportSnapshot");
    const context = this.#rootSession.currentImportContext();
    const runtimeEnvelope = context.requireRuntimeEnvelopeArtifact();
    return buildImportedGraphSnapshotArtifact({
      definition: this.exportDefinition(),
      runtimeEnvelope,
      snapshotEnvelope: context.snapshotEnvelope,
      restoreMode: runtimeEnvelope.runtimeEnvelopeRestoreMode,
      contractHistory: this.#snapshot.contractHistory,
      importPosture: this.importPosture(),
    });
  }

  importPosture() {
    this.#requireActive("importPosture");
    return freezeObject({
      ...this.#snapshot.importPosture,
      graphId: this.#graphId,
      hydrate: this.#ready ? "Applied" : "Deferred",
      hydrateReason: this.#ready
        ? "worker-first root imported graph restored exact runtime envelope into root-owned worker truth"
        : "worker-first root imported graph requires await importedGraph.ready() before exact runtime restore is visible through the shared worker runtime",
    });
  }

  async terminate() {
    if (this.#terminated) {
      return;
    }
    this.#terminated = true;
    this.#rootSession.detachImport(this);
    this.#cachedInputs.clear();
    this.#cachedOutputs.clear();
    this.#cachedDiagnosticsSummary = null;
    this.#cachedDiagnosticsHistory = null;
  }

  diagnosticsSummary() {
    this.#requireHydrated("diagnosticsSummary");
    return this.#cachedDiagnosticsSummary;
  }

  diagnosticsHistory() {
    this.#requireHydrated("diagnosticsHistory");
    return this.#cachedDiagnosticsHistory;
  }

  get definition() { return this.#definition; }
  get rootSession() { return this.#rootSession; }

  async #initialize() {
    try {
      await this.#rootSession.beginExactImport(this.#definition, this.#snapshot, this);
      await this.#refreshCaches();
      this.#ready = true;
    } catch (error) {
      this.#readyError = error;
      throw error;
    }
  }

  async #refreshDiagnosticsCache() {
    const [summaryPacket, historyPacket] = await Promise.all([
      this.#rootSession.bridge().readDiagnosticsSummary(),
      this.#rootSession.bridge().readDiagnosticsHistory(),
    ]);
    this.#cachedDiagnosticsSummary = materializeWorkerCachedValue(summaryPacket.summary);
    this.#cachedDiagnosticsHistory = materializeWorkerCachedValue(historyPacket.history);
  }

  async #refreshCaches() {
    const [inputPacket, outputPacket] = await Promise.all([
      this.#rootSession.bridge().readSignals({ signalIds: this.#trackedInputIds }),
      this.#rootSession.bridge().deliverOutputs({ outputIds: this.#trackedOutputIds }),
      this.#refreshDiagnosticsCache(),
    ]);
    this.#cachedInputs.clear();
    for (const signal of inputPacket.signals) {
      this.#cachedInputs.set(signal.id, materializeWorkerCachedValue(signal.value));
    }
    this.#cachedOutputs.clear();
    for (const output of outputPacket.outputs) {
      this.#cachedOutputs.set(output.id, materializeWorkerCachedValue(output.value));
    }
  }

  #readCachedInput(id) {
    this.#requireHydrated("input.get");
    const tipped = this.#readImportContextTip(id);
    if (tipped.present) {
      return tipped.value;
    }
    if (!this.#cachedInputs.has(id)) {
      throw new TypeError(
        `worker-first imported graph ${this.#graphId} has no cached input readback for \`${id}\``,
      );
    }
    return this.#cachedInputs.get(id);
  }

  #readCachedOutput(id) {
    this.#requireHydrated("output.get");
    const tipped = this.#readImportContextTip(id);
    if (tipped.present) {
      return tipped.value;
    }
    if (!this.#cachedOutputs.has(id)) {
      throw new TypeError(
        `worker-first imported graph ${this.#graphId} has no cached output readback for \`${id}\``,
      );
    }
    return this.#cachedOutputs.get(id);
  }

  #readImportContextTip(id) {
    const context = typeof this.#rootSession.peekActiveImportContext === "function"
      ? this.#rootSession.peekActiveImportContext()
      : null;
    if (!context?.signalValueById?.has(id)) {
      return { present: false, value: undefined };
    }
    return { present: true, value: context.signalValueById.get(id) };
  }

  #requireActive(operation) {
    if (this.#terminated) {
      throw this.invalidatedError(operation);
    }
    if (this.#invalidatedMessage !== null) {
      throw this.invalidatedError(operation);
    }
  }

  #requireHydrated(operation) {
    this.#requireActive(operation);
    if (this.#ready) {
      return;
    }
    if (this.#readyError !== null) {
      throw this.#readyError;
    }
    throw new TypeError(
      `worker-first imported graph ${this.#graphId} ${operation}() requires await importedGraph.ready() before root-owned worker truth can be read`,
    );
  }
}
