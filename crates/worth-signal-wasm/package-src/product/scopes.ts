import { buildControllerContract } from "./controllers.js";
import { createApiFactory, createApiScopeFactory } from "./api/api_namespace.js";
import { createFeatureStoreFactory } from "./feature_store/feature_store_factory.js";
import { createLinkedSignal } from "./linked.js";
import { createLocalNamespace } from "./local/local_namespace.js";
import { createResourceNamespace } from "./resource/facade.js";
import { createRouterNamespace } from "./router/router_namespace.js";
import {
  canonicalId,
  createScopeDescriptor,
  hasExplicitAuthoringIdOption,
  joinScopeId,
  nextGeneratedAuthoringSignalId,
  requireNonEmptyString,
  signalIdentityForScope,
  stripExplicitAuthoringIdOption,
  withPrivateAuthoringId,
} from "./scope_authoring_support.js";
import { tagScopedHandle } from "./scope_handle_tagging.js";
import {
  assertSignalsRuntimeCompatibility,
  createSignalsRuntimeContract,
} from "./runtime_contract.js";
import { RAW_SIGNALS } from "./symbols.js";

function nextGeneratedScopedId(rawSignals, scopeId, family) {
  return nextGeneratedAuthoringSignalId(rawSignals, family, scopeId);
}

export function createScopedSignalNamespace(
  callableSignals,
  rawSignals,
  localScopeId,
  parentScope = null,
  explicitGraphOwnerId = undefined,
) {
  requireNonEmptyString(
    localScopeId,
    "signals.scope requires a non-empty string scope id",
  );
  const parentScopeId = parentScope?.scopeId ?? null;
  const scopeId = joinScopeId(parentScopeId, localScopeId);
  const graphOwnerId =
    explicitGraphOwnerId ?? parentScope?.graphOwnerId ?? null;
  const descriptor = createScopeDescriptor(
    scopeId,
    localScopeId,
    parentScopeId,
    graphOwnerId,
  );
  const contract = createSignalsRuntimeContract({
    surfaceFamily: "mainThreadCompatibilityScoped",
    deployment: "mainThreadCompatibility",
    scopeId,
    capabilities: {
      callableSurface: false,
      scopedAuthoring: true,
      specNamespace: true,
      workerRuntime: false,
    },
  });

  const bindScopedHandle = (handle, localId = null) => tagScopedHandle(
    handle,
    descriptor,
    scopeId,
    graphOwnerId,
    typeof localId === "string"
      ? signalIdentityForScope(descriptor, graphOwnerId, localId)
      : null,
    localId,
  );

  const scopedNamespace = {
    host: callableSignals.host,
    resource: null,
    api: null,
    apiScope: null,
    featureStore: null,
    local: null,
    router: null,
    spec: Object.freeze({
      input(id, initial, options) {
        return bindScopedHandle(
          callableSignals.spec.input(
            canonicalId(scopeId, id),
            initial,
            options,
          ),
          id,
        );
      },
      computed(id, spec) {
        return bindScopedHandle(
          callableSignals.spec.computed(canonicalId(scopeId, id), spec),
          id,
        );
      },
      computedCallback(id, callback, options) {
        return bindScopedHandle(
          callableSignals.spec.computedCallback(
            canonicalId(scopeId, id),
            callback,
            options,
          ),
          id,
        );
      },
      output(id, spec) {
        return bindScopedHandle(
          callableSignals.spec.output(canonicalId(scopeId, id), spec),
          id,
        );
      },
      outputCallback(id, callback, options) {
        return bindScopedHandle(
          callableSignals.spec.outputCallback(
            canonicalId(scopeId, id),
            callback,
            options,
          ),
          id,
        );
      },
    }),
    scope(childLocalScopeId) {
      return createScopedSignalNamespace(
        callableSignals,
        rawSignals,
        childLocalScopeId,
        scopedNamespace,
      );
    },
    controller(definitionOrBuilder) {
      return buildControllerContract(scopedNamespace, definitionOrBuilder);
    },
    publicInput(handle, options) {
      return callableSignals.publicInput(handle, options);
    },
    input(firstArg, secondArg, thirdArg) {
      if (hasExplicitAuthoringIdOption(secondArg)) {
        return bindScopedHandle(
          callableSignals.spec.input(
            canonicalId(scopeId, secondArg.id),
            firstArg,
            stripExplicitAuthoringIdOption(secondArg),
          ),
          secondArg.id,
        );
      }
      const authoringId = nextGeneratedScopedId(rawSignals, scopeId, "input");
      return bindScopedHandle(
        callableSignals.input(
          firstArg,
          withPrivateAuthoringId(secondArg, authoringId),
        ),
      );
    },
    inputAsync(firstArg, secondArg, thirdArg) {
      if (hasExplicitAuthoringIdOption(secondArg)) {
        return Promise.resolve(
          bindScopedHandle(
            callableSignals.spec.input(
              canonicalId(scopeId, secondArg.id),
              firstArg,
              stripExplicitAuthoringIdOption(secondArg),
            ),
            secondArg.id,
          ),
        );
      }
      const authoringId = nextGeneratedScopedId(rawSignals, scopeId, "input");
      return Promise.resolve(
        bindScopedHandle(
          callableSignals.input(
            firstArg,
            withPrivateAuthoringId(secondArg, authoringId),
          ),
        ),
      );
    },
    computedSpec(id, spec, options) {
      return bindScopedHandle(
        callableSignals.spec.computed(
          canonicalId(scopeId, id),
          spec,
          options,
        ),
        id,
      );
    },
    linked(sourceOrDefinition, options) {
      return bindScopedHandle(
        createLinkedSignal(
          scopedNamespace,
          rawSignals,
          sourceOrDefinition,
          options,
        ),
      );
    },
    linkedAsync(sourceOrDefinition, options) {
      return Promise.resolve(this.linked(sourceOrDefinition, options));
    },
    computed(firstArg, secondArg, thirdArg) {
      if (typeof firstArg === "string") {
        if (typeof secondArg === "function") {
          if (thirdArg !== undefined) {
            throw new TypeError(
              "scoped computed callback form does not accept options after an explicit id",
            );
          }
          return bindScopedHandle(
            callableSignals.spec.computedCallback(
              canonicalId(scopeId, firstArg),
              secondArg,
            ),
            firstArg,
          );
        }
        if (thirdArg !== undefined) {
          throw new TypeError(
            "scoped computed spec form does not accept a third argument after an explicit id",
          );
        }
        return bindScopedHandle(
          callableSignals.spec.computed(
            canonicalId(scopeId, firstArg),
            secondArg,
          ),
          firstArg,
        );
      }
      if (hasExplicitAuthoringIdOption(secondArg)) {
        const localId = secondArg.id;
        if (typeof firstArg === "function") {
          return bindScopedHandle(
            callableSignals.spec.computedCallback(
              canonicalId(scopeId, localId),
              firstArg,
              stripExplicitAuthoringIdOption(secondArg),
            ),
            localId,
          );
        }
        return bindScopedHandle(
          callableSignals.spec.computed(
            canonicalId(scopeId, localId),
            firstArg,
            stripExplicitAuthoringIdOption(secondArg),
          ),
          localId,
        );
      }
      const authoringId = nextGeneratedScopedId(
        rawSignals,
        scopeId,
        "computed",
      );
      return bindScopedHandle(
        callableSignals.computed(
          firstArg,
          withPrivateAuthoringId(secondArg, authoringId),
        ),
      );
    },
    computedAsync(firstArg, secondArg, thirdArg) {
      return Promise.resolve(
        scopedNamespace.computed(firstArg, secondArg, thirdArg),
      );
    },
    outputSpec(id, spec, options) {
      return bindScopedHandle(
        callableSignals.spec.output(
          canonicalId(scopeId, id),
          spec,
          options,
        ),
        id,
      );
    },
    output(firstArg, secondArg, thirdArg) {
      if (typeof firstArg === "string") {
        if (typeof secondArg === "function") {
          if (thirdArg !== undefined) {
            throw new TypeError(
              "scoped output callback form does not accept options after an explicit id",
            );
          }
          return bindScopedHandle(
            callableSignals.spec.outputCallback(
              canonicalId(scopeId, firstArg),
              secondArg,
            ),
            firstArg,
          );
        }
        if (thirdArg !== undefined) {
          throw new TypeError(
            "scoped output spec form does not accept a third argument after an explicit id",
          );
        }
        return bindScopedHandle(
          callableSignals.spec.output(
            canonicalId(scopeId, firstArg),
            secondArg,
          ),
          firstArg,
        );
      }
      if (hasExplicitAuthoringIdOption(secondArg)) {
        const localId = secondArg.id;
        if (typeof firstArg === "function") {
          return bindScopedHandle(
            callableSignals.spec.outputCallback(
              canonicalId(scopeId, localId),
              firstArg,
              stripExplicitAuthoringIdOption(secondArg),
            ),
            localId,
          );
        }
        return bindScopedHandle(
          callableSignals.spec.output(
            canonicalId(scopeId, localId),
            firstArg,
            stripExplicitAuthoringIdOption(secondArg),
          ),
          localId,
        );
      }
      const authoringId = nextGeneratedScopedId(rawSignals, scopeId, "output");
      return bindScopedHandle(
        callableSignals.output(
          firstArg,
          withPrivateAuthoringId(secondArg, authoringId),
        ),
      );
    },
    outputAsync(firstArg, secondArg, thirdArg) {
      return Promise.resolve(
        scopedNamespace.output(firstArg, secondArg, thirdArg),
      );
    },
    outputCallback(id, callback, options) {
      return bindScopedHandle(
        callableSignals.spec.outputCallback(
          canonicalId(scopeId, id),
          callback,
          options,
        ),
        id,
      );
    },
    graph: callableSignals.graph.bind(callableSignals),
    history: callableSignals.history.bind(callableSignals),
    canonicalId(localId) {
      return canonicalId(scopeId, localId);
    },
    signalIdentity(localId) {
      requireNonEmptyString(
        localId,
        "scoped authoring requires a non-empty local id",
      );
      return signalIdentityForScope(descriptor, graphOwnerId, localId);
    },
    descriptor() {
      return descriptor;
    },
    get scopeId() {
      return descriptor.id;
    },
    get localScopeId() {
      return descriptor.localScopeId;
    },
    get parentScopeId() {
      return descriptor.parentScopeId;
    },
    get graphOwnerId() {
      return descriptor.graphOwnerId;
    },
    contract() {
      return contract;
    },
    assertCompatibility(options) {
      return assertSignalsRuntimeCompatibility(
        contract,
        options,
        `signals.scope(${JSON.stringify(scopeId)}).assertCompatibility`,
      );
    },
    // Host tip surface: scopes share the root's tip epochs and drain.
    settleAuthoredWork() {
      return callableSignals.settleAuthoredWork();
    },
    authoredSettleInvocationCount() {
      return callableSignals.authoredSettleInvocationCount();
    },
    commitHostTipAndNotify(tipWrites) {
      return callableSignals.commitHostTipAndNotify(tipWrites);
    },
    applyCommittedTipWorkerBatch(tipWrites) {
      return callableSignals.applyCommittedTipWorkerBatch(tipWrites);
    },
    publishAuthoredTipProjection(changedIds) {
      return callableSignals.publishAuthoredTipProjection(changedIds);
    },
    [RAW_SIGNALS]: rawSignals,
  };

  scopedNamespace.resource = createResourceNamespace(
    scopedNamespace,
    rawSignals,
  );
  scopedNamespace.api = createApiFactory(scopedNamespace);
  scopedNamespace.apiScope = createApiScopeFactory(scopedNamespace);
  scopedNamespace.featureStore = createFeatureStoreFactory(scopedNamespace);
  scopedNamespace.local = createLocalNamespace(scopedNamespace);
  scopedNamespace.router = createRouterNamespace(scopeId);

  return Object.freeze(scopedNamespace);
}

export {
  nextGeneratedAuthoringSignalId,
  reserveAuthoringSignalId,
} from "./scope_authoring_support.js";
