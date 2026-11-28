export function buildTypeSmokeSource(packageName) {
  return `import init, { clockCapability, createSignals, hostCapabilityPlan, onlineCapability, persistenceCapability, resourceCollectionShape, resourceDelivery, resourceItemAspects, resourcePatch, resourceValueSummaries, viewportCapability, visibilityCapability, type ApiCollectionResourceFamily, type ApiDetailResourceFamily, type GraphMutationRequest, type PublishedGraphTransaction, type ScopedSignalNamespace, type SignalNamespace } from "${packageName}";
import {
  createReactSignalsStore,
  useOutputValue,
  useSignalValue,
  useSignalsDiagnostics,
} from "${packageName}/react";

const initialized = init();
let visibilityState: "visible" | "hidden" = "visible";
let viewportState = { width: 1280, height: 720 };
let onlineState: "online" | "offline" = "online";
let clockTick = 0;
let persistedDraft = { mode: "draft" as const, revision: 1 };
type ShippingOption = { id: string; label: string };
const emptyHostCapabilityPlan = hostCapabilityPlan();
const signals = await createSignals({
  deployment: "mainThreadCompatibility",
  hostCapabilities: hostCapabilityPlan({
    visibility: visibilityCapability({
      source: {
        current() {
          return visibilityState;
        },
        subscribe() {
          return () => {};
        },
      },
    }),
    viewport: viewportCapability({
      source: {
        current() {
          return viewportState;
        },
        subscribe() {
          return () => {};
        },
      },
    }),
    online: onlineCapability({
      source: {
        current() {
          return onlineState;
        },
        subscribe() {
          return () => {};
        },
      },
    }),
    clock: clockCapability({
      source: {
        current() {
          return clockTick;
        },
      },
      pollMs: 5,
    }),
    persistence: persistenceCapability({
      source: {
        current() {
          return persistedDraft;
        },
      },
    }),
  }),
});
const api = signals.api({ baseUrl: "/api" });
const auditScope = signals.scope("audit");
const auditInput = auditScope.input("value", { debugName: "x" });
const auditComputed = auditScope.computedSpec("computedValue", {
  compute: () => auditInput.value(),
});
const auditOutput = auditScope.outputSpec("outputValue", {
  compute: () => auditComputed.value(),
});
const collectionShapeFactory = resourceCollectionShape;
const deliveryFactory = resourceDelivery;
const itemAspectFactory = resourceItemAspects;
const patchFactory = resourcePatch;
const valueSummaryFactory = resourceValueSummaries;
const auditUserDetail = api.url("/users/:userId").detail({
  load: ({ userId }) => ({
    id: String(userId),
    name: "Ada",
  }),
});
const auditTaskList = api.url("/tasks").params<{
  search?: string;
}>().list({
  itemIdentity: (item: { id: string }) => item.id,
  load: ({ params }) => [{ id: params.search ?? "all" }],
});
const auditAsyncWorkspaceDetail = api.url("/workspaces/:workspaceId").detail({
  load: async ({ workspaceId }) => ({ id: String(workspaceId) }),
});
const auditAsyncWorkspaceVersions = api.url("/workspaces/:workspaceId/versions/:versionId").list({
  itemIdentity: (item: { id: string }) => item.id,
  load: async ({ workspaceId, versionId }) => [{ id: String(workspaceId) + ":" + String(versionId) }],
});
const typedAuditAsyncWorkspaceDetail:
  ApiDetailResourceFamily<"/workspaces/:workspaceId", undefined, { id: string }> =
    auditAsyncWorkspaceDetail;
const typedAuditAsyncWorkspaceVersions:
  ApiCollectionResourceFamily<
    "/workspaces/:workspaceId/versions/:versionId",
    undefined,
    readonly { id: string }[],
    { id: string }
  > = auditAsyncWorkspaceVersions;
const typedAuditAsyncWorkspaceDetailLine = typedAuditAsyncWorkspaceDetail.line({
  workspaceId: "ws-1",
});
const typedAuditAsyncWorkspaceVersionsLine = typedAuditAsyncWorkspaceVersions.line({
  workspaceId: "ws-1",
  versionId: 3,
});
const auditUserLine = auditUserDetail.line({ userId: "user-1" });
const auditTaskLine = auditTaskList.line({ params: { search: "ada" } });
const count = signals.input(1, { debugName: "count" });
const hostViewport = signals.host.viewport;
const doubled = signals.computed(() => count() * 2, { debugName: "doubled" });
const hostVisibility = signals.host.visibility;
const hostOnline = signals.host.online;
const hostClock = signals.host.clock;
const hostPersistence = signals.host.persistence;
const viewportLabel = signals.computed(
  () => (hostViewport?.width() ?? 0) + "x" + (hostViewport?.height() ?? 0),
  { debugName: "viewportLabel" },
);
const persistenceLabel = signals.computed(
  () => hostPersistence?.value().revision ?? 0,
  { debugName: "persistenceLabel" },
);
const countResetCommit = count.reset();
const localDraft = signals.input({
  title: "Ship docs",
  done: false,
  status: "draft",
}, { debugName: "localDraft" });
const localDraftPatchCommit = localDraft.patch({
  done: true,
});
const localDraftAssignCommit = localDraft.assign({
  title: "Ready to ship",
});
signals.transaction((tx) => {
  tx.patch(localDraft, {
    status: "queued",
  });
});
const optionList = signals.input([
  { id: "draft", label: "Draft" },
  { id: "review", label: "Review" },
]);
// @ts-expect-error assign is restricted to plain object inputs
optionList.assign([{ id: "ready", label: "Ready" }]);
// @ts-expect-error primitive input patch is not admitted
count.patch(2);
// @ts-expect-error primitive input assign is not admitted
count.assign({ value: 2 });
signals.transaction((tx) => {
  // @ts-expect-error primitive inputs must not admit transaction patch helpers
  tx.patch(count, 4);
});
const shippingOptions = signals.input([
  { id: "ground", label: "Ground" },
  { id: "air", label: "Air" },
], { debugName: "shippingOptions" });
const firstShippingOption = signals.linked(() => shippingOptions()[0], {
  debugName: "firstShippingOption",
});
const preservedShippingOption = signals.linked<ShippingOption[], ShippingOption>({
  source: () => shippingOptions(),
  computation: (options, previous) =>
    options.find((option) => option.id === previous?.value?.id) ?? options[0],
  debugName: "preservedShippingOption",
});
const linkedRelinkCommit = preservedShippingOption.relink();
const linkedResetCommit = preservedShippingOption.reset();
// @ts-expect-error linked app lane must not accept explicit ids
signals.linked(() => 1, { id: "count" });
const panel = signals.output(() => ({
  count: count(),
  doubled: doubled(),
}), { debugName: "panel" });
const name = signals.input("Ada", { debugName: "name" });
const displayLabel = signals.computed(
  () => name().toUpperCase(),
  { debugName: "displayLabel" },
);
const namingGraph = signals.graph("naming", {
  inputs: {
    name,
  },
  outputs: {
    publicDisplayName: displayLabel,
  },
});
// @ts-expect-error primitive graph inputs must not admit patch helpers
namingGraph.patchInput("name", "Grace");
namingGraph.transaction((tx) => {
  // @ts-expect-error primitive graph inputs must not admit graph transaction patch helpers
  tx.patch("name", "Grace");
});
const requirednessGraph = signals.graph("requiredness", (graph) => {
  const boundary = graph.scope("boundary");
  const requirednessServerValue = boundary.input({
    id: "task-7",
    title: "Ship docs",
  });
  const requirednessDraftValue = boundary.input({
    title: "Ship docs",
  });
  const effectiveValue = boundary.computed(() => ({
    ...requirednessServerValue(),
    ...requirednessDraftValue(),
  }));

  return graph.expose({
    inputs: {
      serverValue: graph.input.required(requirednessServerValue, { authority: "readOnly" }),
      draftValue: graph.input.optional(requirednessDraftValue),
    },
    outputs: {
      effectiveValue,
    },
  });
});
const requirednessDescriptorKind: "required" | "optional" =
  requirednessGraph.inputDescriptors()[0]?.requiredness ?? "required";
const requirednessAuthorityKind: "required" | "optional" =
  requirednessGraph.operationalContract().authorities.draftValue.requiredness;
(await createSignals({ deployment: "mainThreadCompatibility" })).graph("invalidRequirednessTypes", (graph) => {
  const scope = graph.scope("requiredness");
  const value = scope.spec.input("value", 1);
  // @ts-expect-error contradictory requiredness must be unrepresentable
  const impossibleRequired = graph.input.required(value, { requiredness: "optional" });
  // @ts-expect-error contradictory requiredness must be unrepresentable
  const impossibleOptional = graph.input.optional(value, { requiredness: "required" });
  return graph.expose({
    inputs: {
      requiredValue: impossibleRequired,
      optionalValue: impossibleOptional,
    },
    outputs: {
      echoed: scope.output(() => value()),
    },
  });
});
const typedNamingOutputId: string = namingGraph.output("publicDisplayName").id;
const typedNamingContractOutputId: string =
  namingGraph.contract().outputs.publicDisplayName;
function createEditSessionController(namespace: SignalNamespace) {
  return namespace.controller(({ input, linked, computed }) => {
    const serverItemData = input<{
      workflow_target_state_id?: number | null;
    } | null>(null);
    const draftEdits = input<{
      workflow_target_state_id?: number | null;
    }>({});

    const effectiveItemData = computed(() => ({
      ...(serverItemData() ?? {}),
      ...(draftEdits() ?? {}),
    }));

    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftEdits()).length > 0,
    }));
    const preferredTransition = linked<(number | null)[], number | null>({
      source: () => [null, serverItemData()?.workflow_target_state_id ?? null],
      computation: (options, previous) =>
        options.find((option) => option === previous?.value) ?? options[0],
    });

    return {
      inputs: {
        serverItemData,
        draftEdits,
      },
      outputs: {
        effectiveItemData,
        dirtyState,
        preferredTransition,
      },
    };
  });
}
function createWorkflowController(
  namespace: SignalNamespace,
  editSession: ReturnType<typeof createEditSessionController>,
) {
  return namespace.controller(({ computed }) => {
    const submitReadiness = computed(() => {
      const item = editSession.outputs.effectiveItemData();
      const dirty = editSession.outputs.dirtyState();

      return {
        enabled: dirty.isDirty && Boolean(item.workflow_target_state_id),
        targetStateId: item.workflow_target_state_id ?? null,
      };
    });

    return {
      outputs: { submitReadiness },
    };
  });
}
function createFormController(namespace: SignalNamespace) {
  return namespace.controller(({ input, computed }) => {
    const serverValue = input<{
      id: string;
      title: string;
      status: string;
    }>({
      id: "task-7",
      title: "Ship docs",
      status: "draft",
    });
    const draftValue = input<{
      title?: string;
      status?: string;
    }>({
      title: "Ship docs",
      status: "ready",
    });
    const effectiveValue = computed(() => ({
      ...serverValue(),
      ...draftValue(),
    }));
    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftValue()).length > 0,
    }));
    const validation = computed(() => ({
      titleMissing: !effectiveValue().title,
    }));

    return {
      inputs: {
        serverValue,
        draftValue,
      },
      outputs: {
        effectiveValue,
        dirtyState,
        validation,
      },
    };
  });
}
function createResourceController(
  namespace: SignalNamespace,
  form: ReturnType<typeof createFormController>,
) {
  return namespace.controller(({ input, computed }) => {
    const routeParams = input<{
      taskId: string;
      workspaceId: string;
    }>({
      taskId: "task-7",
      workspaceId: "alpha",
    });
    const resourceQuery = computed(() => ({
      taskId: routeParams().taskId,
      workspaceId: routeParams().workspaceId,
      status: form.outputs.effectiveValue().status,
    }));
    const submitAvailability = computed(() => ({
      enabled: form.outputs.dirtyState().isDirty && !form.outputs.validation().titleMissing,
      taskId: resourceQuery().taskId,
    }));

    return {
      inputs: {
        routeParams,
      },
      outputs: {
        resourceQuery,
        submitAvailability,
      },
    };
  });
}
function createAuthorityController(namespace: SignalNamespace) {
  return namespace.controller(({ input, computed, publicInput }) => {
    const serverValue = input<{
      id: string;
      title: string;
    }>({
      id: "task-7",
      title: "Ship docs",
    });
    const draftValue = input<{
      title?: string;
    }>({
      title: "Ship docs",
    });
    const externalParams = input<{
      taskId: string;
    }>({
      taskId: "task-7",
    });
    const effectiveValue = computed(() => ({
      ...serverValue(),
      ...draftValue(),
      taskId: externalParams().taskId,
    }));

    return {
      inputs: {
        serverValue: publicInput(serverValue, { authority: "readOnly" }),
        draftValue: publicInput(draftValue),
        externalParams: publicInput(externalParams, { authority: "imported" }),
      },
      outputs: {
        effectiveValue,
      },
    };
  });
}
const repeatedRows: ScopedSignalNamespace = signals.scope("rows");
const row0: ScopedSignalNamespace = repeatedRows.scope("row-0");
const row0Descriptor = row0.descriptor();
const row0Identity = row0.signalIdentity("count");
const row0Count = row0.spec.input("count", 0);
const row0HandleId = row0Count.id;
const itemDetailGraph = signals.graph("itemDetail", (graph) => {
  const editSession = graph.controller("editSession", ({ input, linked, computed }) => {
    const serverItemData = input<{
      workflow_target_state_id?: number | null;
    } | null>(null);
    const draftEdits = input<{
      workflow_target_state_id?: number | null;
    }>({});
    const effectiveItemData = computed(() => ({
      ...(serverItemData() ?? {}),
      ...(draftEdits() ?? {}),
    }));
    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftEdits()).length > 0,
    }));
    const preferredTransition = linked<(number | null)[], number | null>({
      source: () => [null, serverItemData()?.workflow_target_state_id ?? null],
      computation: (options, previous) =>
        options.find((option) => option === previous?.value) ?? options[0],
    });

    return {
      inputs: {
        serverItemData,
        draftEdits,
      },
      outputs: {
        effectiveItemData,
        dirtyState,
        preferredTransition,
      },
    };
  });
  const workflow = createWorkflowController(graph.scope("workflow"), editSession);
  return graph.expose({
    controllers: [editSession, workflow],
  });
});
const pageModalGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).graph("itemWorkspace", (graph) => {
  const page = createEditSessionController(graph.scope("page"));
  const modal = createEditSessionController(graph.scope("modal"));
  return graph.expose({
    inputs: {
      pageServerItemData: page.inputs.serverItemData,
      modalServerItemData: modal.inputs.serverItemData,
    },
    outputs: {
      pageEffectiveItemData: page.outputs.effectiveItemData,
      modalEffectiveItemData: modal.outputs.effectiveItemData,
    },
  });
});
const taskEditorGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).graph("taskEditor", (graph) => {
  const form = createFormController(graph.scope("form"));
  const resource = createResourceController(graph.scope("resource"), form);
  return graph.expose({
    controllers: [form, resource],
  });
});
const authorityGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).graph("taskAuthority", (graph) => {
  const authority = createAuthorityController(graph.scope("authority"));
  return graph.expose({
    controllers: [authority],
  });
});
const store = createReactSignalsStore(signals);
const storeInput = store.signals.input(1);
persistedDraft = { mode: "draft", revision: 2 };
const persistenceCommit = hostPersistence?.commit();
const adapters = signals.adapters();
const runtimeEnvelope = adapters.exportRuntimeEnvelope();
adapters.replaceRuntimeEnvelope(runtimeEnvelope);
const runtimeProof = adapters.runtimeProofReport();
const restoredBranchId = runtimeEnvelope.snapshot.snapshot.meta.branch_id;
const snapshotExplanationRetention =
  runtimeEnvelope.snapshot.snapshot.meta.artifact_retention.explanation_retention;
const checkpointImage = runtimeEnvelope.snapshot.snapshot.checkpoint_image;
const diagnosticGraph = runtimeEnvelope.snapshot.snapshot.diagnostic_graph;
const history = signals.history();
const specialist = signals.specialist();
const currentBranch = await Promise.resolve(history.current_branch());
const branchReplay = await Promise.resolve(history.replay_for_branch(currentBranch.id));
const branchSnapshot = await Promise.resolve(history.branch_snapshot(currentBranch.id));
const branchEnvelope = await Promise.resolve(history.branch_snapshot_envelope(currentBranch.id));
const specialistGraphSummary = specialist.graphSummary();
const specialistEvaluateDirty = await Promise.resolve(specialist.evaluateDirty());
await Promise.resolve(history.restore_snapshot(branchEnvelope));
await Promise.resolve(history.restore_branch_snapshot(currentBranch.id, branchSnapshot));
const branchProof = await Promise.resolve(history.branch_state_proof(currentBranch.id));
const parityProof = await Promise.resolve(history.replay_parity_proof(currentBranch.id, currentBranch.id));
const artifactProof = await Promise.resolve(history.replay_artifact_proof({
  proofSchemaVersion: runtimeProof.proofSchemaVersion,
  registryBundleDigest: runtimeProof.registryBundleDigest,
  branchStateDigest: branchProof.stateDigest,
}, currentBranch.id));
const diagnostics = signals.diagnostics();
const latestObservation = diagnostics.latestObservation();
const latestFlow = diagnostics.latestFlow();
const latestHostCapabilityEvent = diagnostics.latestHostCapabilityEvent();
const recentHostCapabilityEvents = diagnostics.recentHostCapabilityEvents();
const hostCapabilityReport = diagnostics.hostCapabilityReport();
const performanceSummary = diagnostics.performanceSummary();
const delivered = latestObservation?.observation.delivered_event_count;
const callbackNodeIds = latestFlow?.callbackNodes.map((node) => node.id) ?? [];
const callbackHostCapabilityCompatibility =
  latestFlow?.callbackNodes[0]?.hostCapabilityReads[0]?.compatibility ??
  latestObservation?.callbackNodes[0]?.hostCapabilityReads[0]?.compatibility ??
  null;
const latestHostCapabilityEventKind = latestHostCapabilityEvent?.kind ?? null;
const latestHostCapabilityQueuedCount = latestHostCapabilityEvent?.queuedInvalidationCount ?? 0;
const latestHostCapabilityDeniedIds = latestHostCapabilityEvent?.deniedCallbackIds ?? [];
const hostCapabilityLineageDigest = hostCapabilityReport.lineageDigest;
const hostCapabilityBreadthDigest = hostCapabilityReport.breadthDigest;
const hostCapabilityLineageEntry = hostCapabilityReport.lineage[0] ?? null;
const hostCapabilityBreadthFamily = hostCapabilityReport.breadth.families[0] ?? null;
const hostCapabilityReadCount = performanceSummary.hostCapabilityReadCount ?? 0;
const hostCapabilityReevaluationCount = performanceSummary.hostCapabilityReevaluationCount ?? 0;
const hostCapabilityCompatibilityDenialCount =
  performanceSummary.hostCapabilityCompatibilityDenialCount ?? 0;
const hostCapabilityPollCount = performanceSummary.hostCapabilityPollCount ?? 0;
const hostCapabilityNoOpPollCount = performanceSummary.hostCapabilityNoOpPollCount ?? 0;
const visibilityMode = hostVisibility?.state() ?? "hidden";
const visibilityDescriptor = hostVisibility?.descriptor();
const viewportSize = hostViewport?.size() ?? { width: 0, height: 0 };
const viewportWidth = hostViewport?.width() ?? 0;
const viewportHeight = hostViewport?.height() ?? 0;
const viewportDescriptor = hostViewport?.descriptor();
const onlineMode = hostOnline?.state() ?? "offline";
const onlineDescriptor = hostOnline?.descriptor();
const onlineFlag = hostOnline?.isOnline() ?? false;
const clockNow = hostClock?.now() ?? 0;
const clockDescriptor = hostClock?.descriptor();
const persistenceValue = hostPersistence?.value() ?? { mode: "draft", revision: 0 };
const persistenceDescriptor = hostPersistence?.descriptor();
const proofVersion = runtimeProof.proofSchemaVersion;
const exportedPolicyPreset = runtimeEnvelope.definitions.policy.preset;
const snapshotPolicyTier = runtimeEnvelope.snapshot.snapshot.meta.runtime_policy.tier;
const snapshotReplayHead = runtimeEnvelope.snapshot.snapshot.meta.replay_head;
const replayHasCallback = branchReplay.frames.some((frame) => frame.callback?.id === "doubled");
const specialistGraphProfile = specialistGraphSummary.profile;
const specialistTouchedNodes = specialistEvaluateDirty.touchedNodes;
const artifactParity = artifactProof.parity;
const panelValue = signals.read(panel);
const panelView = useOutputValue<{ count: number; doubled: number }>(panel, store);
const itemDetailGraphRead = itemDetailGraph.read();
const itemDetailGraphSummary = itemDetailGraph.summary();
const itemDetailGraphContract = itemDetailGraph.contract();
const itemDetailGraphOperationalContract = itemDetailGraph.operationalContract();
const itemDetailGraphOperationalWriteId = itemDetailGraphOperationalContract.writes.serverItemData;
const itemDetailGraphOperationalPatchId = itemDetailGraphOperationalContract.patches.draftEdits;
const itemDetailGraphOperationalAuthority =
  itemDetailGraphOperationalContract.authorities.draftEdits.authority;
const itemDetailGraphOperationalRequest: GraphMutationRequest<{
  serverItemData: typeof itemDetailGraph.inputs.serverItemData;
  draftEdits: typeof itemDetailGraph.inputs.draftEdits;
}> = {
  writes: {
    serverItemData: {
      workflow_target_state_id: 5,
    },
  },
  commands: {},
  reset: ["draftEdits"],
};
const itemDetailGraphWriteCommit = itemDetailGraph.writeInputs({
  serverItemData: {
    workflow_target_state_id: 3,
  },
});
const itemDetailGraphSingleWriteCommit = itemDetailGraph.writeInput("serverItemData", {
  workflow_target_state_id: 4,
});
const itemDetailGraphPatchCommit = itemDetailGraph.patchInputs({
  draftEdits: {
    workflow_target_state_id: 9,
  },
});
const itemDetailGraphSinglePatchCommit = itemDetailGraph.patchInput("draftEdits", {
  workflow_target_state_id: 10,
});
const itemDetailGraphResetCommit = itemDetailGraph.resetInputs(["draftEdits"]);
const itemDetailGraphSingleResetCommit = itemDetailGraph.resetInput("draftEdits");
const itemDetailGraphApplyCommit = itemDetailGraph.apply(itemDetailGraphOperationalRequest);
const itemDetailGraphTransactionCommit = itemDetailGraph.transaction((tx: PublishedGraphTransaction<{
  serverItemData: typeof itemDetailGraph.inputs.serverItemData;
  draftEdits: typeof itemDetailGraph.inputs.draftEdits;
}>) => {
  tx.set("draftEdits", {
    workflow_target_state_id: 11,
  });
  tx.set(itemDetailGraph.inputs.serverItemData, {
    workflow_target_state_id: 12,
  });
  tx.patch("draftEdits", {
    workflow_target_state_id: 13,
  });
});
const itemDetailGraphDiagnostics = itemDetailGraph.inspectDiagnostics();
const itemDetailGraphHistory = itemDetailGraph.inspectHistory();
const itemDetailGraphCompatibility = itemDetailGraph.exportCompatibilityDefinition();
const itemDetailGraphExportDefinition = itemDetailGraph.exportDefinition();
const itemDetailGraphExportSnapshot = itemDetailGraph.exportSnapshot();
const itemDetailGraphDependency = itemDetailGraphDiagnostics.dependenciesForOutput("submitReadiness");
const itemDetailGraphContractSummary = itemDetailGraphDiagnostics.contractSummary();
const itemDetailGraphContractDelta = itemDetailGraph.contractDelta(itemDetailGraphContract);
const itemDetailGraphContractHistory = itemDetailGraph.contractHistory();
const itemDetailGraphContractInputId = itemDetailGraphContract.inputs.serverItemData;
const itemDetailGraphDiagnosticsInputWhy = itemDetailGraphDiagnostics.inputs.serverItemData.why;
const itemDetailGraphHistoryInputReplay = itemDetailGraphHistory.inputs.serverItemData.replay;
const itemDetailGraphHistoryDependency = itemDetailGraphHistory.dependenciesForOutput("submitReadiness");
const itemDetailGraphHistoryContractSummary = itemDetailGraphHistory.contractSummary();
const itemDetailGraphCompatibilityContractInputId =
  itemDetailGraphCompatibility.contract.inputs.serverItemData;
const restoredItemDetailGraph = (await createSignals({ deployment: "mainThreadCompatibility" })).importGraph(
  itemDetailGraphExportDefinition,
  itemDetailGraphExportSnapshot,
);
const restoredItemDetailGraphContract = restoredItemDetailGraph.contract();
const restoredItemDetailGraphContractHistory = restoredItemDetailGraph.contractHistory();
const restoredItemDetailGraphRead = restoredItemDetailGraph.read();
const restoredItemDetailGraphInputs = restoredItemDetailGraph.readInputs();
const taskEditorGraphContract = taskEditorGraph.contract();
const taskEditorGraphOperationalContract = taskEditorGraph.operationalContract();
const taskEditorGraphInputId = taskEditorGraphContract.inputs.serverValue;
const taskEditorGraphOutputId = taskEditorGraphContract.outputs.submitAvailability;
const taskEditorGraphPatchId = taskEditorGraphOperationalContract.patches.draftValue;
const taskEditorGraphPatchCommit = taskEditorGraph.patchInputs({
  draftValue: {
    status: "published",
  },
});
const taskEditorGraphSinglePatchCommit = taskEditorGraph.patchInput("draftValue", {
  title: "Ship package",
});
const taskEditorGraphApplyCommit = taskEditorGraph.apply({
  writes: {
    routeParams: {
      taskId: "task-8",
      workspaceId: "beta",
    },
  },
  patches: {
    draftValue: {
      title: "Ship package",
    },
  },
  commands: {},
});
const taskEditorGraphTransactionCommit = taskEditorGraph.transaction((tx) => {
  tx.set("serverValue", {
    id: "task-7",
    title: "Ship docs",
    status: "ready",
  });
});
const authorityGraphContract = authorityGraph.contract();
const authorityGraphOperationalContract = authorityGraph.operationalContract();
const authorityGraphReadOnlyAuthority =
  authorityGraphOperationalContract.authorities.serverValue.authority;
const authorityGraphImportedAuthority =
  authorityGraphOperationalContract.authorities.externalParams.authority;
const authorityGraphPatchId = authorityGraphOperationalContract.patches.draftValue;
const authorityGraphWriteCommit = authorityGraph.writeInputs({
  draftValue: {
    title: "Ready to ship",
  },
});
const authorityGraphSingleWriteCommit = authorityGraph.writeInput("draftValue", {
  title: "Reviewed",
});
const authorityGraphPatchCommit = authorityGraph.patchInputs({
  draftValue: {
    title: "Approved",
  },
});
const authorityGraphSinglePatchCommit = authorityGraph.patchInput("draftValue", {
  title: "Queued",
});
const authorityGraphResetCommit = authorityGraph.resetInputs(["draftValue"]);
const authorityGraphSingleResetCommit = authorityGraph.resetInput("draftValue");
const authorityGraphApplyCommit = authorityGraph.apply({
  writes: {
    draftValue: {
      title: "Ship package",
    },
  },
  commands: {},
});
const authorityGraphTransactionCommit = authorityGraph.transaction((tx) => {
  tx.set("draftValue", {
    title: "Queued",
  });
  tx.patch("draftValue", {
    title: "Queued",
  });
});
const authorityGraphInputId = authorityGraphContract.inputs.serverValue;
const authorityGraphImportedInputId = authorityGraphContract.inputs.externalParams;
const authorityGraphOutputId = authorityGraphContract.outputs.effectiveValue;
const authorityGraphDiagnosticsInputWhy =
  authorityGraph.inspectDiagnostics().inputs.externalParams.why;
const authorityGraphHistoryOutputReplay =
  authorityGraph.inspectHistory().outputs.effectiveValue.replay;
const taskEditorGraphInputWhy = taskEditorGraph.inspectDiagnostics().inputs.serverValue.why;
const taskEditorGraphOutputReplay = taskEditorGraph.inspectHistory().outputs.submitAvailability.replay;
const itemDetailGraphOutput = itemDetailGraph.output("submitReadiness");
const itemDetailGraphView = useOutputValue(itemDetailGraphOutput, store);
const pageModalPageInputId = pageModalGraph.contract().inputs.pageServerItemData;
const pageModalModalInputId = pageModalGraph.contract().inputs.modalServerItemData;
const pageModalPageOutputId = pageModalGraph.contract().outputs.pageEffectiveItemData;
const pageModalModalOutputId = pageModalGraph.contract().outputs.modalEffectiveItemData;
const countView = useSignalValue<number>(count, store);
const doubledView = useSignalValue<number>(doubled, store);
const diagnosticsView = useSignalsDiagnostics(store);

void delivered;
void callbackNodeIds;
void callbackHostCapabilityCompatibility;
void latestHostCapabilityEventKind;
void latestHostCapabilityQueuedCount;
void latestHostCapabilityDeniedIds;
void recentHostCapabilityEvents;
void hostCapabilityLineageDigest;
void hostCapabilityBreadthDigest;
void hostCapabilityLineageEntry;
void hostCapabilityBreadthFamily;
void hostCapabilityReadCount;
void hostCapabilityReevaluationCount;
void hostCapabilityCompatibilityDenialCount;
void hostCapabilityPollCount;
void hostCapabilityNoOpPollCount;
void visibilityState;
void visibilityMode;
void visibilityDescriptor;
void onlineState;
void onlineMode;
void onlineDescriptor;
void onlineFlag;
void clockTick;
void clockNow;
void clockDescriptor;
void persistedDraft;
void hostPersistence;
void persistenceLabel;
void persistenceCommit;
void persistenceValue;
void persistenceDescriptor;
void runtimeEnvelope;
void runtimeProof;
void restoredBranchId;
void snapshotExplanationRetention;
void checkpointImage;
void diagnosticGraph;
void proofVersion;
void exportedPolicyPreset;
void snapshotPolicyTier;
void snapshotReplayHead;
void history;
void specialist;
void currentBranch;
void branchReplay;
void branchSnapshot;
void branchEnvelope;
void branchProof;
void parityProof;
void artifactProof;
void replayHasCallback;
void specialistGraphProfile;
void specialistTouchedNodes;
void artifactParity;
void panelValue;
void panelView;
void itemDetailGraphRead;
void itemDetailGraphSummary;
void itemDetailGraphContract;
void itemDetailGraphOperationalContract;
void itemDetailGraphOperationalWriteId;
void itemDetailGraphOperationalPatchId;
void itemDetailGraphOperationalAuthority;
void itemDetailGraphOperationalRequest;
void itemDetailGraphWriteCommit;
void itemDetailGraphPatchCommit;
void itemDetailGraphResetCommit;
void itemDetailGraphApplyCommit;
void itemDetailGraphTransactionCommit;
void itemDetailGraphDiagnostics;
void itemDetailGraphHistory;
void itemDetailGraphCompatibility;
void itemDetailGraphDependency;
void itemDetailGraphContractSummary;
void itemDetailGraphContractDelta;
void itemDetailGraphContractInputId;
void itemDetailGraphExportDefinition;
void itemDetailGraphExportSnapshot;
void itemDetailGraphContractHistory;
void itemDetailGraphDiagnosticsInputWhy;
void itemDetailGraphHistoryInputReplay;
void itemDetailGraphHistoryDependency;
void itemDetailGraphHistoryContractSummary;
void itemDetailGraphCompatibilityContractInputId;
void restoredItemDetailGraph;
void restoredItemDetailGraphContract;
void restoredItemDetailGraphContractHistory;
void restoredItemDetailGraphRead;
void restoredItemDetailGraphInputs;
void taskEditorGraphOperationalContract;
void taskEditorGraphPatchId;
void taskEditorGraphPatchCommit;
void taskEditorGraphApplyCommit;
void taskEditorGraphTransactionCommit;
void authorityGraph;
void authorityGraphContract;
void authorityGraphOperationalContract;
void authorityGraphReadOnlyAuthority;
void authorityGraphImportedAuthority;
void authorityGraphPatchId;
void authorityGraphWriteCommit;
void authorityGraphPatchCommit;
void authorityGraphResetCommit;
void authorityGraphApplyCommit;
void authorityGraphTransactionCommit;
void authorityGraphInputId;
void authorityGraphImportedInputId;
void authorityGraphOutputId;
void authorityGraphDiagnosticsInputWhy;
void authorityGraphHistoryOutputReplay;
void itemDetailGraphOutput;
void itemDetailGraphView;
void pageModalGraph;
void pageModalPageInputId;
void pageModalModalInputId;
void pageModalPageOutputId;
void pageModalModalOutputId;
void countView;
void doubledView;
void diagnosticsView;
void auditUserLine.value();
void auditTaskLine.value();
void typedAuditAsyncWorkspaceDetailLine.value();
void typedAuditAsyncWorkspaceVersionsLine.value();
void emptyHostCapabilityPlan;
void auditOutput.value();
void storeInput.value();
void initialized;
void collectionShapeFactory;
void deliveryFactory;
void itemAspectFactory;
void patchFactory;
void valueSummaryFactory;
void viewportState;
void hostViewport;
void viewportLabel;
void viewportSize;
void viewportWidth;
void viewportHeight;
void viewportDescriptor;
`;
}
