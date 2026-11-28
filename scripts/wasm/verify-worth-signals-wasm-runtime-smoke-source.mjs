export function buildRuntimeSmokeSource(packageName) {
  return `import init, { clockCapability, createSignals, hostCapabilityPlan, onlineCapability, persistenceCapability, viewportCapability, visibilityCapability } from "${packageName}";
import * as reactApi from "${packageName}/react";

await init();
let visibilityState = "visible";
let visibilityListener = null;
let viewportState = { width: 1280, height: 720 };
let viewportListener = null;
let onlineState = "online";
let onlineListener = null;
let clockTick = 0;
let persistedDraft = { mode: "draft", revision: 1 };
const signals = await createSignals({
  deployment: "mainThreadCompatibility",
  hostCapabilities: hostCapabilityPlan({
    visibility: visibilityCapability({
      source: {
        current() {
          return visibilityState;
        },
        subscribe(listener) {
          visibilityListener = listener;
          return () => {
            visibilityListener = null;
          };
        },
      },
      compatibility: "LiveOnly",
    }),
    viewport: viewportCapability({
      source: {
        current() {
          return viewportState;
        },
        subscribe(listener) {
          viewportListener = listener;
          return () => {
            viewportListener = null;
          };
        },
      },
    }),
    online: onlineCapability({
      source: {
        current() {
          return onlineState;
        },
        subscribe(listener) {
          onlineListener = listener;
          return () => {
            onlineListener = null;
          };
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
const count = signals.input(1, { debugName: "count" });
const doubled = signals.computed(() => count() * 2, { debugName: "doubled" });
const visibleLabel = signals.computed(
  () => (signals.host.visibility?.isVisible() ? "visible" : "hidden"),
  { debugName: "visibleLabel" },
);
const viewportLabel = signals.computed(
  () => (signals.host.viewport?.width() ?? 0) + "x" + (signals.host.viewport?.height() ?? 0),
  { debugName: "viewportLabel" },
);
const onlineLabel = signals.computed(
  () => (signals.host.online?.isOnline() ? "online" : "offline"),
  { debugName: "onlineLabel" },
);
const clockLabel = signals.computed(
  () => (signals.host.clock?.now() ?? 0) + count(),
  { debugName: "clockLabel" },
);
const persistenceLabel = signals.computed(
  () => signals.host.persistence?.value().revision ?? 0,
  { debugName: "persistenceLabel" },
);
const localDraft = signals.input({
  title: "Ship docs",
  done: false,
}, { debugName: "localDraft" });
const name = signals.input("Ada", { debugName: "name" });
const displayLabel = signals.computed(
  () => name().toUpperCase(),
  { debugName: "displayLabel" },
);
const shippingOptions = signals.input([
  { id: "ground", label: "Ground" },
  { id: "air", label: "Air" },
], { debugName: "shippingOptions" });
const firstShippingOption = signals.linked(() => shippingOptions()[0], {
  debugName: "firstShippingOption",
});
const preservedShippingOption = signals.linked({
  source: () => shippingOptions(),
  computation: (options, previous) =>
    options.find((option) => option.id === previous?.value?.id) ?? options[0],
  debugName: "preservedShippingOption",
});
const firstShippingOptionInitial = firstShippingOption();
const namingGraph = signals.graph("naming", {
  inputs: {
    name,
  },
  outputs: {
    publicDisplayName: displayLabel,
  },
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
function createEditSessionController(namespace) {
  return namespace.controller(({ input, computed }) => {
    const serverItemData = input(null, { id: "serverItemData" });
    const draftEdits = input({}, { id: "draftEdits" });

    const effectiveItemData = computed(() => ({
      ...(serverItemData() ?? {}),
      ...(draftEdits() ?? {}),
    }), { id: "effectiveItemData" });

    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftEdits()).length > 0,
    }), { id: "dirtyState" });

    return {
      inputs: {
        serverItemData,
        draftEdits,
      },
      outputs: {
        effectiveItemData,
        dirtyState,
      },
    };
  });
}
function createWorkflowController(namespace, editSession) {
  return namespace.controller(({ computed }) => {
    const submitReadiness = computed(() => {
      const item = editSession.outputs.effectiveItemData();
      const dirty = editSession.outputs.dirtyState();

      return {
        enabled: dirty.isDirty && Boolean(item.workflow_target_state_id),
        targetStateId: item.workflow_target_state_id ?? null,
      };
    }, { id: "submitReadiness" });

    return {
      outputs: {
        submitReadiness,
      },
    };
  });
}
function createFormController(namespace) {
  return namespace.controller(({ input, computed }) => {
    const serverValue = input({
      id: "task-7",
      title: "Ship docs",
      status: "draft",
    }, { id: "serverValue" });
    const draftValue = input({
      title: "Ship docs",
      status: "ready",
    }, { id: "draftValue" });
    const effectiveValue = computed(() => ({
      ...serverValue(),
      ...draftValue(),
    }), { id: "effectiveValue" });
    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftValue()).length > 0,
    }), { id: "dirtyState" });
    const validation = computed(() => ({
      titleMissing: !effectiveValue().title,
    }), { id: "validation" });

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
function createResourceController(namespace, form) {
  return namespace.controller(({ input, computed }) => {
    const routeParams = input({
      taskId: "task-7",
      workspaceId: "alpha",
    }, { id: "routeParams" });
    const resourceQuery = computed(() => ({
      taskId: routeParams().taskId,
      workspaceId: routeParams().workspaceId,
      status: form.outputs.effectiveValue().status,
    }), { id: "resourceQuery" });
    const submitAvailability = computed(() => ({
      enabled: form.outputs.dirtyState().isDirty && !form.outputs.validation().titleMissing,
      taskId: resourceQuery().taskId,
    }), { id: "submitAvailability" });

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
function createAuthorityController(namespace) {
  return namespace.controller(({ input, computed, publicInput }) => {
    const serverValue = input({
      id: "task-7",
      title: "Ship docs",
    }, { id: "serverValue" });
    const draftValue = input({
      title: "Ship docs",
    }, { id: "draftValue" });
    const externalParams = input({
      taskId: "task-7",
    }, { id: "externalParams" });
    const effectiveValue = computed(() => ({
      ...serverValue(),
      ...draftValue(),
      taskId: externalParams().taskId,
    }), { id: "effectiveValue" });

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
const itemDetailGraph = signals.graph("itemDetail", (graph) => {
  const editSession = graph.controller("editSession", ({ input, computed }) => {
    const serverItemData = input(null, { id: "serverItemData" });
    const draftEdits = input({}, { id: "draftEdits" });
    const effectiveItemData = computed(() => ({
      ...(serverItemData() ?? {}),
      ...(draftEdits() ?? {}),
    }), { id: "effectiveItemData" });
    const dirtyState = computed(() => ({
      isDirty: Object.keys(draftEdits()).length > 0,
    }), { id: "dirtyState" });

    return {
      inputs: {
        serverItemData,
        draftEdits,
      },
      outputs: {
        effectiveItemData,
        dirtyState,
      },
    };
  });
  const workflow = createWorkflowController(graph.scope("workflow"), editSession);
  return graph.expose({
    controllers: [editSession, workflow],
  });
});
const pageModalGraph = signals.graph("itemWorkspace", (graph) => {
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
const repeatedRows = signals.scope("rows");
const row0 = repeatedRows.scope("row-0");
const row1 = repeatedRows.scope("row-1");
const row0Descriptor = row0.descriptor();
const row0Identity = row0.signalIdentity("count");
const row1Identity = row1.signalIdentity("count");
const row0Count = row0.input(0, { id: "count" });
const row1Count = row1.input(1, { id: "count" });
const taskEditorGraph = signals.graph("taskEditor", (graph) => {
  const form = createFormController(graph.scope("form"));
  const resource = createResourceController(graph.scope("resource"), form);
  return graph.expose({
    controllers: [form, resource],
  });
});
const authorityGraph = signals.graph("taskAuthority", (graph) => {
  const authority = createAuthorityController(graph.scope("authority"));
  return graph.expose({
    controllers: [authority],
  });
});
const linkedSelectionGraph = signals.graph("shippingSelection", (graph) => {
  const selection = graph.scope("selection");
  const available = selection.input([
    { id: "draft", label: "Draft" },
    { id: "review", label: "Review" },
  ]);
  const chosen = selection.linked({
    source: () => available(),
    computation: (options, previous) =>
      options.find((option) => option.id === previous?.value?.id) ?? options[0] ?? null,
    debugName: "chosenShippingOption",
  });
  return graph.expose({
    inputs: {
      available,
      chosen,
    },
    outputs: {
      chosen,
    },
  });
});
const linkedRevisionGraph = signals.graph("linkedRevisionSelection", (graph) => {
  const selection = graph.scope("selection");
  const available = selection.input({
    revision: 1,
    options: [
      { id: "draft", label: "Draft" },
      { id: "review", label: "Review" },
    ],
  });
  const chosen = selection.linked({
    source: () => available(),
    computation: (source, previous) => {
      const preserved = previous && previous.source.revision === source.revision
        ? source.options.find((option) => option.id === previous.value?.id) ?? null
        : null;
      return preserved ?? source.options[0] ?? null;
    },
    debugName: "chosenRevisionOption",
  });
  return graph.expose({
    inputs: {
      available,
      chosen,
    },
    outputs: {
      chosen,
    },
  });
});
  visibleLabel();
  viewportLabel();
  onlineLabel();
  clockLabel();
  viewportState = { width: 1440, height: 900 };
  viewportListener?.();
  visibilityState = "hidden";
  visibilityListener?.();
  onlineState = "offline";
  onlineListener?.();
  clockTick = 5;
  await new Promise((resolve) => setTimeout(resolve, 15));
  persistedDraft = { mode: "draft", revision: 2 };
  await Promise.resolve(signals.host.persistence?.commit());
  await Promise.resolve(signals.host.persistence?.commit());
await Promise.resolve(signals.transaction((tx) => {
  tx.set(count, 2);
}));
await Promise.resolve(localDraft.patch({
  done: true,
}));
await Promise.resolve(localDraft.assign({
  title: "Ready to ship",
}));
await Promise.resolve(signals.transaction((tx) => {
  tx.patch(localDraft, {
    status: "queued",
  });
}));
await Promise.resolve(preservedShippingOption.set({ id: "air", label: "Air" }));
await Promise.resolve(shippingOptions.set([
  { id: "ground", label: "Ground" },
  { id: "air", label: "Air" },
  { id: "sea", label: "Sea" },
]));
const preservedShippingAfterSourceChange = preservedShippingOption();
await Promise.resolve(preservedShippingOption.relink());
const preservedShippingAfterRelink = preservedShippingOption();
await Promise.resolve(preservedShippingOption.set({ id: "manual", label: "Manual" }));
await Promise.resolve(preservedShippingOption.reset());
const preservedShippingAfterReset = preservedShippingOption();
await Promise.resolve(shippingOptions.set([
  { id: "sea", label: "Sea" },
  { id: "ground", label: "Ground" },
]));
await Promise.resolve(firstShippingOption.set({ id: "manual", label: "Manual" }));
await Promise.resolve(firstShippingOption.reset());
const firstShippingAfterReset = firstShippingOption();
await Promise.resolve(firstShippingOption.relink());
const firstShippingAfterRelink = firstShippingOption();
await Promise.resolve(preservedShippingOption.set({ id: "manual", label: "Manual" }));
await Promise.resolve(preservedShippingOption.relink());
const preservedShippingAfterFallbackRelink = preservedShippingOption();
const history = signals.history();
const branch = history.current_branch();
const replay = history.replay_for_branch(branch.id);
const snapshot = history.snapshot();
const branchSnapshot = history.branch_snapshot(branch.id);
const branchEnvelope = history.branch_snapshot_envelope(branch.id);
const adapters = signals.adapters();
const runtimeEnvelope = adapters.exportRuntimeEnvelope();
const transportReport = adapters.hostCapabilityTransportReport(runtimeEnvelope);
const restoredExact = await createSignals({ deployment: "mainThreadCompatibility" });
await restoredExact.adapters().restoreExactRuntimeEnvelope(runtimeEnvelope);
const portableImport = await createSignals({ deployment: "mainThreadCompatibility" });
let portableImportError = null;
try {
  await portableImport.adapters().replaceRuntimeEnvelope(runtimeEnvelope);
} catch (error) {
  portableImportError = {
    code: error?.code ?? null,
    message: error?.message ?? String(error),
  };
}
const portableImportDiagnostics = portableImport.diagnostics();
const portableImportLatestHostCapabilityEvent =
  portableImportDiagnostics.latestHostCapabilityEvent();
const portableImportRecentHostCapabilityEvents =
  portableImportDiagnostics.recentHostCapabilityEvents();
const portableImportHostCapabilityReport =
  portableImportDiagnostics.hostCapabilityReport();
const portableImportDeniedCallbackIds =
  portableImportRecentHostCapabilityEvents.flatMap((event) => event?.deniedCallbackIds ?? []);
const portableImportPerformanceSummary = portableImportDiagnostics.performanceSummary();
const unavailableVisibleLabel =
  runtimeEnvelope.definitions.unavailableCallbacks.find((artifact) => artifact.id === "visibleLabel");
const unavailableViewportLabel =
  runtimeEnvelope.definitions.unavailableCallbacks.find((artifact) => artifact.id === "viewportLabel");
const unavailableOnlineLabel =
  runtimeEnvelope.definitions.unavailableCallbacks.find((artifact) => artifact.id === "onlineLabel");
const unavailableClockLabel =
  runtimeEnvelope.definitions.unavailableCallbacks.find((artifact) => artifact.id === "clockLabel");
const unavailablePersistenceLabel =
  runtimeEnvelope.definitions.unavailableCallbacks.find((artifact) => artifact.id === "persistenceLabel");
const specialist = signals.specialist();
const specialistGraphSummary = specialist.graphSummary();
const specialistEvaluateDirty = specialist.evaluateDirty();
const diagnostics = signals.diagnostics();
const performanceSummary = diagnostics.performanceSummary();
const hostCapabilityReport = diagnostics.hostCapabilityReport();
const latestHostCapabilityEvent = diagnostics.latestHostCapabilityEvent();
const recentHostCapabilityEvents = diagnostics.recentHostCapabilityEvents();
const itemDetailGraphSummary = itemDetailGraph.summary();
const itemDetailGraphRead = itemDetailGraph.read();
const itemDetailGraphInputs = itemDetailGraph.readInputs();
const itemDetailGraphOutputId = itemDetailGraph.output("submitReadiness").id;
const itemDetailGraphContract = itemDetailGraph.contract();
const itemDetailGraphOperationalContract = itemDetailGraph.operationalContract();
await Promise.resolve(itemDetailGraph.writeInputs({
  serverItemData: {
    workflow_target_state_id: "ready",
  },
}));
await Promise.resolve(itemDetailGraph.writeInput("serverItemData", {
  workflow_target_state_id: "review",
}));
await Promise.resolve(itemDetailGraph.patchInputs({
  draftEdits: {
    title: "Ship docs",
  },
}));
await Promise.resolve(itemDetailGraph.patchInput("draftEdits", {
  queued: true,
}));
await Promise.resolve(itemDetailGraph.transaction((tx) => {
  tx.set("draftEdits", {
    title: "Ready to ship",
    workflow_target_state_id: "ready",
  });
  tx.patch("draftEdits", {
    staged: true,
  });
}));
await Promise.resolve(itemDetailGraph.apply({
  writes: {
    serverItemData: {
      workflow_target_state_id: "done",
    },
  },
  patches: {
    draftEdits: {
      reviewState: "approved",
    },
  },
  commands: {},
}));
const itemDetailGraphOperationalSnapshot = itemDetailGraph.readInputs();
itemDetailGraph.resetInputs();
const itemDetailGraphResetSnapshot = itemDetailGraph.readInputs();
const itemDetailGraphDiagnostics = itemDetailGraph.inspectDiagnostics();
const itemDetailGraphHistory = itemDetailGraph.inspectHistory();
const itemDetailGraphCompatibility = itemDetailGraph.exportCompatibilityDefinition();
const itemDetailGraphExportDefinition = itemDetailGraph.exportDefinition();
const itemDetailGraphExportSnapshot = itemDetailGraph.exportSnapshot();
const itemDetailGraphImportPosture = itemDetailGraph.importPosture();
const importedItemDetailGraph = (
  await createSignals({ deployment: "mainThreadCompatibility" })
).importGraph(
  itemDetailGraphExportDefinition,
  itemDetailGraphExportSnapshot,
);
const importedItemDetailGraphContract = importedItemDetailGraph.contract();
const importedItemDetailGraphContractHistory = importedItemDetailGraph.contractHistory();
const importedItemDetailGraphImportPosture = importedItemDetailGraph.importPosture();
const importedItemDetailGraphRead = importedItemDetailGraph.read();
const importedItemDetailGraphInputs = importedItemDetailGraph.readInputs();
const itemDetailGraphDependency =
  itemDetailGraphDiagnostics.dependenciesForOutput("submitReadiness");
const itemDetailGraphContractSummary = itemDetailGraphDiagnostics.contractSummary();
const itemDetailGraphContractDelta = itemDetailGraph.contractDelta({
  ...itemDetailGraphContract,
  outputs: {},
});
const taskEditorGraphContract = taskEditorGraph.contract();
const taskEditorGraphOperationalContract = taskEditorGraph.operationalContract();
await Promise.resolve(taskEditorGraph.patchInputs({
  draftValue: {
    status: "published",
  },
}));
await Promise.resolve(taskEditorGraph.patchInput("draftValue", {
  title: "Ship package",
}));
await Promise.resolve(taskEditorGraph.apply({
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
}));
const taskEditorGraphDiagnostics = taskEditorGraph.inspectDiagnostics();
const taskEditorGraphHistory = taskEditorGraph.inspectHistory();
const taskEditorGraphCompatibility = taskEditorGraph.exportCompatibilityDefinition();
const authorityGraphContract = authorityGraph.contract();
const authorityGraphOperationalContract = authorityGraph.operationalContract();
await Promise.resolve(authorityGraph.writeInputs({
  draftValue: {
    title: "Ready to ship",
  },
}));
await Promise.resolve(authorityGraph.writeInput("draftValue", {
  title: "Reviewed",
}));
await Promise.resolve(authorityGraph.patchInputs({
  draftValue: {
    title: "Approved",
  },
}));
await Promise.resolve(authorityGraph.patchInput("draftValue", {
  status: "queued",
}));
await Promise.resolve(authorityGraph.transaction((tx) => {
  tx.set("draftValue", {
    title: "Queued",
  });
  tx.patch("draftValue", {
    status: "queued",
  });
}));
await Promise.resolve(linkedSelectionGraph.writeInputs({
  chosen: { id: "review", label: "Review" },
}));
await Promise.resolve(linkedSelectionGraph.writeInputs({
  available: [
    { id: "ready", label: "Ready" },
    { id: "review", label: "Review" },
  ],
}));
await Promise.resolve(linkedSelectionGraph.resetInputs(["chosen"]));
const linkedSelectionAfterGraphReset = linkedSelectionGraph.readInputs().chosen;
await Promise.resolve(linkedRevisionGraph.writeInputs({
  available: {
    revision: 2,
    options: [
      { id: "review", label: "Review" },
      { id: "ready", label: "Ready" },
    ],
  },
}));
await Promise.resolve(linkedRevisionGraph.resetInputs(["chosen"]));
await Promise.resolve(linkedRevisionGraph.writeInputs({
  available: {
    revision: 2,
    options: [
      { id: "approved", label: "Approved" },
      { id: "review", label: "Review" },
    ],
  },
}));
await Promise.resolve(linkedRevisionGraph.resetInputs(["chosen"]));
const linkedRevisionAfterSecondGraphReset = linkedRevisionGraph.readInputs().chosen;
const authorityGraphRead = authorityGraph.read();
const authorityGraphInputs = authorityGraph.readInputs();
const authorityGraphDiagnostics = authorityGraph.inspectDiagnostics();
const authorityGraphHistory = authorityGraph.inspectHistory();
const authorityGraphCompatibility = authorityGraph.exportCompatibilityDefinition();

const summary = {
  hasInit: typeof init === "function",
  hasCreateSignals: typeof createSignals === "function",
  reactKeys: Object.keys(reactApi).sort(),
  doubled: doubled(),
  visibleLabel: visibleLabel(),
  viewportLabel: viewportLabel(),
  onlineLabel: onlineLabel(),
  clockLabel: clockLabel(),
  persistenceLabel: persistenceLabel(),
  visibilityState: signals.host.visibility?.state() ?? null,
  visibilityCompatibility: signals.host.visibility?.descriptor().compatibility ?? null,
  viewportSize: signals.host.viewport?.size() ?? null,
  viewportCompatibility: signals.host.viewport?.descriptor().compatibility ?? null,
  onlineState: signals.host.online?.state() ?? null,
  onlineCompatibility: signals.host.online?.descriptor().compatibility ?? null,
  clockNow: signals.host.clock?.now() ?? null,
  clockCompatibility: signals.host.clock?.descriptor().compatibility ?? null,
  persistenceValue: signals.host.persistence?.value() ?? null,
  persistenceCompatibility: signals.host.persistence?.descriptor().compatibility ?? null,
  localDraftValue: localDraft(),
  nameValue: name(),
  nameOpaqueId: name.id,
  displayLabelOpaqueId: displayLabel.id,
  firstShippingOptionId: firstShippingOptionInitial.id,
  firstShippingOptionDebugName: firstShippingOption.debugName,
  preservedShippingAfterSourceChangeId: preservedShippingAfterSourceChange?.id ?? null,
  preservedShippingAfterRelinkId: preservedShippingAfterRelink?.id ?? null,
  preservedShippingAfterResetId: preservedShippingAfterReset?.id ?? null,
  firstShippingAfterResetId: firstShippingAfterReset?.id ?? null,
  firstShippingAfterRelinkId: firstShippingAfterRelink?.id ?? null,
  preservedShippingAfterFallbackRelinkId: preservedShippingAfterFallbackRelink?.id ?? null,
  linkedSelectionAfterGraphResetId: linkedSelectionAfterGraphReset?.id ?? null,
  linkedRevisionAfterSecondGraphResetId: linkedRevisionAfterSecondGraphReset?.id ?? null,
  namingGraphInputId: namingGraph.contract().inputs.name,
  namingGraphOutputId: namingGraph.output("publicDisplayName").id,
  namingGraphDescriptor: namingGraph.descriptors()[0],
  namingGraphCompatibilityOutputId:
    namingGraph.exportCompatibilityDefinition().contract.outputs.publicDisplayName,
  requirednessInputDescriptors: requirednessGraph.inputDescriptors(),
  requirednessServerAuthority:
    requirednessGraph.operationalContract().authorities.serverValue,
  requirednessDraftAuthority:
    requirednessGraph.operationalContract().authorities.draftValue,
  runtimeEnvelopeRestoreMode: runtimeEnvelope.runtimeEnvelopeRestoreMode,
  runtimeEnvelopeUnavailableHostCapabilityCompatibility:
    unavailableVisibleLabel?.hostCapabilityReads[0]?.compatibility ?? null,
  runtimeEnvelopeUnavailablePortableOutcome:
    unavailableVisibleLabel?.hostCapabilityTransports[0]?.portableImportOutcome ?? null,
  runtimeEnvelopeUnavailableExactRestoreOutcome:
    unavailableVisibleLabel?.hostCapabilityTransports[0]?.exactRestoreOutcome ?? null,
  runtimeEnvelopeUnavailablePortableReason:
    unavailableVisibleLabel?.hostCapabilityTransports[0]?.portableImportReason ?? null,
  runtimeEnvelopeUnavailableOnlineCompatibility:
    unavailableOnlineLabel?.hostCapabilityReads[0]?.compatibility ?? null,
  runtimeEnvelopeUnavailableViewportCompatibility:
    unavailableViewportLabel?.hostCapabilityReads[0]?.compatibility ?? null,
  runtimeEnvelopeUnavailableViewportPortableOutcome:
    unavailableViewportLabel?.hostCapabilityTransports[0]?.portableImportOutcome ?? null,
  runtimeEnvelopeUnavailableViewportPortableReason:
    unavailableViewportLabel?.hostCapabilityTransports[0]?.portableImportReason ?? null,
  runtimeEnvelopeUnavailableOnlinePortableOutcome:
    unavailableOnlineLabel?.hostCapabilityTransports[0]?.portableImportOutcome ?? null,
  runtimeEnvelopeUnavailableOnlinePortableReason:
    unavailableOnlineLabel?.hostCapabilityTransports[0]?.portableImportReason ?? null,
  runtimeEnvelopeUnavailableClockCompatibility:
    unavailableClockLabel?.hostCapabilityReads[0]?.compatibility ?? null,
  runtimeEnvelopeUnavailableClockPortableOutcome:
    unavailableClockLabel?.hostCapabilityTransports[0]?.portableImportOutcome ?? null,
  runtimeEnvelopeUnavailableClockPortableReason:
    unavailableClockLabel?.hostCapabilityTransports[0]?.portableImportReason ?? null,
  runtimeEnvelopeUnavailablePersistenceCompatibility:
    unavailablePersistenceLabel?.hostCapabilityReads[0]?.compatibility ?? null,
  runtimeEnvelopeUnavailablePersistencePortableOutcome:
    unavailablePersistenceLabel?.hostCapabilityTransports[0]?.portableImportOutcome ?? null,
  runtimeEnvelopeUnavailableCallbackCount:
    runtimeEnvelope.definitions.unavailableCallbacks.length,
  runtimeEnvelopeUnavailableCurrentReads: unavailableVisibleLabel?.currentReads ?? [],
  latestHostCapabilityEventKind: latestHostCapabilityEvent?.kind ?? null,
  latestHostCapabilityEventQueuedCount: latestHostCapabilityEvent?.queuedInvalidationCount ?? null,
  recentHostCapabilityEventCount: recentHostCapabilityEvents.length,
  hostCapabilityReadCount: performanceSummary.hostCapabilityReadCount ?? null,
  hostCapabilityPollCount: performanceSummary.hostCapabilityPollCount ?? null,
  hostCapabilityNoOpPollCount: performanceSummary.hostCapabilityNoOpPollCount ?? null,
  hostCapabilityManualCommitCount: performanceSummary.hostCapabilityManualCommitCount ?? null,
  hostCapabilityNoOpManualCommitCount: performanceSummary.hostCapabilityNoOpManualCommitCount ?? null,
  hostCapabilityInvalidationCount: performanceSummary.hostCapabilityInvalidationCount ?? null,
  hostCapabilityInvalidationBatchFlushCount: performanceSummary.hostCapabilityInvalidationBatchFlushCount ?? null,
  hostCapabilityReevaluationCount: performanceSummary.hostCapabilityReevaluationCount ?? null,
  hostCapabilityInvalidationTouchedNodeCount: performanceSummary.hostCapabilityInvalidationTouchedNodeCount ?? null,
  hostCapabilityCompatibilityDenialCount: performanceSummary.hostCapabilityCompatibilityDenialCount ?? null,
  hostCapabilityUnavailabilityArtifactCount: performanceSummary.hostCapabilityUnavailabilityArtifactCount ?? null,
  hostCapabilityBroadFanoutDenialCount: performanceSummary.hostCapabilityBroadFanoutDenialCount ?? null,
  hostCapabilityReportDigest: hostCapabilityReport.digest,
  hostCapabilityReportLineageDigest: hostCapabilityReport.lineageDigest,
  hostCapabilityReportBreadthDigest: hostCapabilityReport.breadthDigest,
  hostCapabilityReportFamilyCount: hostCapabilityReport.families.length,
  hostCapabilityReportLineageCount: hostCapabilityReport.lineage.length,
  hostCapabilityReportMaxTouchedNodes: hostCapabilityReport.breadth.maxTouchedNodes,
  hostCapabilityReportMaxReevaluatedNodes: hostCapabilityReport.breadth.maxReevaluatedNodes,
  transportReportDigest: transportReport.digest,
  transportReportUnavailableArtifactCount: transportReport.totals.unavailableArtifactCount,
  transportReportDeniedFamilyCount: transportReport.totals.deniedFamilyCount,
  transportReportUnavailableFamilyCount: transportReport.totals.unavailableFamilyCount,
  portableImportLatestHostCapabilityEventKind:
    portableImportLatestHostCapabilityEvent?.kind ?? null,
  portableImportLatestHostCapabilityEventQueuedCount:
    portableImportLatestHostCapabilityEvent?.queuedInvalidationCount ?? null,
  portableImportLatestHostCapabilityEventDeniedIds:
    portableImportLatestHostCapabilityEvent?.deniedCallbackIds ?? [],
  portableImportDeniedCallbackIds,
  portableImportRecentHostCapabilityEventCount:
    portableImportRecentHostCapabilityEvents.length,
  portableImportHostCapabilityCompatibilityDenialCount:
    portableImportPerformanceSummary.hostCapabilityCompatibilityDenialCount ?? null,
  portableImportHostCapabilityReportDigest:
    portableImportHostCapabilityReport.digest,
  portableImportHostCapabilityReportLineageDigest:
    portableImportHostCapabilityReport.lineageDigest,
  portableImportHostCapabilityReportFamilyCount:
    portableImportHostCapabilityReport.families.length,
  branchIdType: typeof branch.id,
  replayFrameCount: replay.frames.length,
  replayHasCallback: replay.frames.some((frame) => frame.callback?.id === doubled.id),
  snapshotBranchId: snapshot.snapshot.meta.branch_id,
  branchSnapshotBranchId: branchSnapshot.meta.branch_id,
  branchSnapshotRestoreMode: branchSnapshot.snapshotRestoreMode,
  branchEnvelopeRestoreMode: branchEnvelope.snapshotEnvelopeRestoreMode,
  exportedPolicyPreset: runtimeEnvelope.definitions.policy.preset,
  snapshotPolicyTier: snapshot.snapshot.meta.runtime_policy.tier,
  snapshotReplayHead: snapshot.snapshot.meta.replay_head,
  snapshotExplanationRetention: snapshot.snapshot.meta.artifact_retention.explanation_retention,
  restoredExactDoubled: restoredExact.read(doubled.id),
  portableImportErrorCode: portableImportError?.code ?? null,
  portableImportErrorMessage: portableImportError?.message ?? null,
  specialistGraphProfile: specialistGraphSummary.profile,
  specialistTouchedNodes: specialistEvaluateDirty.touchedNodes,
  itemDetailGraphId: itemDetailGraphSummary.id,
  itemDetailGraphInputNames: itemDetailGraphSummary.inputNames,
  itemDetailGraphOutputNames: itemDetailGraphSummary.outputNames,
  itemDetailGraphInputKeys: Object.keys(itemDetailGraphInputs).sort(),
  itemDetailGraphReadKeys: Object.keys(itemDetailGraphRead).sort(),
  itemDetailGraphSubmitOutputId: itemDetailGraphOutputId,
  itemDetailGraphOperationalWriteId:
    itemDetailGraphOperationalContract.writes.serverItemData,
  itemDetailGraphOperationalPatchId:
    itemDetailGraphOperationalContract.patches.draftEdits,
  itemDetailGraphOperationalAuthority:
    itemDetailGraphOperationalContract.authorities.draftEdits.authority,
  itemDetailGraphOperationalSupportsPatch:
    itemDetailGraphOperationalContract.authorities.draftEdits.supportsPatch,
  itemDetailGraphOperationalServerState:
    itemDetailGraphOperationalSnapshot.serverItemData?.workflow_target_state_id ?? null,
  itemDetailGraphOperationalDraftTitle:
    itemDetailGraphOperationalSnapshot.draftEdits?.title ?? null,
  itemDetailGraphOperationalDraftReviewState:
    itemDetailGraphOperationalSnapshot.draftEdits?.reviewState ?? null,
  itemDetailGraphResetDraftKeys:
    Object.keys(itemDetailGraphResetSnapshot.draftEdits ?? {}).sort(),
  itemDetailGraphResetServerValue:
    itemDetailGraphResetSnapshot.serverItemData,
  row0ScopePath: row0Descriptor.path.map((segment) => segment.id),
  row0ScopeParent: row0Descriptor.identity.parentScopeId,
  row0SignalCanonicalId: row0Identity.canonicalId,
  row0SignalGraphId: row0Identity.graphId,
  row0SignalRootScopeId: row0Identity.rootScopeId,
  row1SignalCanonicalId: row1Identity.canonicalId,
  row0HandleId: row0Count.id,
  row1HandleId: row1Count.id,
  itemDetailGraphContractInputId: itemDetailGraphContract.inputs.serverItemData,
  itemDetailGraphWhyId: itemDetailGraphDiagnostics.outputs.submitReadiness.why.id,
  itemDetailGraphInputWhyId: itemDetailGraphDiagnostics.inputs.serverItemData.why.id,
  itemDetailGraphReplayFrameCount:
    itemDetailGraphHistory.outputs.submitReadiness.replay.frames.length,
  itemDetailGraphInputReplayFrameCount:
    itemDetailGraphHistory.inputs.serverItemData.replay.frames.length,
  itemDetailGraphCompatibilityInputId:
    itemDetailGraphCompatibility.inputs.serverItemData,
  itemDetailGraphCompatibilityContractInputId:
    itemDetailGraphCompatibility.contract.inputs.serverItemData,
  itemDetailGraphCompatibilityOutputId:
    itemDetailGraphCompatibility.outputs.submitReadiness,
  itemDetailGraphDependencyInputNames:
    [...itemDetailGraphDependency.publicInputNames].sort(),
  itemDetailGraphDependencySourceIds:
    [...itemDetailGraphDependency.publicInputSourceIds].sort(),
  itemDetailGraphContractSummaryOutputCount:
    itemDetailGraphContractSummary.outputCount,
  itemDetailGraphContractDeltaAddedOutputs:
    itemDetailGraphContractDelta.outputs.added,
  importedItemDetailGraphOutputId:
    importedItemDetailGraph.exportCompatibilityDefinition().outputs.submitReadiness,
  importedItemDetailGraphReadiness:
    importedItemDetailGraphRead.submitReadiness?.targetStateId ?? null,
  importedItemDetailGraphInputCount:
    importedItemDetailGraphInputs.serverItemData?.workflow_target_state_id ?? null,
  importedItemDetailGraphContractInputId:
    importedItemDetailGraphContract.inputs.serverItemData,
  importedItemDetailGraphHistoryChanged:
    importedItemDetailGraphContractHistory.changedSinceBaseline,
  importedItemDetailGraphHistoryRestoreMode:
    importedItemDetailGraphContractHistory.restoreMode,
  itemDetailGraphImportPortableMode:
    itemDetailGraphImportPosture.portableImport,
  itemDetailGraphImportHydrateMode:
    itemDetailGraphImportPosture.hydrate,
  importedItemDetailGraphExactRestoreMode:
    importedItemDetailGraphImportPosture.exactRestoreMode,
  pageModalPageInputId: pageModalGraph.contract().inputs.pageServerItemData,
  pageModalModalInputId: pageModalGraph.contract().inputs.modalServerItemData,
  pageModalPageOutputId: pageModalGraph.contract().outputs.pageEffectiveItemData,
  pageModalModalOutputId: pageModalGraph.contract().outputs.modalEffectiveItemData,
  taskEditorGraphInputId: taskEditorGraphContract.inputs.serverValue,
  taskEditorGraphRouteParamsId: taskEditorGraphContract.inputs.routeParams,
  taskEditorGraphOutputId: taskEditorGraphContract.outputs.submitAvailability,
  taskEditorGraphPatchId: taskEditorGraphOperationalContract.patches.draftValue,
  taskEditorGraphPatchedStatus:
    taskEditorGraph.readInputs().draftValue.status ?? null,
  taskEditorGraphRouteParamTaskId:
    taskEditorGraph.readInputs().routeParams.taskId,
  taskEditorGraphInputWhyId: taskEditorGraphDiagnostics.inputs.routeParams.why.id,
  taskEditorGraphOutputWhyId: taskEditorGraphDiagnostics.outputs.submitAvailability.why.id,
  taskEditorGraphInputReplayFrameCount:
    taskEditorGraphHistory.inputs.routeParams.replay.frames.length,
  taskEditorGraphOutputReplayFrameCount:
    taskEditorGraphHistory.outputs.submitAvailability.replay.frames.length,
  taskEditorGraphCompatibilityInputId:
    taskEditorGraphCompatibility.contract.inputs.routeParams,
  taskEditorGraphCompatibilityOutputId:
    taskEditorGraphCompatibility.contract.outputs.submitAvailability,
  authorityGraphInputId: authorityGraphContract.inputs.serverValue,
  authorityGraphOutputId: authorityGraphContract.outputs.effectiveValue,
  authorityGraphWritablePatchId: authorityGraphOperationalContract.patches.draftValue,
  authorityGraphReadOnlyAuthority:
    authorityGraphOperationalContract.authorities.serverValue.authority,
  authorityGraphImportedAuthority:
    authorityGraphOperationalContract.authorities.externalParams.authority,
  authorityGraphDraftTitle: authorityGraphInputs.draftValue.title ?? null,
  authorityGraphTaskId: authorityGraphRead.effectiveValue.taskId ?? null,
  authorityGraphInputWhyId: authorityGraphDiagnostics.inputs.externalParams.why.id,
  authorityGraphOutputReplayFrameCount:
    authorityGraphHistory.outputs.effectiveValue.replay.frames.length,
  authorityGraphCompatibilityInputId:
    authorityGraphCompatibility.contract.inputs.externalParams,
};

console.log(JSON.stringify(summary));
`;
}
