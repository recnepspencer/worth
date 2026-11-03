import { createApiFactory, createApiScopeFactory } from "../api/api_namespace.js";
import {
  forbidOpaqueIdOption,
  requireAuthoringOptions,
  requireOptionalDebugName,
} from "../authoring_option_validation.js";
import { freezeObject } from "../graph_support.js";
import { PRIVATE_AUTHORING_ID } from "../symbols.js";
import { createWorkerFirstScopedNamespace } from "./worker_first_authoring_namespace.js";
import { createWorkerFirstAsyncInputHandle } from "./worker_first_async_input.js";
import { createWorkerFirstAsyncLinkedHandle } from "./worker_first_async_linked.js";
import {
  createWorkerFirstAsyncRecipeHandle,
  normalizeWorkerFirstAsyncRecipeOptions,
} from "./worker_first_async_recipe.js";
import { runWorkerFirstAsyncTransaction } from "./worker_first_async_transaction.js";
import { createWorkerFirstFormFactory } from "./worker_first_form_factory.js";
import { createFeatureStoreFactory } from "../feature_store/feature_store_factory.js";
import { createLocalNamespace } from "../local/local_namespace.js";
import { createWorkerLocalTruthFactory } from "../local_truth/protocol/worker_local_truth_proxy.js";
import { createWorkerFirstResourceNamespace } from "./worker_first_resource_namespace.js";
import { createWorkerFirstExplicitSpecNamespace } from "./worker_first_explicit_spec_namespace.js";
import { createRouterNamespace } from "../router/router_namespace.js";
import {
  assertSignalsRuntimeCompatibility,
  createSignalsRuntimeContract,
} from "../runtime_contract.js";
import { createWorkerFirstRootImportedGraph } from "./worker_first_root_imported_graph.js";
import { readRootHistoryFacade } from "./worker_first_root_history.js";
import { createWorkerFirstRootSession } from "./worker_first_root_session.js";
import { createWorkerFirstRootTermination } from "./worker_first_root_termination.js";
import {
  createRootAdaptersFacade,
  createRootDiagnosticsFacade,
  createRootSpecialistFacade,
  readRootSignalValue,
} from "./worker_first_root_cached_facades.js";
import {
  createWorkerFirstSyncComputedCallbackHandle,
  createWorkerFirstSyncInputHandle,
  createWorkerFirstSyncLinkedHandle,
  createWorkerFirstSyncOutputCallbackHandle,
  createWorkerFirstSyncRecipeHandle,
} from "./worker_first_sync_authoring.js";

export function createWorkerFirstCallableSignals(request) {
  const rootSession = createWorkerFirstRootSession(request);
  return createWorkerFirstCallableSignalsAfterBootstrap(rootSession, request);
}

async function createWorkerFirstCallableSignalsAfterBootstrap(rootSession, request) {
  await rootSession.ready();
  const trackedGraphs = new Set();
  const terminate = createWorkerFirstRootTermination(rootSession, trackedGraphs);
  let diagnostics = null;
  let history = null;
  let adapters = null;
  let specialist = null;
  let form = null;
  let resource = null;
  let api = null;
  let apiScope = null;
  let featureStore = null;
  let local = null;
  let localTruth = null;
  let router = null;
  let spec = null;
  let rootNamespace = null;
  const contract = createSignalsRuntimeContract({
    surfaceFamily: "workerFirstCallable",
    deployment: "workerFirst",
    capabilities: {
      callableSurface: true,
      scopedAuthoring: true,
      specNamespace: true,
      workerRuntime: true,
    },
  });
  void request;

  const namespace = () => {
    rootNamespace ??= createWorkerFirstScopedNamespace(rootSession);
    return rootNamespace;
  };

  const callableSignals = {
    host: rootSession.hostSurface(),
    get spec() {
      spec ??= createWorkerFirstExplicitSpecNamespace(rootSession);
      return spec;
    },
    get form() {
      form ??= createWorkerFirstFormFactory(callableSignals);
      return form;
    },
    get resource() {
      resource ??= createWorkerFirstResourceNamespace(callableSignals, rootSession);
      return resource;
    },
    get api() {
      api ??= createApiFactory(callableSignals);
      return api;
    },
    get apiScope() {
      apiScope ??= createApiScopeFactory(callableSignals);
      return apiScope;
    },
    get featureStore() {
      featureStore ??= createFeatureStoreFactory(callableSignals);
      return featureStore;
    },
    get local() {
      local ??= createLocalNamespace(callableSignals);
      return local;
    },
    get localTruth() {
      localTruth ??= createWorkerLocalTruthFactory(rootSession);
      return localTruth;
    },
    get router() {
      router ??= createRouterNamespace();
      return router;
    },
    scope(localScopeId) {
      return namespace().scope(localScopeId);
    },
    settleAuthoredWork() {
      return rootSession.settleAuthoredWork();
    },
    authoredSettleInvocationCount() {
      return rootSession.authoredSettleInvocationCount();
    },
    commitHostTipAndNotify(tipWrites) {
      return rootSession.commitHostTipAndNotify(tipWrites);
    },
    applyCommittedTipWorkerBatch(tipWrites) {
      return rootSession.applyCommittedTipWorkerBatch(tipWrites);
    },
    publishAuthoredTipProjection(changedIds) {
      return rootSession.publishAuthoredTipProjection(changedIds);
    },
    controller(definitionOrBuilder) {
      return namespace().controller(definitionOrBuilder);
    },
    publicInput(handle, options) {
      return namespace().publicInput(handle, options);
    },
    input(initial, options) {
      const normalizedOptions = normalizeWorkerFirstInputOptions(options);
      const id =
        normalizedOptions?.[PRIVATE_AUTHORING_ID]
        ?? rootSession.nextGeneratedStandaloneSignalId("input");
      return createWorkerFirstSyncInputHandle(rootSession, id, initial, normalizedOptions);
    },
    async inputAsync(initial, options) {
      const normalizedOptions = normalizeWorkerFirstInputOptions(options);
      const id =
        normalizedOptions?.[PRIVATE_AUTHORING_ID]
        ?? rootSession.nextGeneratedStandaloneSignalId("input");
      await rootSession.createStandaloneInput(id, initial, normalizedOptions);
      return createWorkerFirstAsyncInputHandle(
        rootSession,
        id,
        normalizedOptions
          ? requireOptionalDebugName("input", normalizedOptions)
          : null,
      );
    },
    linked(sourceOrDefinition, options) {
      return createWorkerFirstSyncLinkedHandle(
        rootSession,
        rootSession.nextGeneratedStandaloneSignalId("input"),
        sourceOrDefinition,
        options,
      );
    },
    async linkedAsync(sourceOrDefinition, options) {
      return createWorkerFirstAsyncLinkedHandle(
        rootSession,
        rootSession.nextGeneratedStandaloneSignalId("input"),
        sourceOrDefinition,
        options,
      );
    },
    computedSpec(id, spec, options) {
      return callableSignals.spec.computed(id, spec, options);
    },
    computed(specOrCompute, options) {
      return createWorkerFirstSyncRecipeHandle(
        rootSession,
        "computed",
        rootSession.nextGeneratedStandaloneSignalId("computed"),
        specOrCompute,
        options,
        "signals.computed",
      );
    },
    async computedAsync(specOrCompute, options) {
      const normalizedOptions = normalizeWorkerFirstAsyncRecipeOptions("computed", options);
      return createWorkerFirstAsyncRecipeHandle(
        rootSession,
        "computed",
        rootSession.nextGeneratedStandaloneSignalId("computed"),
        specOrCompute,
        normalizedOptions,
      );
    },
    computedCallback(id, compute, options) {
      return createWorkerFirstSyncComputedCallbackHandle(
        rootSession,
        id,
        compute,
        options,
      );
    },
    outputSpec(id, spec, options) {
      return callableSignals.spec.output(id, spec, options);
    },
    output(specOrCompute, options) {
      return createWorkerFirstSyncRecipeHandle(
        rootSession,
        "output",
        rootSession.nextGeneratedStandaloneSignalId("output"),
        specOrCompute,
        options,
        "signals.output",
      );
    },
    async outputAsync(specOrCompute, options) {
      const normalizedOptions = normalizeWorkerFirstAsyncRecipeOptions("output", options);
      return createWorkerFirstAsyncRecipeHandle(
        rootSession,
        "output",
        rootSession.nextGeneratedStandaloneSignalId("output"),
        specOrCompute,
        normalizedOptions,
      );
    },
    outputCallback(id, compute, options) {
      return createWorkerFirstSyncOutputCallbackHandle(
        rootSession,
        id,
        compute,
        options,
      );
    },
    graph(id, definitionOrBuilder) {
      return namespace().graph(id, definitionOrBuilder);
    },
    importGraph(definition, snapshot) {
      const graph = createWorkerFirstRootImportedGraph(rootSession, { definition, snapshot });
      trackedGraphs.add(graph);
      return freezeObject({
        ...graph,
        async terminate() {
          trackedGraphs.delete(graph);
          await graph.terminate();
        },
      });
    },
    read(target) {
      return readRootSignalValue(rootSession, target);
    },
    transaction(callback) {
      return runWorkerFirstAsyncTransaction(
        rootSession,
        callback,
        "signals.transaction",
      );
    },
    batch(callback) {
      return runWorkerFirstAsyncTransaction(
        rootSession,
        callback,
        "signals.batch",
      );
    },
    transactionAsync(callback) {
      return runWorkerFirstAsyncTransaction(
        rootSession,
        callback,
        "signals.transactionAsync",
      );
    },
    batchAsync(callback) {
      return runWorkerFirstAsyncTransaction(
        rootSession,
        callback,
        "signals.batchAsync",
      );
    },
    watch(target, callback) {
      return rootSession.watch(target, callback);
    },
    effect(target, callback) {
      return rootSession.effect(target, callback);
    },
    nuke(handle) {
      return rootSession.nuke(handle);
    },
    diagnostics() {
      diagnostics ??= createRootDiagnosticsFacade(rootSession);
      return diagnostics;
    },
    history() {
      history ??= readRootHistoryFacade(rootSession);
      return history;
    },
    specialist() {
      specialist ??= createRootSpecialistFacade(rootSession);
      return specialist;
    },
    contract() {
      return contract;
    },
    assertCompatibility(options) {
      return assertSignalsRuntimeCompatibility(
        contract,
        options,
        "signals.assertCompatibility",
      );
    },
    adapters() {
      adapters ??= createRootAdaptersFacade(rootSession);
      return adapters;
    },
    compatibilityApp() {
      throwWorkerFirstCallableUnavailable("signals.compatibilityApp");
    },
    compatibilityRuntime() {
      throwWorkerFirstCallableUnavailable("signals.compatibilityRuntime");
    },
    terminate,
    free() {
      void callableSignals.terminate();
    },
    [Symbol.dispose]() {
      void callableSignals.terminate();
    },
  };

  return freezeObject(callableSignals);
}

function createUnavailableNamespace(operation) {
  return new Proxy(
    freezeObject({}),
    {
      get(_target, property) {
        void property;
        throwWorkerFirstCallableUnavailable(operation);
      },
      has() {
        return false;
      },
      ownKeys() {
        return [];
      },
      getOwnPropertyDescriptor() {
        return undefined;
      },
    },
  );
}

function normalizeWorkerFirstInputOptions(options) {
  if (options === undefined) {
    return undefined;
  }
  const normalized = requireAuthoringOptions("input", options);
  forbidOpaqueIdOption("input", normalized);
  return normalized;
}

function throwWorkerFirstCallableUnavailable(operation) {
  const error = new Error(
    `${operation} is unavailable on the current worker-first callable surface; use deployment: "mainThreadCompatibility" for authoring and root runtime operations beyond imported graph hydration`,
  );
  error.name = "WorkerFirstCallableSurfaceUnavailable";
  error.code = "workerFirstCallableSurfaceUnavailable";
  error.compatibilityRecovery = Object.freeze({
    deployment: "mainThreadCompatibility",
    message:
      'Retry with deployment: "mainThreadCompatibility" to use the full callable root surface.',
  });
  throw error;
}
