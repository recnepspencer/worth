import { createWorkerRuntimeBridge } from "./bridge/worker_runtime_bridge.js";
import { createWorkerFirstHostCapabilities } from "./worker_first_host_capabilities.js";
import { createWorkerFirstRootHistoryLifecycle } from "./worker_first_root_history_lifecycle.js";
import { createWorkerFirstRootMutation } from "./worker_first_root_mutation.js";
import { createWorkerFirstRootObservationManager } from "./worker_first_root_observations.js";
import { createWorkerFirstResourceBranchLifecycle } from "./worker_first_resource_branch_lifecycle.js";
import { createWorkerFirstRootRuntimeReplacement } from "./worker_first_root_runtime_replacement.js";
import { createWorkerFirstRootAuthoredRuntime } from "./sessions/support/authored/worker_first_root_authored_runtime.js";
import { tipBranchId } from "./sessions/support/authored/worker_first_authored_tip_catalog.js";
import { buildActiveImportContext } from "./sessions/support/worker_first_root_import_context.js";
import { beginWorkerFirstRootExactImport } from "./worker_first_root_session_exact_import.js";
import { createWorkerFirstRootLiveDiagnostics } from "./worker_first_root_live_diagnostics.js";
import { resolveRootBridgeOptions } from "./worker_first_root_session_bridge_options.js";
import { createWorkerFirstRootSessionHistoryDelegates } from "./worker_first_root_session_history_delegates.js";
import {
  bootstrapWorkerFirstRootBridge,
  invalidateWorkerFirstActiveImport,
  refreshWorkerFirstRootSessionAfterHistoryMutation,
  refreshWorkerFirstRootSessionBranchCache,
  requireWorkerFirstControllerActive,
  requireWorkerFirstRootActive,
} from "./worker_first_root_session_lifecycle.js";
import { resolveWorkerFirstRootWhy } from "./worker_first_root_why.js";
import { assertWorkerFirstHandleOwnership } from "./worker_first_handle_ownership.js";

export function createWorkerFirstRootSession(options = {}) {
  return new WorkerFirstRootSession(options);
}

class WorkerFirstRootSession {
  #bridge;
  #bootstrap;
  #hostCapabilities;
  #observations;
  #activeImportController;
  #activeImportDependents;
  #activeImportContext;
  #importChain;
  #authoredRuntime;
  #historyLifecycle;
  #mutation;
  #runtimeReplacement;
  #resourceBranches;
  #liveDiagnostics;
  #cachedCurrentBranch;
  #cachedBranches;
  #terminated;
  #authoredSettleInvocations;

  constructor(options) {
    this.#authoredSettleInvocations = 0;
    this.#bridge = createWorkerRuntimeBridge(resolveRootBridgeOptions(options));
    this.#liveDiagnostics = createWorkerFirstRootLiveDiagnostics(this.#bridge);
    this.#activeImportController = null;
    this.#activeImportDependents = new Set();
    this.#activeImportContext = null;
    this.#importChain = Promise.resolve();
    this.#authoredRuntime = createWorkerFirstRootAuthoredRuntime(
      this.#bridge,
      () => this.#activeImportContext,
      (operation) => this.#requireActive(operation),
      this,
    );
    this.#observations = createWorkerFirstRootObservationManager({
      hasAuthoredSignal: (id) => this.#authoredRuntime.hasAuthoredSignalId(id),
      readAuthoredSignal: (id) => this.#authoredRuntime.readSignalValue(id),
    });
    this.#historyLifecycle = createWorkerFirstRootHistoryLifecycle({
      ready: () => this.ready(),
      requireActive: (operation) => this.#requireActive(operation),
      requireControllerActive: (controller, operation) => this.#requireControllerActive(controller, operation),
      bridge: this.#bridge,
      observations: this.#observations,
      authoredRuntime: this.#authoredRuntime,
      settlePendingMutations: () => this.#mutation.settlePendingMutations(),
      activeImportContext: () => this.#activeImportContext,
      activeImportController: () => this.#activeImportController,
      setActiveImportContext: (context) => { this.#activeImportContext = context; },
      refreshBranchCache: () => this.#refreshBranchCache(),
      refreshActiveImportContext: () => this.refreshActiveImportContext(),
      refreshAfterHistoryMutation: (operation, activeImportContext) => this.#refreshAfterHistoryMutation(operation, activeImportContext),
      readCachedCurrentBranch: () => this.#cachedCurrentBranch,
      publishDiagnosticsChanged: () => this.publishDiagnosticsChanged(),
    });
    this.#mutation = createWorkerFirstRootMutation({
      ready: () => this.ready(),
      requireActive: (operation) => this.#requireActive(operation),
      requireControllerActive: (controller, operation) => this.#requireControllerActive(controller, operation),
      bridge: this.#bridge,
      observations: this.#observations,
      authoredRuntime: this.#authoredRuntime,
      activeImportContext: () => this.#activeImportContext,
      activeImportController: () => this.#activeImportController,
      setActiveImportContext: (context) => { this.#activeImportContext = context; },
      refreshBranchCache: () => this.#refreshBranchCache(),
      currentImportContext: () => this.currentImportContext(),
      hasMutableInputId: (id) => this.hasMutableInputId(id),
      applyImportMutation: (controller, transactionOps, outputIds) => this.applyImportMutation(controller, transactionOps, outputIds),
      publishDiagnosticsChanged: () => this.publishDiagnosticsChanged(),
    });
    this.#resourceBranches = createWorkerFirstResourceBranchLifecycle({
      ready: () => this.ready(),
      requireActive: (operation) => this.#requireActive(operation),
      settlePendingPublications: () => this.#authoredRuntime.settlePendingPublications(),
      settlePendingMutations: () => this.#mutation.settlePendingMutations(),
      bridge: this.#bridge,
      refreshBranchCache: () => this.#refreshBranchCache(),
      readCurrentTipBranchId: () => tipBranchId(this.#cachedCurrentBranch),
      markActiveTipCatalogChanged: () => this.#authoredRuntime.markActiveTipCatalogChanged(),
      readmitReadyAuthoredOntoActiveTip: () => (
        this.#authoredRuntime.readmitReadyAuthoredOntoActiveTip()
      ),
    });
    this.#cachedCurrentBranch = null;
    this.#cachedBranches = [];
    this.#hostCapabilities = createWorkerFirstHostCapabilities(this, options.hostCapabilities ?? null);
    this.#runtimeReplacement = createWorkerFirstRootRuntimeReplacement({
      ready: () => this.ready(),
      requireActive: (operation) => this.#requireActive(operation),
      bridge: this.#bridge,
      observations: this.#observations,
      authoredRuntime: this.#authoredRuntime,
      hostCapabilities: this.#hostCapabilities,
      invalidateActiveImport: (message) => this.#invalidateActiveImport(message),
      publishDiagnosticsChanged: () => this.publishDiagnosticsChanged(),
    });
    this.#bootstrap = this.#bootstrapBridge();
    this.#terminated = false;
    Object.assign(
      this,
      createWorkerFirstRootSessionHistoryDelegates({
        historyLifecycle: this.#historyLifecycle,
        resourceBranches: this.#resourceBranches,
        runtimeReplacement: this.#runtimeReplacement,
      }),
    );
  }

  bridge() { return this.#bridge; }
  peekLiveRootDiagnostics() { return this.#liveDiagnostics.peek(); }
  subscribeDiagnostics(listener) {
    this.#requireActive("diagnostics.subscribe");
    return this.#liveDiagnostics.subscribe(listener);
  }
  requireActiveDiagnostics(operation) {
    this.#requireActive(operation);
  }
  async publishDiagnosticsChanged() {
    if (this.#activeImportContext === null) {
      await this.#liveDiagnostics.refresh();
      return;
    }
    this.#liveDiagnostics.notify();
  }
  why(id) {
    this.#requireActive("diagnostics.why");
    return resolveWorkerFirstRootWhy({
      id,
      activeImportContext: this.#activeImportContext,
      authoredRuntime: this.#authoredRuntime,
      bridge: this.#bridge,
    });
  }
  ready() { return this.#bootstrap; }
  async settleAuthoredPublications() {
    await this.ready();
    await this.#authoredRuntime.settlePendingPublications();
  }

  async settleAuthoredWork() {
    this.#authoredSettleInvocations += 1;
    await this.ready();
    await this.#authoredRuntime.settlePendingPublications();
    await this.#mutation.settlePendingMutations();
  }
  authoredSettleInvocationCount() { return this.#authoredSettleInvocations; }
  hostSurface() { return this.#hostCapabilities.host; }
  latestHostCapabilityEvent() { return this.#hostCapabilities.latestEvent(); }
  recentHostCapabilityEvents() { return this.#hostCapabilities.recentEvents(); }
  hostCapabilityReport() { return this.#hostCapabilities.report(this.#authoredRuntime.hostDependencyReport()); }
  watch(target, callback) {
    assertWorkerFirstHandleOwnership(this, target, "worker-first root watch(...)");
    return this.#observations.watch(this.#bridge, target, callback);
  }
  effect(target, callback) {
    assertWorkerFirstHandleOwnership(this, target, "worker-first root effect(...)");
    return this.#observations.effect(this.#bridge, target, callback);
  }
  nuke(handle) { return this.#observations.nuke(this.#bridge, handle); }

  beginExactImport(definition, snapshot, controller) {
    return beginWorkerFirstRootExactImport(this.#exactImportDeps(), definition, snapshot, controller);
  }
  #exactImportDeps() {
    return {
      bridge: this.#bridge, authoredRuntime: this.#authoredRuntime, observations: this.#observations,
      hostCapabilities: this.#hostCapabilities, importChain: this.#importChain,
      ready: () => this.ready(),
      requireActive: (operation) => this.#requireActive(operation),
      requireControllerActive: (c, operation) => this.#requireControllerActive(c, operation),
      invalidateActiveImport: (message) => this.#invalidateActiveImport(message),
      setActiveImportController: (c) => { this.#activeImportController = c; },
      setActiveImportContext: (context) => { this.#activeImportContext = context; },
      setImportChain: (chain) => { this.#importChain = chain; },
      publishDiagnosticsChanged: () => this.publishDiagnosticsChanged(),
    };
  }

  detachImport(controller) {
    if (this.#activeImportController === controller) this.#activeImportController = null;
  }

  registerActiveImportDependent(dependent) { this.#activeImportDependents.add(dependent); }
  unregisterActiveImportDependent(dependent) { this.#activeImportDependents.delete(dependent); }

  currentImportContext() {
    if (this.#activeImportContext === null) {
      throw new TypeError(
        "worker-first root surface requires an active imported graph; await importedGraph.ready() first",
      );
    }
    return this.#activeImportContext;
  }

  /** Null when no importGraph has been admitted yet (empty worker-first roots). */
  peekActiveImportContext() {
    return this.#activeImportContext;
  }

  nextGeneratedStandaloneSignalId(family, scopeId = null) {
    return this.#authoredRuntime.nextGeneratedStandaloneSignalId(family, scopeId);
  }

  hasKnownSignalId(id) {
    return this.#authoredRuntime.hasKnownSignalId(id);
  }

  hasMutableInputId(id) {
    return this.#authoredRuntime.hasMutableInputId(id);
  }

  readSignalValue(id) {
    return this.#authoredRuntime.readSignalValue(id);
  }

  readAuthoredInputBaseline(id) {
    return this.#authoredRuntime.readAuthoredInputBaseline(id);
  }

  currentBranchSummary() {
    return this.#activeImportContext?.currentBranch ?? this.#cachedCurrentBranch;
  }

  branchesSummary() {
    return this.#activeImportContext?.branches ?? this.#cachedBranches;
  }

  writeAuthoredInputBaseline(id, value) {
    this.#authoredRuntime.writeAuthoredInputBaseline(id, value);
  }

  async createStandaloneInput(id, initial, options = {}) {
    await this.ready(); await this.#authoredRuntime.createStandaloneInput(id, initial, options);
  }
  createEagerStandaloneInput(id, initial, options = {}) {
    this.#authoredRuntime.createEagerStandaloneInput(id, initial, options);
  }
  async createStandaloneReadable(id, family, spec) {
    await this.ready(); await this.#authoredRuntime.createStandaloneReadable(id, family, spec);
  }
  createEagerStandaloneReadable(id, family, spec, initialValue, dependencyIds) {
    this.#authoredRuntime.createEagerStandaloneReadable(
      id,
      family,
      spec,
      initialValue,
      dependencyIds,
    );
  }
  async createStandaloneCallbackReadable(id, family, callback) {
    await this.ready(); await this.#authoredRuntime.createStandaloneCallbackReadable(id, family, callback);
  }
  createEagerStandaloneCallbackReadable(id, family, callback) {
    this.#authoredRuntime.createEagerStandaloneCallbackReadable(id, family, callback);
  }

  async terminate() {
    if (this.#terminated) {
      return;
    }
    await this.#authoredRuntime.settlePendingPublications();
    await this.#mutation.settlePendingMutations();
    this.#terminated = true;
    this.#hostCapabilities.dispose();
    this.#invalidateActiveImport("worker-first root session terminated");
    this.#authoredRuntime.invalidate("worker-first root session terminated");
    this.#activeImportContext = null;
    this.#liveDiagnostics.reset();
    await this.#observations.clearContext(this.#bridge);
    await this.#observations.clearObservers(this.#bridge);
    await this.#bridge.terminate();
  }

  async refreshActiveImportContext() {
    if (this.#activeImportController === null || this.#activeImportContext === null) {
      return;
    }
    const activeImportController = this.#activeImportController;
    const activeImportContext = this.#activeImportContext;
    this.#requireActive("refreshActiveImportContext");
    this.#activeImportContext = await buildActiveImportContext(
      this.#bridge,
      activeImportContext.definition,
      activeImportContext.snapshot,
    );
    await this.#observations.replaceContext(this.#bridge, this.#activeImportContext);
    await this.publishDiagnosticsChanged();
    if (typeof activeImportController.refreshFromRootRuntime === "function") {
      await activeImportController.refreshFromRootRuntime();
    }
    this.#requireControllerActive(activeImportController, "refreshActiveImportContext");
  }

  async applyImportMutation(controller, transactionOps, outputIds) {
    return this.#mutation.applyImportMutation(controller, transactionOps, outputIds);
  }
  async applyActiveTransaction(transactionOps) { return this.#mutation.applyActiveTransaction(transactionOps); }
  async applyActiveInputMutation(id, mutation) { return this.#mutation.applyActiveInputMutation(id, mutation); }
  async refreshHostCapabilityReadables(hostDependencyIds) { return this.#authoredRuntime.refreshReadables([], hostDependencyIds); }

  applyAuthoredInputMutation(id, mutation) { return this.#mutation.applyAuthoredInputMutation(id, mutation); }
  projectAuthoredInputTip(id, value) { return this.#authoredRuntime.projectAuthoredInputTip(id, value); }
  commitHostTipAndNotify(tipWrites) {
    return this.#authoredRuntime.commitHostTipAndNotify(this.#observations, () => this.#activeImportContext, tipWrites);
  }
  publishAuthoredTipProjection(changedIds) { this.#authoredRuntime.notifyHostTipIds(this.#observations, changedIds); }
  applyCommittedTipWorkerBatch(tipWrites) { return this.#mutation.applyCommittedTipWorkerBatch(tipWrites); }

  #invalidateActiveImport(message) {
    const state = {
      activeImportController: this.#activeImportController,
      activeImportDependents: this.#activeImportDependents,
      activeImportContext: this.#activeImportContext,
    };
    invalidateWorkerFirstActiveImport(state, message);
    this.#activeImportController = state.activeImportController;
    this.#activeImportContext = state.activeImportContext;
  }

  async #bootstrapBridge() {
    await bootstrapWorkerFirstRootBridge({
      bridge: this.#bridge,
      hostCapabilities: this.#hostCapabilities,
      refreshBranchCache: () => this.#refreshBranchCache(),
    });
  }

  #requireActive(operation) {
    requireWorkerFirstRootActive(this.#terminated, operation);
  }

  #requireControllerActive(controller, operation) {
    requireWorkerFirstControllerActive(controller, operation);
  }

  async #refreshAfterHistoryMutation(operation, activeImportContext) {
    await refreshWorkerFirstRootSessionAfterHistoryMutation(
      {
        bridge: this.#bridge,
        observations: this.#observations,
        authoredRuntime: this.#authoredRuntime,
        activeImportContext: () => this.#activeImportContext,
        activeImportController: () => this.#activeImportController,
        setActiveImportContext: (context) => {
          this.#activeImportContext = context;
        },
        refreshBranchCache: () => this.#refreshBranchCache(),
        readCachedCurrentBranch: () => this.#cachedCurrentBranch,
        requireControllerActive: (controller, action) => this.#requireControllerActive(controller, action),
        publishDiagnosticsChanged: () => this.publishDiagnosticsChanged(),
      },
      operation,
      activeImportContext,
    );
  }

  async #refreshBranchCache() {
    await refreshWorkerFirstRootSessionBranchCache(this.#bridge, (currentBranch, branches) => {
      this.#cachedCurrentBranch = currentBranch;
      this.#cachedBranches = branches;
    });
  }
}
