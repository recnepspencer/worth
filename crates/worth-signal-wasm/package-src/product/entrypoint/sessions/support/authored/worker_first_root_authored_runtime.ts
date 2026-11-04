import {
  createAuthoredInputPublication,
  createWorkerFirstAuthoredInputState,
  hasMutableWorkerFirstAuthoredInputId,
  readWorkerFirstAuthoredInputBaseline,
  invalidateWorkerFirstAuthoredInputs,
  nextGeneratedStandaloneSignalId,
  readWorkerFirstAuthoredInputValue,
} from "./worker_first_authored_input_state.js";
import {
  applyCommittedWorkerFirstAuthoredInputOps,
  beginWorkerFirstAuthoredInputMutation,
  requireWorkerFirstAuthoredInputPublicationReady,
  writeWorkerFirstAuthoredInputBaselineValue,
} from "./worker_first_authored_input_mutation.js";
import {
  createAuthoredReadablePublication,
  hasWorkerFirstAuthoredReadableId,
  invalidateWorkerFirstAuthoredReadables,
  readWorkerFirstAuthoredReadableValue,
} from "./worker_first_authored_readable_state.js";
import { assertAuthoredReadableReads } from "./worker_first_authored_readable_reads.js";
import { createHostRefreshCurrentness } from "./worker_first_host_refresh_currentness.js";
import {
  captureWorkerFirstAuthoredCallback,
} from "./worker_first_authored_callback_authoring.js";
import { buildWorkerFirstHostDependencyReport } from "./worker_first_host_dependency_report.js";
import {
  refreshWorkerFirstAuthoredReadables,
  refreshWorkerFirstAuthoredCallbackReadables,
  refreshWorkerFirstAuthoredReadableSignals,
} from "./worker_first_authored_readable_refresh.js";
import {
  projectAuthoredInputTip,
  refreshAuthoredCallbackTipsFromHost,
} from "./worker_first_authored_tip_projection.js";
import {
  commitHostTipAndNotify,
  notifyHostTipIds,
} from "./worker_first_host_tip_commit.js";
import {
  createAuthoredPublicationTracker,
  markAuthoredPublicationFailed,
  markAuthoredPublicationReady,
} from "./worker_first_authored_publication_tracking.js";
import {
  createEagerStandaloneAuthoredCallbackReadable,
  createEagerStandaloneAuthoredInput,
  createEagerStandaloneAuthoredReadable,
  createStandaloneAuthoredCallbackReadable,
  createStandaloneAuthoredInput,
  createStandaloneAuthoredReadable,
} from "./worker_first_authored_signal_creation.js";
import {
  createAuthoredTipCatalogAdmission,
  ensureAuthoredInputsPresentOnWorkerTip,
  readmitStaleAuthoredOntoActiveTip,
} from "./worker_first_authored_tip_catalog.js";
import { outputProjectionSpec } from "../../../../output_projection_ids.js";
import { pendingWorkerInputIds } from "./worker_first_authored_readback_currentness.js";

export function createWorkerFirstRootAuthoredRuntime(
  bridge,
  activeImportContext,
  requireActive,
  runtimeMarker,
) {
  return new WorkerFirstRootAuthoredRuntime(
    bridge,
    activeImportContext,
    requireActive,
    runtimeMarker,
  );
}

class WorkerFirstRootAuthoredRuntime {
  #hostRefreshes = createHostRefreshCurrentness();
  #bridge;
  #activeImportContext;
  #requireActive;
  #runtimeMarker;
  #authoredInputs;
  #authoredReadables;
  #authoredCallbacks;
  #generatedStandaloneSignalCounters;
  #publications;
  #tipCatalog;
  #creationDeps;

  constructor(bridge, activeImportContext, requireActive, runtimeMarker) {
    this.#bridge = bridge;
    this.#activeImportContext = activeImportContext;
    this.#requireActive = requireActive;
    this.#runtimeMarker = runtimeMarker;
    this.#authoredInputs = new Map();
    this.#authoredReadables = new Map();
    this.#authoredCallbacks = new Map();
    this.#generatedStandaloneSignalCounters = new Map();
    this.#publications = createAuthoredPublicationTracker();
    this.#tipCatalog = createAuthoredTipCatalogAdmission();
    this.#creationDeps = {
      bridge: this.#bridge,
      authoredInputs: this.#authoredInputs,
      authoredReadables: this.#authoredReadables,
      authoredCallbacks: this.#authoredCallbacks,
      generatedStandaloneSignalCounters: this.#generatedStandaloneSignalCounters,
      requireActive: (operation) => this.#requireActive(operation),
      assertUnusedId: (id, operation) => this.#assertUnusedId(id, operation),
      assertSupportedReadableSpec: (family, spec) => this.#assertSupportedReadableSpec(family, spec),
      captureCallback: (callback, family) => this.#captureCallback(callback, family),
      trackEagerPublication: (ids, publication, failureMessage) => {
        this.#trackEagerPublication(ids, publication, failureMessage);
      },
      publishAuthoredInput: (id, initial, options) => this.#publishAuthoredInput(id, initial, options),
      publishCallbackReadableGraph: (id, family, hiddenInputId, initialValue) => (
        this.#publishCallbackReadableGraph(id, family, hiddenInputId, initialValue)
      ),
      hasKnownSignalId: (id) => this.hasKnownSignalId(id),
      currentTipEpoch: () => this.#tipCatalog.currentEpoch(),
      stampAdmittedIfEpoch: (state, publishEpoch) => (
        this.#tipCatalog.stampAdmittedIfEpoch(state, publishEpoch)
      ),
    };
  }

  nextGeneratedStandaloneSignalId(family, scopeId = null) {
    return nextGeneratedStandaloneSignalId(this.#generatedStandaloneSignalCounters, family, scopeId);
  }

  hasKnownSignalId(id) {
    return this.#authoredInputs.has(id)
      || hasWorkerFirstAuthoredReadableId(this.#authoredReadables, id)
      || this.#activeImportContext()?.signalValueById.has(id) === true;
  }

  hasMutableInputId(id) {
    return this.#activeImportContext()?.inputDescriptorBySourceId.has(id) === true
      || hasMutableWorkerFirstAuthoredInputId(this.#authoredInputs, id);
  }

  isAuthoredInputPublicationReady(id) {
    return isWorkerFirstAuthoredInputPublicationReady(this.#authoredInputs, id);
  }

  readSignalValue(id) {
    if (this.#authoredInputs.has(id)) {
      return readWorkerFirstAuthoredInputValue(this.#authoredInputs, id);
    }
    if (this.#authoredReadables.has(id)) {
      return readWorkerFirstAuthoredReadableValue(this.#authoredReadables, id);
    }
    const context = this.#activeImportContext();
    if (context?.signalValueById.has(id)) {
      return context.signalValueById.get(id);
    }
    throw new TypeError(
      `worker-first root read(${JSON.stringify(id)}) requires a currently available worker-first signal`,
    );
  }

  readAuthoredInputBaseline(id) {
    return readWorkerFirstAuthoredInputBaseline(this.#authoredInputs, id);
  }

  createStandaloneInput(id, initial, options = {}) {
    return createStandaloneAuthoredInput(this.#creationDeps, id, initial, options);
  }

  createEagerStandaloneInput(id, initial, options = {}) {
    createEagerStandaloneAuthoredInput(this.#creationDeps, id, initial, options);
  }

  createStandaloneReadable(id, family, spec) {
    return createStandaloneAuthoredReadable(this.#creationDeps, id, family, spec);
  }

  createEagerStandaloneReadable(id, family, spec, initialValue, dependencyIds) {
    createEagerStandaloneAuthoredReadable(
      this.#creationDeps,
      id,
      family,
      spec,
      initialValue,
      dependencyIds,
    );
  }

  createStandaloneCallbackReadable(id, family, callback) {
    return createStandaloneAuthoredCallbackReadable(this.#creationDeps, id, family, callback);
  }

  createEagerStandaloneCallbackReadable(id, family, callback) {
    createEagerStandaloneAuthoredCallbackReadable(this.#creationDeps, id, family, callback);
  }

  beginAuthoredInputMutation(id, mutation) {
    return beginWorkerFirstAuthoredInputMutation(this.#authoredInputs, id, mutation);
  }

  /** Advance authored input tip without enqueueing worker apply. */
  projectAuthoredInputTip(id, value) {
    return projectAuthoredInputTip(this.#authoredInputs, id, value);
  }

  /** Host-only tip recompute for callback readables that depend on changed ids. */
  refreshAuthoredCallbackTipsFromHost(changedIds = [], changedHostDependencyIds = []) {
    return refreshAuthoredCallbackTipsFromHost({
      authoredReadables: this.#authoredReadables,
      authoredCallbacks: this.#authoredCallbacks,
      changedIds,
      changedHostDependencyIds,
      readSignalValue: (id) => this.readSignalValue(id),
      captureCallback: (callback, family) => this.#captureCallback(callback, family),
    });
  }

  /** Unified tip ingress: write tips + project dependents + notify observers. */
  commitHostTipAndNotify(observations, activeImportContext, tipWrites) {
    return commitHostTipAndNotify({
      authoredInputs: this.#authoredInputs,
      activeImportContext,
      refreshAuthoredCallbackTipsFromHost: (ids) => this.refreshAuthoredCallbackTipsFromHost(ids),
      observations,
    }, tipWrites);
  }

  notifyHostTipIds(observations, changedIds) {
    return notifyHostTipIds({
      refreshAuthoredCallbackTipsFromHost: (ids) => this.refreshAuthoredCallbackTipsFromHost(ids),
      observations,
    }, changedIds);
  }

  requireAuthoredInputPublicationReady(id) {
    requireWorkerFirstAuthoredInputPublicationReady(this.#authoredInputs, id);
  }

  applyCommittedInputs(transactionOps) {
    applyCommittedWorkerFirstAuthoredInputOps(this.#authoredInputs, transactionOps);
  }

  writeAuthoredInputBaseline(id, value) {
    writeWorkerFirstAuthoredInputBaselineValue(this.#authoredInputs, id, value);
  }

  acknowledgementReadbackIds() {
    return [...this.#authoredReadables].filter(([, state]) =>
      state.invalidatedMessage === null && state.publicationState !== "pending").map(([id]) => id);
  }
  hostDependencyReadback() { return this.#hostRefreshes; }

  async refreshReadables(changedIds = [], changedHostDependencyIds = [], bridge = this.#bridge) {
    await refreshWorkerFirstAuthoredReadables({
      hostRefreshes: this.#hostRefreshes,
      pendingInputIds: () => pendingWorkerInputIds(this.#authoredInputs, this.#activeImportContext()),
      bridge, authoredInputs: this.#authoredInputs, authoredReadables: this.#authoredReadables,
      authoredCallbacks: this.#authoredCallbacks, changedIds, changedHostDependencyIds,
      captureCallback: (callback, family) => this.#captureCallback(callback, family),
      awaitPublication: (id) => this.#publications.awaitPublication(id),
    });
  }

  hostDependencyReport() {
    return buildWorkerFirstHostDependencyReport(this.#authoredCallbacks);
  }

  invalidate(message) {
    invalidateWorkerFirstAuthoredInputs(this.#authoredInputs, message);
    invalidateWorkerFirstAuthoredReadables(this.#authoredReadables, message);
  }

  async settlePendingPublications() {
    await this.#publications.settlePendingPublications();
  }

  async awaitPublication(id) {
    await this.#publications.awaitPublication(id);
  }

  /**
   * Tip-changing branch/history ops must call this so ordinary set()/apply does
   * not pay a speculative readSignals probe on every mutation.
   */
  markActiveTipCatalogChanged() {
    this.#tipCatalog.markActiveTipCatalogChanged();
  }

  async readmitReadyAuthoredOntoActiveTip() {
    await readmitStaleAuthoredOntoActiveTip({
      bridge: this.#bridge,
      authoredInputs: this.#authoredInputs,
      authoredReadables: this.#authoredReadables,
      tipCatalog: this.#tipCatalog,
    });
  }

  /**
   * Epoch-gated recovery: no bridge work when the id is already stamped for the
   * active tip. Used as a race safety net if apply runs before tip readmit.
   */
  async ensureAuthoredInputsPresentOnWorker(ids) {
    await ensureAuthoredInputsPresentOnWorkerTip({
      bridge: this.#bridge,
      authoredInputs: this.#authoredInputs,
      tipCatalog: this.#tipCatalog,
    }, ids);
  }

  hasAuthoredSignalId(id) {
    return this.#authoredInputs.has(id)
      || hasWorkerFirstAuthoredReadableId(this.#authoredReadables, id);
  }

  async refreshAllReadables() {
    const refreshDeps = {
      hostRefreshes: this.#hostRefreshes,
      pendingInputIds: () => pendingWorkerInputIds(this.#authoredInputs, this.#activeImportContext()),
      bridge: this.#bridge,
      authoredReadables: this.#authoredReadables,
      awaitPublication: (id) => this.#publications.awaitPublication(id),
    };
    await refreshWorkerFirstAuthoredReadableSignals(refreshDeps);
    await refreshWorkerFirstAuthoredCallbackReadables({
      hostRefreshes: this.#hostRefreshes,
      bridge: this.#bridge,
      authoredInputs: this.#authoredInputs,
      authoredReadables: this.#authoredReadables,
      authoredCallbacks: this.#authoredCallbacks,
      changedIds: null,
      pendingInputIds: () => pendingWorkerInputIds(this.#authoredInputs, this.#activeImportContext()),
      changedHostDependencyIds: null,
      captureCallback: (callback, family) => this.#captureCallback(callback, family),
      skipDirectReadableRefresh: true,
      awaitPublication: (id) => this.#publications.awaitPublication(id),
    });
    await refreshWorkerFirstAuthoredReadableSignals(refreshDeps);
  }

  #assertUnusedId(id, operation) {
    if (
      this.#authoredInputs.has(id)
      || this.#authoredReadables.has(id)
      || this.#activeImportContext()?.signalValueById.has(id)
    ) {
      throw new TypeError(
        `worker-first ${operation}(...) cannot reuse canonical id \`${id}\` in the same worker-owned runtime`,
      );
    }
  }

  async #publishAuthoredInput(id, initial, options) {
    const publishEpoch = this.#tipCatalog.currentEpoch();
    await this.#bridge.publishPortableGraph(createAuthoredInputPublication(id, initial, options));
    const state = createWorkerFirstAuthoredInputState(initial, "ready", options);
    this.#tipCatalog.stampAdmittedIfEpoch(state, publishEpoch);
    this.#authoredInputs.set(id, state);
  }

  async #publishCallbackReadableGraph(id, family, hiddenInputId, initialValue) {
    const publishEpoch = this.#tipCatalog.currentEpoch();
    await this.#bridge.publishPortableGraph(createAuthoredInputPublication(hiddenInputId, initialValue, {}));
    this.#tipCatalog.stampAdmittedIfEpoch(this.#authoredInputs.get(hiddenInputId), publishEpoch);
    await this.#bridge.publishPortableGraph(
      createAuthoredReadablePublication(id, family, outputProjectionSpec(hiddenInputId)),
    );
    this.#tipCatalog.stampAdmittedIfEpoch(this.#authoredReadables.get(id), publishEpoch);
  }

  #assertSupportedReadableSpec(family, spec) {
    assertAuthoredReadableReads(family, spec?.reads, (id) => this.hasKnownSignalId(id));
  }

  #captureCallback(callback, family) {
    return captureWorkerFirstAuthoredCallback(
      this.#runtimeMarker,
      callback,
      family,
      (id) => this.hasKnownSignalId(id),
    );
  }

  #trackEagerPublication(ids, publication, failureMessage) {
    const publishEpoch = this.#tipCatalog.currentEpoch();
    this.#publications.trackPendingPublication(ids, publication, (outcome) => {
      for (const id of ids) {
        const input = this.#authoredInputs.get(id);
        const readable = this.#authoredReadables.get(id);
        if (outcome === "ready") {
          markAuthoredPublicationReady(input);
          markAuthoredPublicationReady(readable);
          this.#tipCatalog.stampAdmittedIfEpoch(input, publishEpoch);
          this.#tipCatalog.stampAdmittedIfEpoch(readable, publishEpoch);
        } else {
          markAuthoredPublicationFailed(input, failureMessage);
          markAuthoredPublicationFailed(readable, failureMessage);
        }
      }
    });
  }
}
