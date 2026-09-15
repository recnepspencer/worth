import {
  applyCompatibilityCommittedTipBatch,
  commitCompatibilityHostTip,
  publishCompatibilityAuthoredTipProjection,
} from "./authoring/compatibility_host_tip.js";
import { withComputedCallbackFrame } from "./callback_frames.js";
import { buildControllerContract } from "./controllers.js";
import { createHostCapabilities } from "./host_capabilities.js";
import { wrapDiagnostics } from "./diagnostics.js";
import { createFormController } from "./forms/form_controller.js";
import { defineFormDeclaration } from "./forms/form_declaration.js";
import { createFormSourceFactory } from "./forms/sources/form_sources.js";
import { requireRouteFormsAuthorityArtifact } from "./router/projection/admission/router_forms_authority_artifact.js";
import { createApiFactory, createApiScopeFactory } from "./api/api_namespace.js";
import { createFeatureStoreFactory } from "./feature_store/feature_store_factory.js";
import { createRouterNamespace } from "./router/router_namespace.js";
import {
  createImportedSignalGraph,
  createPublishedSignalGraph,
} from "./graphs.js";
import { wrapHistory } from "./history.js";
import {
  unwrapSignalTarget,
  wrapInputSignal,
  wrapReadableSignal,
} from "./handles.js";
import { createLinkedSignal } from "./linked.js";
import { createLocalNamespace } from "./local/local_namespace.js";
import { createCompatibilityLocalTruthFactory } from "./local_truth/signals_local_truth_factory.js";
import {
  nextOutputProjectionId,
  outputProjectionSpec,
} from "./output_projection_ids.js";
import { createPublicGraphInputEntry } from "./public_inputs.js";
import { createResourceNamespace } from "./resource/facade.js";
import { withReservedSignalId } from "./reserved_authoring_ids.js";
import { createScopedSignalNamespace } from "./scopes.js";
import { wrapSpecialist } from "./specialist.js";
import {
  assertSignalsRuntimeCompatibility,
  createSignalsRuntimeContract,
} from "./runtime_contract.js";
import { RAW_SIGNALS } from "./symbols.js";
import { wrapAdapters, wrapTransaction } from "./transactions.js";
import {
  cloneSignalValue,
  explicitSignalSpecNamespace,
  parseOpaqueCallbackOptions,
  parseOpaqueInputArgs,
  parseOpaqueSpecOptions,
} from "./authoring/compatibility_signal_authoring.js";
import {
  createCompatibilityObservation,
  RAW_OBSERVATION_HANDLE,
} from "./authoring/compatibility_observation.js";

export * from "./public_helpers.js";

export function wrapSignals(rawSignals, options) {
  const hostCapabilities = createHostCapabilities(rawSignals, options);
  let diagnostics = null;
  let history = null;
  let authoredSettleInvocations = 0;
  // Releasing the runtime is idempotent: terminate(), free() and
  // Symbol.dispose all release once, and a second release after any of them
  // is a no-op rather than a second free() into the wasm handle.
  let released = false;
  function releaseRuntime(releaseRaw) {
    if (released) {
      return;
    }
    released = true;
    hostCapabilities.dispose();
    releaseRaw();
  }
  const contract = createSignalsRuntimeContract({
    surfaceFamily: "mainThreadCompatibilityCallable",
    deployment: "mainThreadCompatibility",
    capabilities: {
      callableSurface: true,
      scopedAuthoring: true,
      specNamespace: true,
      workerRuntime: false,
    },
  });
  const explicitSpec = explicitSignalSpecNamespace(rawSignals);
  const formSourceFactory = createFormSourceFactory();
  function createForm(declaration) {
    return createFormController(callableSignals, declaration, {
      requireRouteFormsAuthorityArtifact,
    });
  }
  Object.defineProperty(createForm, "source", {
    enumerable: true,
    value: formSourceFactory,
  });
  Object.defineProperty(createForm, "define", {
    enumerable: true,
    value: defineFormDeclaration,
  });
  const callableSignals = {
    host: hostCapabilities.host,
    resource: createResourceNamespace(null, rawSignals),
    api: null,
    apiScope: null,
    featureStore: null,
    local: null,
    localTruth: null,
    router: null,
    spec: explicitSpec,
    scope(localScopeId) {
      return createScopedSignalNamespace(
        callableSignals,
        rawSignals,
        localScopeId,
      );
    },
    controller(definitionOrBuilder) {
      return buildControllerContract(callableSignals, definitionOrBuilder);
    },
    form: createForm,
    publicInput(handle, options) {
      return createPublicGraphInputEntry(handle, options);
    },
    input(idOrInitial, initialOrOptions, maybeOptions) {
      const { id, initial, options, debugName } = parseOpaqueInputArgs(
        rawSignals,
        idOrInitial,
        initialOrOptions,
        maybeOptions,
      );
      return withReservedSignalId(rawSignals, "input", id, () =>
        wrapInputSignal(
          rawSignals.input(id, initial, options),
          rawSignals,
          cloneSignalValue(initial),
          debugName,
        ),
      );
    },
    inputAsync(idOrInitial, initialOrOptions, maybeOptions) {
      return Promise.resolve(
        callableSignals.input(idOrInitial, initialOrOptions, maybeOptions),
      );
    },
    linked(sourceOrDefinition, options) {
      return createLinkedSignal(
        callableSignals,
        rawSignals,
        sourceOrDefinition,
        options,
      );
    },
    linkedAsync(sourceOrDefinition, options) {
      return Promise.resolve(callableSignals.linked(sourceOrDefinition, options));
    },
    computedSpec(id, spec, options) {
      return explicitSpec.computed(id, spec, options);
    },
    computed(idOrCompute, specOrCompute, maybeOptions) {
      const callbackArgs = parseOpaqueCallbackOptions(
        rawSignals,
        "computed",
        idOrCompute,
        specOrCompute,
        maybeOptions,
      );
      if (callbackArgs) {
        const callback = withComputedCallbackFrame(
          rawSignals,
          callbackArgs.callback,
        );
        return withReservedSignalId(
          rawSignals,
          "computed",
          callbackArgs.id,
          () =>
            wrapReadableSignal(
              rawSignals.computedCallback(callbackArgs.id, callback),
              rawSignals,
              "computed",
              callbackArgs.debugName,
            ),
        );
      }
      const { id, spec, debugName } = parseOpaqueSpecOptions(
        rawSignals,
        "computed",
        idOrCompute,
        specOrCompute,
        maybeOptions,
      );
      return withReservedSignalId(rawSignals, "computed", id, () =>
        wrapReadableSignal(
          rawSignals.computedSpec(id, spec),
          rawSignals,
          "computed",
          debugName,
        ),
      );
    },
    computedAsync(idOrCompute, specOrCompute, maybeOptions) {
      return Promise.resolve(
        callableSignals.computed(idOrCompute, specOrCompute, maybeOptions),
      );
    },
    outputSpec(id, spec, options) {
      return explicitSpec.output(id, spec, options);
    },
    output(idOrSpec, specOrCompute, maybeOptions) {
      const callbackArgs = parseOpaqueCallbackOptions(
        rawSignals,
        "output",
        idOrSpec,
        specOrCompute,
        maybeOptions,
      );
      if (callbackArgs) {
        const wrappedCallback = withComputedCallbackFrame(
          rawSignals,
          callbackArgs.callback,
        );
        const hiddenComputedId = nextOutputProjectionId(
          rawSignals,
          callbackArgs.id,
        );
        return withReservedSignalId(
          rawSignals,
          "output",
          callbackArgs.id,
          () => {
            rawSignals.computedCallback(hiddenComputedId, wrappedCallback);
            return wrapReadableSignal(
              rawSignals.outputSpec(
                callbackArgs.id,
                outputProjectionSpec(hiddenComputedId),
              ),
              rawSignals,
              "output",
              callbackArgs.debugName,
            );
          },
        );
      }
      const { id, spec, debugName } = parseOpaqueSpecOptions(
        rawSignals,
        "output",
        idOrSpec,
        specOrCompute,
        maybeOptions,
      );
      return withReservedSignalId(rawSignals, "output", id, () =>
        wrapReadableSignal(
          rawSignals.outputSpec(id, spec),
          rawSignals,
          "output",
          debugName,
        ),
      );
    },
    outputAsync(idOrSpec, specOrCompute, maybeOptions) {
      return Promise.resolve(
        callableSignals.output(idOrSpec, specOrCompute, maybeOptions),
      );
    },
    outputCallback(id, callback, options) {
      return explicitSpec.outputCallback(id, callback, options);
    },
    graph(id, definition) {
      return createPublishedSignalGraph(
        callableSignals,
        rawSignals,
        id,
        definition,
      );
    },
    importGraph(definition, snapshot) {
      return createImportedSignalGraph(
        callableSignals,
        rawSignals,
        definition,
        snapshot,
      );
    },
    read(target) {
      return rawSignals.read(
        unwrapSignalTarget(target, rawSignals, "signals.read"),
      );
    },
    watch(target, callback) {
      return createCompatibilityObservation(rawSignals, "watch", target, callback);
    },
    effect(target, callback) {
      return createCompatibilityObservation(rawSignals, "effect", target, callback);
    },
    transaction(callback) {
      return rawSignals.transaction((rawTx) =>
        callback(wrapTransaction(rawTx, rawSignals)),
      );
    },
    batch(callback) {
      return rawSignals.batch((rawTx) =>
        callback(wrapTransaction(rawTx, rawSignals)),
      );
    },
    transactionAsync(callback) {
      return Promise.resolve(
        rawSignals.transaction((rawTx) =>
          callback(wrapTransaction(rawTx, rawSignals)),
        ),
      );
    },
    batchAsync(callback) {
      return Promise.resolve(
        rawSignals.batch((rawTx) =>
          callback(wrapTransaction(rawTx, rawSignals)),
        ),
      );
    },
    nuke(handle) {
      return rawSignals.nuke(handle?.[RAW_OBSERVATION_HANDLE] ?? handle);
    },
    diagnostics() {
      if (!diagnostics) {
        diagnostics = wrapDiagnostics(
          rawSignals.diagnostics(),
          hostCapabilities,
        );
      }
      return diagnostics;
    },
    history() {
      if (!history) {
        history = wrapHistory(rawSignals.history());
      }
      return history;
    },
    specialist() {
      return wrapSpecialist(rawSignals.specialist());
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
      return wrapAdapters(rawSignals.adapters(), hostCapabilities);
    },
    compatibilityApp: rawSignals.compatibilityApp.bind(rawSignals),
    compatibilityRuntime: rawSignals.compatibilityRuntime.bind(rawSignals),
    // Host tip surface (see host_tip_surface.d.ts). The compatibility runtime
    // applies tips synchronously, so the authored-work drain resolves at once;
    // the invocation counter exists so proofs can assert it was (not) called.
    settleAuthoredWork() {
      authoredSettleInvocations += 1;
      return Promise.resolve();
    },
    authoredSettleInvocationCount() {
      return authoredSettleInvocations;
    },
    commitHostTipAndNotify(tipWrites) {
      return commitCompatibilityHostTip(rawSignals, tipWrites);
    },
    applyCommittedTipWorkerBatch(tipWrites) {
      return applyCompatibilityCommittedTipBatch(rawSignals, tipWrites);
    },
    publishAuthoredTipProjection(changedIds) {
      publishCompatibilityAuthoredTipProjection(rawSignals, changedIds);
    },
    async terminate() {
      releaseRuntime(() => rawSignals.free());
    },
    free() {
      releaseRuntime(() => rawSignals.free());
    },
    [Symbol.dispose]() {
      releaseRuntime(() => {
        if (typeof rawSignals[Symbol.dispose] === "function") {
          rawSignals[Symbol.dispose]();
          return;
        }
        rawSignals.free();
      });
    },
    [RAW_SIGNALS]: rawSignals,
  };
  callableSignals.resource = createResourceNamespace(
    callableSignals,
    rawSignals,
  );
  callableSignals.api = createApiFactory(callableSignals);
  callableSignals.apiScope = createApiScopeFactory(callableSignals);
  callableSignals.featureStore = createFeatureStoreFactory(callableSignals);
  callableSignals.local = createLocalNamespace(callableSignals);
  callableSignals.localTruth = createCompatibilityLocalTruthFactory(callableSignals);
  callableSignals.router = createRouterNamespace();
  return callableSignals;
}
