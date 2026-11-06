import type {
  CallableSignalsTransaction, AsyncCallableSignalsTransaction,
} from "./callable/authoring.js";
export type {
  CallableSignalsTransaction, AsyncCallableSignalsTransaction,
} from "./callable/authoring.js";
import type {
  AspectId, VersionSummary, ComputedSpec, OutputSpec, RunSummary, SignalValue,
  WebObservationNotice,
} from "./model.js";
import type { HostCapabilityDiagnosticsReport, GraphSummary } from "./diagnostics.js";
import type {
  ExportedSignalGraphDefinition, ExportedSignalGraphSnapshot, GraphBuilder, GraphInputDefinitions,
  GraphOutputDefinitions, GraphPublicationRequest, GraphScope, ImportedSignalGraph,
  PublicGraphInputContractEntry, PublicGraphInputOptions, PublishedSignalGraph,
} from "./graph_surface.js";
import type {
  ControllerContract, ControllerContractBuilder, ControllerContractDefinition,
} from "./controller_surface.js";
import type { FormFactory } from "./forms_surface.js";
import type { RouterNamespace } from "./router_surface.js";
import type {
  ComputedSignal, DisposableHandle, InputSignal, OutputSignal, SignalAdapters, SignalApp,
  SignalDiagnostics, SignalHistory, SignalRuntime, SignalSpecialist, Signals,
} from "./raw_surface.js";
import type {
  InputSignalHandle, ComputedSignalHandle, OutputSignalHandle, ExplicitSignalSpecNamespace,
  InputAuthoringOptions, LinkedSignalOptions, LinkedSignalHandle, LinkedIdentitySignalDefinition,
  LinkedComputedSignalDefinition, NamedComputedCallbackDefinition, SignalAuthoringOptions,
  CallbackSignalAuthoringOptions, NamedOutputCallbackDefinition, Signal,
} from "./callable/authoring.js";
import type { CallableSignalsHost } from "./callable/host_capabilities.js";
import type {
  CallableSignalHistory, CallableSignalAdapters,
} from "./callable/runtime_artifacts.js";
import type {
  SignalsRuntimeContract, SignalsCompatibilityAssertionOptions,
} from "./callable/construction.js";
export type {
  Signal, NamedComputedCallbackDefinition, NamedOutputCallbackDefinition, InputSignalHandle,
  LinkedSignalPrevious, LinkedSignalOptions, LinkedIdentitySignalDefinition,
  LinkedComputedSignalDefinition, LinkedSignalDefinition, LinkedSignalHandle,
  ComputedSignalHandle, OutputSignalHandle, InputAuthoringOptions, SignalAuthoringOptions,
  CallbackSignalAuthoringOptions, ExplicitSignalSpecNamespace,
} from "./callable/authoring.js";
export type {
  HostCapabilitySubscription, ViewportCapabilityState, ViewportCapabilitySource,
  ViewportCapabilityOptions, ViewportCapabilityRegistration, VisibilityCapabilitySource,
  VisibilityCapabilityOptions, VisibilityCapabilityRegistration, OnlineCapabilitySource,
  OnlineCapabilityOptions, OnlineCapabilityRegistration, ClockCapabilitySource,
  ClockCapabilityOptions, ClockCapabilityRegistration, PersistenceCapabilitySource,
  PersistenceCapabilityOptions, PersistenceCapabilityRegistration, HostCapabilityPlanInput,
  HostCapabilityPlan, ViewportCapabilityDescriptor, HostViewportCapability,
  VisibilityCapabilityDescriptor, HostVisibilityCapability, OnlineCapabilityDescriptor,
  HostOnlineCapability, ClockCapabilityDescriptor, HostClockCapability,
  PersistenceCapabilityDescriptor, HostPersistenceCapability, CallableSignalsHost,
} from "./callable/host_capabilities.js";
export {
  viewportCapability, visibilityCapability, onlineCapability, clockCapability,
  persistenceCapability, hostCapabilityPlan,
} from "./callable/host_capabilities.js";
export type {
  SignalsDeployment, CreateSignalsAssets, CreateSignalsOptions, WrapSignalsOptions,
  SignalsCompatibilityRecovery, SignalsRuntimeCapabilityName, SignalsRuntimeSurfaceFamily,
  SignalsRuntimeCapabilities, SignalsRuntimeContract, SignalsCompatibilityAssertionOptions,
  SignalsConstructionArtifact, SignalsConstructionExplanation,
} from "./callable/construction.js";
export {
  explainCreateSignalsConstruction, planCreateSignalsDeployment, createSignals,
  createCallableSignals, wrapSignals,
} from "./callable/construction.js";
export type {
  CallableSignalAdapters, CallableBranchId, RuntimeEnvelopeArtifact,
  RuntimeSnapshotEnvelopeArtifact, RuntimeSnapshotArtifactWithWire, CallableSignalHistory,
} from "./callable/runtime_artifacts.js";

/** Observation consumes branded readable identity, independently of its value type. */
export type CallableSignalTarget = string | Signal<unknown>;

export type CallableSignalDiagnostics =
  SignalDiagnostics & {
    hostCapabilityReport(): HostCapabilityDiagnosticsReport;
  };

export interface CallableSignalSpecialist {
  evaluateDirty(): RunSummary | Promise<RunSummary>;
  evaluate_dirty(): RunSummary | Promise<RunSummary>;
  graphSummary(): GraphSummary;
  graph_summary(): GraphSummary;
  readVersions(ids: ReadonlyArray<string>): ReadonlyArray<VersionSummary>;
  read_versions(ids: ReadonlyArray<string>): ReadonlyArray<VersionSummary>;
  free(): void;
  [Symbol.dispose](): void;
}

export interface ScopeDescriptor {
  readonly id: string;
  readonly localScopeId: string;
  readonly parentScopeId: string | null;
  readonly depth: number;
  readonly path: ReadonlyArray<ScopePathSegment>;
  readonly identity: ScopeIdentity;
  readonly graphOwnerId?: string | null;
}

export interface ScopePathSegment {
  readonly id: string;
  readonly localScopeId: string;
  readonly depth: number;
}

export interface ScopeIdentity {
  readonly scopeId: string;
  readonly parentScopeId: string | null;
  readonly path: ReadonlyArray<ScopePathSegment>;
  readonly depth: number;
}

export interface ScopedSignalIdentity {
  readonly localId: string;
  readonly canonicalId: string;
  readonly scopeId: string;
  readonly graphOwnerId: string | null;
  readonly graphId: string | null;
  readonly rootScopeId: string | null;
  readonly scopePath: ReadonlyArray<ScopePathSegment>;
}

export interface ScopedSignalNamespace<TPersistence = SignalValue> {
  readonly host: CallableSignalsHost<TPersistence>;
  readonly router: RouterNamespace;
  readonly scopeId: string;
  readonly localScopeId: string;
  readonly parentScopeId: string | null;
  readonly spec: ExplicitSignalSpecNamespace;
  scope(localScopeId: string): ScopedSignalNamespace<TPersistence>;
  controller<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = Record<string, never>,
    TInternal extends Record<string, unknown> = Record<string, never>,
  >(definition: ControllerContractDefinition<TInputs, TOutputs, TInternal>): ControllerContract<TInputs, TOutputs, TInternal>;
  controller<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = Record<string, never>,
    TInternal extends Record<string, unknown> = Record<string, never>,
  >(builder: ControllerContractBuilder<TPersistence, TInputs, TOutputs, TInternal>): ControllerContract<TInputs, TOutputs, TInternal>;
  readonly form: FormFactory;
  publicInput<THandle extends InputSignalHandle>(
    handle: THandle,
    options?: PublicGraphInputOptions,
  ): PublicGraphInputContractEntry<THandle>;
  input<T = SignalValue>(initial: T, options?: InputAuthoringOptions): InputSignalHandle<T>;
  inputAsync<T = SignalValue>(
    initial: T,
    options?: InputAuthoringOptions,
  ): Promise<InputSignalHandle<T>>;
  linked<T = SignalValue>(
    source: () => T,
    options?: LinkedSignalOptions,
  ): LinkedSignalHandle<T, T>;
  linked<TSource = SignalValue>(
    definition: LinkedIdentitySignalDefinition<TSource>,
  ): LinkedSignalHandle<TSource, TSource>;
  linked<TSource = SignalValue, TValue = TSource>(
    definition: LinkedComputedSignalDefinition<TSource, TValue>,
  ): LinkedSignalHandle<TValue, TSource>;
  linkedAsync<T = SignalValue>(
    source: () => T,
    options?: LinkedSignalOptions,
  ): Promise<LinkedSignalHandle<T, T>>;
  linkedAsync<TSource = SignalValue>(
    definition: LinkedIdentitySignalDefinition<TSource>,
  ): Promise<LinkedSignalHandle<TSource, TSource>>;
  linkedAsync<TSource = SignalValue, TValue = TSource>(
    definition: LinkedComputedSignalDefinition<TSource, TValue>,
  ): Promise<LinkedSignalHandle<TValue, TSource>>;
  computedSpec<T = SignalValue>(
    id: string,
    spec: ComputedSpec | NamedComputedCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  computed<T = SignalValue>(spec: ComputedSpec, options?: SignalAuthoringOptions): ComputedSignalHandle<T>;
  computed<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): ComputedSignalHandle<T>;
  computedAsync<T = SignalValue>(spec: ComputedSpec, options?: SignalAuthoringOptions): Promise<ComputedSignalHandle<T>>;
  computedAsync<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): Promise<ComputedSignalHandle<T>>;
  computedCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  outputSpec<T = SignalValue>(
    id: string,
    spec: OutputSpec | NamedOutputCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): OutputSignalHandle<T>;
  output<T = SignalValue>(spec: OutputSpec, options?: SignalAuthoringOptions): OutputSignalHandle<T>;
  output<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): OutputSignalHandle<T>;
  outputAsync<T = SignalValue>(spec: OutputSpec, options?: SignalAuthoringOptions): Promise<OutputSignalHandle<T>>;
  outputAsync<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): Promise<OutputSignalHandle<T>>;
  outputCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): OutputSignalHandle<T>;
  graph<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = GraphOutputDefinitions,
  >(
    id: string,
    definition: GraphPublicationRequest<TInputs, TOutputs>,
  ): PublishedSignalGraph<TOutputs, TInputs>;
  graph<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = GraphOutputDefinitions,
  >(
    id: string,
    builder: GraphBuilder<TPersistence, TInputs, TOutputs>,
  ): PublishedSignalGraph<TOutputs, TInputs>;
  history(): CallableSignalHistory;
  contract(): SignalsRuntimeContract;
  assertCompatibility(options: SignalsCompatibilityAssertionOptions): SignalsRuntimeContract;
  canonicalId(localId: string): string;
  signalIdentity(localId: string): ScopedSignalIdentity;
  descriptor(): ScopeDescriptor;
}

export interface CallableSignals<TPersistence = SignalValue> {
  readonly host: CallableSignalsHost<TPersistence>;
  readonly router: RouterNamespace;
  readonly spec: ExplicitSignalSpecNamespace;
  scope(localScopeId: string): GraphScope<TPersistence>;
  controller<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = Record<string, never>,
    TInternal extends Record<string, unknown> = Record<string, never>,
  >(definition: ControllerContractDefinition<TInputs, TOutputs, TInternal>): ControllerContract<TInputs, TOutputs, TInternal>;
  controller<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = Record<string, never>,
    TInternal extends Record<string, unknown> = Record<string, never>,
  >(builder: ControllerContractBuilder<TPersistence, TInputs, TOutputs, TInternal>): ControllerContract<TInputs, TOutputs, TInternal>;
  readonly form: FormFactory;
  publicInput<THandle extends InputSignalHandle>(
    handle: THandle,
    options?: PublicGraphInputOptions,
  ): PublicGraphInputContractEntry<THandle>;
  input<T = SignalValue>(initial: T, options?: InputAuthoringOptions): InputSignalHandle<T>;
  inputAsync<T = SignalValue>(
    initial: T,
    options?: InputAuthoringOptions,
  ): Promise<InputSignalHandle<T>>;
  linked<T = SignalValue>(
    source: () => T,
    options?: LinkedSignalOptions,
  ): LinkedSignalHandle<T, T>;
  linked<TSource = SignalValue>(
    definition: LinkedIdentitySignalDefinition<TSource>,
  ): LinkedSignalHandle<TSource, TSource>;
  linked<TSource = SignalValue, TValue = TSource>(
    definition: LinkedComputedSignalDefinition<TSource, TValue>,
  ): LinkedSignalHandle<TValue, TSource>;
  linkedAsync<T = SignalValue>(
    source: () => T,
    options?: LinkedSignalOptions,
  ): Promise<LinkedSignalHandle<T, T>>;
  linkedAsync<TSource = SignalValue>(
    definition: LinkedIdentitySignalDefinition<TSource>,
  ): Promise<LinkedSignalHandle<TSource, TSource>>;
  linkedAsync<TSource = SignalValue, TValue = TSource>(
    definition: LinkedComputedSignalDefinition<TSource, TValue>,
  ): Promise<LinkedSignalHandle<TValue, TSource>>;
  computedSpec<T = SignalValue>(
    id: string,
    spec: ComputedSpec | NamedComputedCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  computed<T = SignalValue>(spec: ComputedSpec, options?: SignalAuthoringOptions): ComputedSignalHandle<T>;
  computed<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): ComputedSignalHandle<T>;
  computedAsync<T = SignalValue>(spec: ComputedSpec, options?: SignalAuthoringOptions): Promise<ComputedSignalHandle<T>>;
  computedAsync<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): Promise<ComputedSignalHandle<T>>;
  computedCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  outputSpec<T = SignalValue>(
    id: string,
    spec: OutputSpec | NamedOutputCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): OutputSignalHandle<T>;
  output<T = SignalValue>(spec: OutputSpec, options?: SignalAuthoringOptions): OutputSignalHandle<T>;
  output<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): OutputSignalHandle<T>;
  outputAsync<T = SignalValue>(spec: OutputSpec, options?: SignalAuthoringOptions): Promise<OutputSignalHandle<T>>;
  outputAsync<T = SignalValue>(compute: () => T, options?: SignalAuthoringOptions): Promise<OutputSignalHandle<T>>;
  outputCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): OutputSignalHandle<T>;
  graph<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = GraphOutputDefinitions,
  >(
    id: string,
    definition: GraphPublicationRequest<TInputs, TOutputs>,
  ): PublishedSignalGraph<TOutputs, TInputs>;
  graph<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = GraphOutputDefinitions,
  >(
    id: string,
    builder: GraphBuilder<TPersistence, TInputs, TOutputs>,
  ): PublishedSignalGraph<TOutputs, TInputs>;
  importGraph<
    TInputs extends GraphInputDefinitions = Record<string, never>,
    TOutputs extends GraphOutputDefinitions = GraphOutputDefinitions,
  >(
    definition: ExportedSignalGraphDefinition<TOutputs, TInputs>,
    snapshot: ExportedSignalGraphSnapshot<TOutputs, TInputs>,
  ): ImportedSignalGraph<TOutputs, TInputs>;
  read<T = SignalValue>(target: CallableSignalTarget): T;
  transaction(callback: (tx: CallableSignalsTransaction) => void): RunSummary | Promise<RunSummary>;
  batch(callback: (tx: CallableSignalsTransaction) => void): RunSummary | Promise<RunSummary>;
  transactionAsync(callback: (tx: AsyncCallableSignalsTransaction) => void): Promise<RunSummary>;
  batchAsync(callback: (tx: AsyncCallableSignalsTransaction) => void): Promise<RunSummary>;
  /**
   * Worker-first: drain pending authored publications and mutations.
   * Await before tip-honest handoffs (submit, write, worker proof).
   * Not required for React paint — host tip notify is the paint path.
   * Line `awaitSettlement({ drainAuthoredWork: true })` can opt into this drain
   * after tip status settles; default awaitSettlement does not.
   */
  settleAuthoredWork(): Promise<void>;
  watch(target: CallableSignalTarget, callback: (notice: WebObservationNotice) => void): DisposableHandle;
  effect(target: CallableSignalTarget, callback: () => void): DisposableHandle;
  nuke(handle: DisposableHandle): boolean;
  diagnostics(): CallableSignalDiagnostics;
  history(): CallableSignalHistory;
  contract(): SignalsRuntimeContract;
  assertCompatibility(options: SignalsCompatibilityAssertionOptions): SignalsRuntimeContract;
  specialist(): CallableSignalSpecialist;
  adapters(): CallableSignalAdapters;
  compatibilityApp(): SignalApp;
  compatibilityRuntime(): SignalRuntime;
  terminate(): void | Promise<void>;
  free(): void;
  [Symbol.dispose](): void;
}

export {
  Signals as RawSignals,
  InputSignal as RawInputSignal,
  ComputedSignal as RawComputedSignal,
  OutputSignal as RawOutputSignal,
  DisposableHandle as RawDisposableHandle,
  SignalAdapters,
  SignalApp,
  SignalDiagnostics,
  SignalHistory,
  SignalRuntime,
  SignalSpecialist,
};
