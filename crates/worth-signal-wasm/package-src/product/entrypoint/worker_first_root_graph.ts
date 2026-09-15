import { buildImportedGraphSnapshotArtifact } from "../imported_graph_surface_support.js";
import {
  buildCompatibilityDefinition,
  buildGraphContractDelta,
  buildGraphContractHistory,
  buildGraphDiagnosticsSurface,
  buildGraphExportDefinition,
  buildGraphHistorySurface,
  freezeObject,
} from "../graph_support.js";
import {
  createRootAdaptersFacade,
  createRootDiagnosticsFacade,
  createRootSpecialistFacade,
} from "./worker_first_root_cached_facades.js";
import { createRootHistoryFacade } from "./worker_first_root_history.js";
import {
  normalizeWorkerFirstRootGraphDefinition,
  resolveWorkerFirstRootGraphDefinition,
} from "./worker_first_root_graph_support.js";
import {
  applyWorkerFirstRootGraphMutation,
  runWorkerFirstRootGraphTransaction,
} from "./worker_first_root_graph_mutation.js";

export function createWorkerFirstRootGraph(rootSession, path, graphId, definitionOrBuilder) {
  const context = rootSession.currentImportContext();
  const definition = resolveWorkerFirstRootGraphDefinition(
    rootSession,
    path,
    graphId,
    definitionOrBuilder,
  );
  return new WorkerFirstRootGraph(rootSession, context, graphId, definition).api();
}

class WorkerFirstRootGraph {
  #rootSession;
  #context;
  #graphId;
  #graphSummary;
  #contract;
  #contractHistory;
  #operationalContract;
  #inputDescriptors;
  #outputDescriptors;
  #inputs;
  #outputs;
  #mutationContext;
  #invalidatedMessage;

  constructor(rootSession, context, graphId, definition) {
    const normalized = normalizeWorkerFirstRootGraphDefinition(context, graphId, definition);
    this.#rootSession = rootSession;
    this.#context = context;
    this.#graphId = graphId;
    this.#graphSummary = normalized.graphSummary;
    this.#contract = normalized.contract;
    this.#contractHistory = buildGraphContractHistory(this.#contract);
    this.#operationalContract = normalized.operationalContract;
    this.#inputDescriptors = normalized.inputDescriptors;
    this.#outputDescriptors = normalized.outputDescriptors;
    this.#inputs = freezeObject(normalized.inputs);
    this.#outputs = freezeObject(normalized.outputs);
    this.#mutationContext = normalized.mutationContext;
    this.#invalidatedMessage = null;
    this.#rootSession.registerActiveImportDependent(this);
  }

  api() {
    const diagnostics = () => createRootDiagnosticsFacade(this.#rootSession);
    const history = () => createRootHistoryFacade(this.#rootSession);
    const specialist = () => createRootSpecialistFacade(this.#rootSession);
    const adapters = () => createRootAdaptersFacade(this.#rootSession);
    const signalsFacade = freezeObject({ diagnostics, history, specialist, adapters });
    const exportDefinition = () => buildGraphExportDefinition(
      this.#graphId,
      this.#graphSummary,
      this.#contract,
      this.#operationalContract,
      this.#inputDescriptors,
      this.#outputDescriptors,
      this.#context.runtimeDefinitionEnvelope,
    );
    return freezeObject({
      id: this.#graphId,
      inputs: this.#inputs,
      outputs: this.#outputs,
      contract: () => this.contract(),
      contractDelta: (previousContract) => buildGraphContractDelta(this.#contract, previousContract),
      contractHistory: () => this.#contractHistory,
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
      transaction: (callback) => this.transaction(callback),
      transactionAsync: (callback) => this.transactionAsync(callback),
      batchAsync: (callback) => this.batchAsync(callback),
      why: (name) => this.why(name),
      replayFor: (name) => this.replayFor(name),
      lineageFor: (name) => this.lineageFor(name),
      readVersions: () => this.readVersions(),
      summary: () => this.#graphSummary,
      inputDescriptors: () => this.#inputDescriptors,
      descriptors: () => this.#outputDescriptors,
      inspectDiagnostics: () => buildGraphDiagnosticsSurface(
        signalsFacade,
        this.#graphSummary,
        this.#inputDescriptors,
        this.#outputDescriptors,
        this.#context.runtimeDefinitionEnvelope,
      ),
      inspectHistory: () => buildGraphHistorySurface(
        signalsFacade,
        this.#graphSummary,
        this.#inputDescriptors,
        this.#outputDescriptors,
        this.#context.runtimeDefinitionEnvelope,
      ),
      exportCompatibilityDefinition: () => buildCompatibilityDefinition(
        this.#graphId,
        this.#graphSummary,
        this.#inputDescriptors,
        this.#outputDescriptors,
        this.#context.runtimeDefinitionEnvelope,
      ),
      exportDefinition,
      exportSnapshot: () => {
        const runtimeEnvelope = this.#context.requireRuntimeEnvelopeArtifact();
        return buildImportedGraphSnapshotArtifact({
          definition: exportDefinition(),
          runtimeEnvelope,
          snapshotEnvelope: this.#context.snapshotEnvelope,
          restoreMode: runtimeEnvelope.runtimeEnvelopeRestoreMode,
          contractHistory: this.#contractHistory,
          importPosture: this.importPosture(),
        });
      },
      diagnostics,
      history,
      specialist,
      adapters,
      compatibilityApp() {
        throwWorkerFirstGraphMutationUnavailable(graphId, "compatibilityApp");
      },
      compatibilityRuntime() {
        throwWorkerFirstGraphMutationUnavailable(graphId, "compatibilityRuntime");
      },
    });
  }

  invalidate(message) {
    if (this.#invalidatedMessage !== null) {
      return;
    }
    this.#invalidatedMessage = message;
  }

  contract() {
    this.#requireActive("contract");
    return this.#contract;
  }

  importPosture() {
    this.#requireActive("importPosture");
    return freezeObject({
      ...this.#context.snapshot.importPosture,
      graphId: this.#graphId,
      hydrate: "Applied",
      hydrateReason:
        "worker-first root graph aliases the active imported graph truth already admitted into the shared worker runtime",
    });
  }

  input(name) {
    this.#requireActive("input");
    return requireKnownEntry(this.#inputs, this.#graphId, name, "public input");
  }

  output(name) {
    this.#requireActive("output");
    return requireKnownEntry(this.#outputs, this.#graphId, name, "published output");
  }

  read() {
    this.#requireActive("read");
    const snapshot = Object.create(null);
    const context = this.#rootSession.currentImportContext();
    for (const descriptor of this.#outputDescriptors) {
      snapshot[descriptor.outputName] = context.signalValueById.get(descriptor.publishedId);
    }
    return freezeObject(snapshot);
  }

  readInputs() {
    this.#requireActive("readInputs");
    const snapshot = Object.create(null);
    const context = this.#rootSession.currentImportContext();
    for (const descriptor of this.#inputDescriptors) {
      snapshot[descriptor.inputName] = context.signalValueById.get(descriptor.sourceId);
    }
    return freezeObject(snapshot);
  }

  writeInputs(values) {
    this.#requireActive("writeInputs");
    return applyWorkerFirstRootGraphMutation(this.#rootSession, this.#graphId, this.#mutationContext, {
      writes: values,
    });
  }

  writeInput(name, value) {
    this.#requireActive("writeInput");
    return this.writeInputs({ [name]: value });
  }

  patchInputs(patches) {
    this.#requireActive("patchInputs");
    return applyWorkerFirstRootGraphMutation(this.#rootSession, this.#graphId, this.#mutationContext, {
      patches,
    });
  }

  patchInput(name, patch) {
    this.#requireActive("patchInput");
    return this.patchInputs({ [name]: patch });
  }

  resetInputs(names = this.#inputDescriptors.map((descriptor) => descriptor.inputName)) {
    this.#requireActive("resetInputs");
    return applyWorkerFirstRootGraphMutation(this.#rootSession, this.#graphId, this.#mutationContext, {
      reset: names,
    });
  }

  resetInput(name) {
    this.#requireActive("resetInput");
    return this.resetInputs([name]);
  }

  apply(mutation) {
    this.#requireActive("apply");
    return applyWorkerFirstRootGraphMutation(
      this.#rootSession,
      this.#graphId,
      this.#mutationContext,
      mutation,
    );
  }

  transaction(callback) {
    this.#requireActive("transaction");
    return runWorkerFirstRootGraphTransaction(
      this.#rootSession,
      this.#graphId,
      {
        ...this.#mutationContext,
        currentInputValues: this.#currentInputValues(),
      },
      callback,
      "transaction",
    );
  }

  transactionAsync(callback) {
    this.#requireActive("transactionAsync");
    return runWorkerFirstRootGraphTransaction(
      this.#rootSession,
      this.#graphId,
      {
        ...this.#mutationContext,
        currentInputValues: this.#currentInputValues(),
      },
      callback,
      "transactionAsync",
    );
  }

  batchAsync(callback) {
    this.#requireActive("batchAsync");
    return runWorkerFirstRootGraphTransaction(
      this.#rootSession,
      this.#graphId,
      {
        ...this.#mutationContext,
        currentInputValues: this.#currentInputValues(),
      },
      callback,
      "batchAsync",
    );
  }

  why(name) {
    this.#requireActive("why");
    return createRootDiagnosticsFacade(this.#rootSession).why(this.output(name).id);
  }

  replayFor(name) {
    this.#requireActive("replayFor");
    return createRootHistoryFacade(this.#rootSession).replay_for(this.output(name).id);
  }

  lineageFor(name) {
    this.#requireActive("lineageFor");
    return createRootHistoryFacade(this.#rootSession).lineage_for(this.output(name).id);
  }

  readVersions() {
    this.#requireActive("readVersions");
    return createRootSpecialistFacade(this.#rootSession).readVersions(
      this.#outputDescriptors.map((descriptor) => descriptor.publishedId),
    );
  }

  #currentInputValues() {
    const context = this.#rootSession.currentImportContext();
    const currentValues = new Map();
    for (const descriptor of this.#inputDescriptors) {
      currentValues.set(descriptor.sourceId, context.signalValueById.get(descriptor.sourceId));
    }
    return currentValues;
  }

  #requireActive(operation) {
    if (this.#invalidatedMessage !== null) {
      throw new TypeError(
        `worker-first root graph ${this.#graphId} ${operation}() cannot be used because ${this.#invalidatedMessage}`,
      );
    }
  }
}
function requireKnownEntry(record, graphId, name, family) {
  if (!(name in record)) {
    throw new TypeError(`signals.graph \`${graphId}\` does not expose ${family} \`${name}\``);
  }
  return record[name];
}

function throwWorkerFirstGraphMutationUnavailable(graphId, operation) {
  const error = new Error(
    `worker-first root graph ${graphId} ${operation}() is unavailable because this lane is still synchronous; use graph.transactionAsync(...) or deployment: "mainThreadCompatibility" instead`,
  );
  error.name = "WorkerFirstGraphMutationUnavailable";
  error.code = "workerFirstGraphMutationUnavailable";
  error.compatibilityRecovery = freezeObject({
    deployment: "mainThreadCompatibility",
    message:
      'Retry with deployment: "mainThreadCompatibility" for synchronous graph mutation lanes.',
  });
  throw error;
}
