import type {
  AspectId,
  VersionSummary,
  InputOptions,
  ComputedSpec,
  OutputSpec,
  RunSummary,
  SignalValue,
  WebObservationNotice,
} from "./model.js";
import type {
  BranchStateProofReport,
  HostCapabilityCompatibility,
  LineageSummary,
  MergePlanArtifact,
  MergePlanProofEnvelope,
  MergePolicyPreviewRequest,
  MergeResultArtifact,
  MergeResultProofEnvelope,
  ReplayArtifactProofInput,
  ReplayArtifactProofReport,
  ReplayParityProofReport,
  ReplaySummary,
  RuntimeBranchHandle,
  RuntimeDefinitionEnvelope,
  RuntimeEnvelope,
  HostCapabilityDiagnosticsReport,
  HostCapabilityTransportReport,
  RuntimeProofReport,
  RuntimeSnapshotArtifact,
  RuntimeSnapshotEnvelope,
  GraphSummary,
} from "./diagnostics.js";
import type {
  ExportedSignalGraphDefinition,
  ExportedSignalGraphSnapshot,
  GraphBuilder,
  GraphInputDefinitions,
  GraphOutputDefinitions,
  GraphPublicationRequest,
  GraphScope,
  ImportedSignalGraph,
  PublicGraphInputContractEntry,
  PublicGraphInputOptions,
  PublishedSignalGraph,
} from "./graph_surface.js";
import type {
  ControllerAuthoringSurface,
  ControllerContract,
  ControllerContractBuilder,
  ControllerContractDefinition,
} from "./controller_surface.js";
import type {
  FormFactory,
} from "./forms_surface.js";
import type {
  RouterNamespace,
} from "./router_surface.js";
import type {
  ComputedSignal,
  DisposableHandle,
  InputSignal,
  OutputSignal,
  SignalAdapters,
  SignalApp,
  SignalDiagnostics,
  SignalHistory,
  SignalRuntime,
  SignalSpecialist,
  Signals,
} from "./raw_surface.js";

declare const WorthSignalBrand: unique symbol;
declare const WorthSignalInputBrand: unique symbol;
declare const WorthSignalComputedBrand: unique symbol;
declare const WorthSignalOutputBrand: unique symbol;
declare const WorthSignalHostCapabilityPlanBrand: unique symbol;
declare const WorthSignalViewportCapabilityRegistrationBrand: unique symbol;
declare const WorthSignalViewportCapabilityHandleBrand: unique symbol;
declare const WorthSignalVisibilityCapabilityRegistrationBrand: unique symbol;
declare const WorthSignalVisibilityCapabilityHandleBrand: unique symbol;
declare const WorthSignalOnlineCapabilityRegistrationBrand: unique symbol;
declare const WorthSignalOnlineCapabilityHandleBrand: unique symbol;
declare const WorthSignalClockCapabilityRegistrationBrand: unique symbol;
declare const WorthSignalClockCapabilityHandleBrand: unique symbol;
declare const WorthSignalPersistenceCapabilityRegistrationBrand: unique symbol;
declare const WorthSignalPersistenceCapabilityValueBrand: unique symbol;
declare const WorthSignalPersistenceCapabilityHandleBrand: unique symbol;

export interface Signal<T = SignalValue> {
  (): T;
  get(): T;
  value(): T;
  free(): void;
  [Symbol.dispose](): void;
  readonly id: string;
  readonly debugName: string | null;
  readonly [WorthSignalBrand]: "signal";
}

export interface NamedComputedCallbackDefinition<T = SignalValue> {
  compute: () => T;
}

export interface NamedOutputCallbackDefinition<T = SignalValue> {
  compute: () => T;
}

type PatchSignalValue<T> = T extends ReadonlyArray<infer U>
  ? ReadonlyArray<U>
  : T extends Array<infer U>
    ? Array<U>
    : T extends object
      ? Partial<T>
      : never;

type AssignSignalValue<T> = T extends ReadonlyArray<unknown>
  ? never
  : T extends Array<unknown>
    ? never
    : T extends object
      ? Partial<T>
      : never;

export interface InputSignalHandle<T = SignalValue> extends Signal<T> {
  set(value: T): RunSummary | Promise<RunSummary>;
  reset(): RunSummary | Promise<RunSummary>;
  patch(value: PatchSignalValue<T>): RunSummary | Promise<RunSummary>;
  assign(fields: AssignSignalValue<T>): RunSummary | Promise<RunSummary>;
  readonly [WorthSignalInputBrand]: "input";
}

export interface LinkedSignalPrevious<TValue = SignalValue, TSource = TValue> {
  readonly value: TValue;
  readonly source: TSource;
}

export interface LinkedSignalOptions {
  debugName?: string;
}

export interface LinkedIdentitySignalDefinition<TSource = SignalValue> {
  source: () => TSource;
  debugName?: string;
}

export interface LinkedComputedSignalDefinition<TSource = SignalValue, TValue = SignalValue> {
  source: () => TSource;
  computation: (
    source: TSource,
    previous: LinkedSignalPrevious<TValue, TSource> | null,
  ) => TValue;
  debugName?: string;
}

export type LinkedSignalDefinition<TSource = SignalValue, TValue = SignalValue> =
  | LinkedIdentitySignalDefinition<TSource>
  | LinkedComputedSignalDefinition<TSource, TValue>;

export interface LinkedSignalHandle<TValue = SignalValue, TSource = TValue>
  extends InputSignalHandle<TValue> {
  relink(): RunSummary | Promise<RunSummary>;
}

export interface ComputedSignalHandle<T = SignalValue> extends Signal<T> {
  readonly [WorthSignalComputedBrand]: "computed";
}

export interface OutputSignalHandle<T = SignalValue> extends Signal<T> {
  readonly [WorthSignalOutputBrand]: "output";
}

export type HostCapabilitySubscription =
  | void
  | (() => void)
  | { dispose(): void }
  | { free(): void };

export interface ViewportCapabilityState {
  width: number;
  height: number;
}

export interface ViewportCapabilitySource {
  current(): ViewportCapabilityState;
  subscribe(listener: () => void): HostCapabilitySubscription;
}

export interface ViewportCapabilityOptions {
  source: ViewportCapabilitySource;
  compatibility?: HostCapabilityCompatibility;
}

export interface ViewportCapabilityRegistration {
  readonly family: "viewport";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalViewportCapabilityRegistrationBrand]: "viewportCapabilityRegistration";
}

export interface VisibilityCapabilitySource {
  current(): boolean | "visible" | "hidden";
  subscribe(listener: () => void): HostCapabilitySubscription;
}

export interface VisibilityCapabilityOptions {
  source: VisibilityCapabilitySource;
  compatibility?: HostCapabilityCompatibility;
}

export interface VisibilityCapabilityRegistration {
  readonly family: "visibility";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalVisibilityCapabilityRegistrationBrand]: "visibilityCapabilityRegistration";
}

export interface OnlineCapabilitySource {
  current(): boolean | "online" | "offline";
  subscribe(listener: () => void): HostCapabilitySubscription;
}

export interface OnlineCapabilityOptions {
  source: OnlineCapabilitySource;
  compatibility?: HostCapabilityCompatibility;
}

export interface OnlineCapabilityRegistration {
  readonly family: "online";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalOnlineCapabilityRegistrationBrand]: "onlineCapabilityRegistration";
}

export interface ClockCapabilitySource {
  current(): number;
}

export interface ClockCapabilityOptions {
  source: ClockCapabilitySource;
  pollMs?: number;
  compatibility?: HostCapabilityCompatibility;
}

export interface ClockCapabilityRegistration {
  readonly family: "clock";
  readonly compatibility: HostCapabilityCompatibility;
  readonly pollMs: number;
  readonly [WorthSignalClockCapabilityRegistrationBrand]: "clockCapabilityRegistration";
}

export interface PersistenceCapabilitySource<T = SignalValue> {
  current(): T;
}

export interface PersistenceCapabilityOptions<T = SignalValue> {
  source: PersistenceCapabilitySource<T>;
  compatibility?: HostCapabilityCompatibility;
}

export interface PersistenceCapabilityRegistration<T = SignalValue> {
  readonly family: "persistence";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalPersistenceCapabilityRegistrationBrand]: "persistenceCapabilityRegistration";
  readonly [WorthSignalPersistenceCapabilityValueBrand]?: T;
}

export interface HostCapabilityPlanInput<TPersistence = SignalValue> {
  viewport?: ViewportCapabilityRegistration;
  visibility?: VisibilityCapabilityRegistration;
  online?: OnlineCapabilityRegistration;
  clock?: ClockCapabilityRegistration;
  persistence?: PersistenceCapabilityRegistration<TPersistence>;
}

export interface HostCapabilityPlan<TPersistence = SignalValue> {
  readonly viewport?: ViewportCapabilityRegistration;
  readonly visibility?: VisibilityCapabilityRegistration;
  readonly online?: OnlineCapabilityRegistration;
  readonly clock?: ClockCapabilityRegistration;
  readonly persistence?: PersistenceCapabilityRegistration<TPersistence>;
  readonly [WorthSignalHostCapabilityPlanBrand]: "hostCapabilityPlan";
}

export type SignalsDeployment = "workerFirst" | "mainThreadCompatibility";

export interface CreateSignalsAssets {
  /**
   * Bundler-emitted URL for `worth_signal_wasm_bg.wasm`.
   * Vite: `import wasmUrl from "worth-signals-wasm/wasm?url"`.
   */
  wasmUrl?: string | URL;
  /**
   * Bundler-emitted URL for the worker-first runtime worker.
   * Required together with `wasmUrl` when `deployment` is `workerFirst`.
   * Vite: `import workerUrl from "worth-signals-wasm/worker?worker&url"`.
   */
  workerUrl?: string | URL;
}

export interface CreateSignalsOptions<TPersistence = SignalValue> {
  deployment?: SignalsDeployment;
  hostCapabilities?: HostCapabilityPlan<TPersistence>;
  /**
   * Explicit asset URLs for bundlers that relocate package modules
   * (for example older Vite optimizeDeps hosts). Optional on Vite 8+ when
   * default relative resolution works; recommended as the portable path.
   */
  assets?: CreateSignalsAssets;
}

export interface WrapSignalsOptions<TPersistence = SignalValue> {
  hostCapabilities?: HostCapabilityPlan<TPersistence>;
}

export interface SignalsCompatibilityRecovery {
  deployment: "mainThreadCompatibility";
  message: string;
}

export type SignalsRuntimeCapabilityName =
  | "callableSurface"
  | "scopedAuthoring"
  | "specNamespace"
  | "workerRuntime";

export type SignalsRuntimeSurfaceFamily =
  | "mainThreadCompatibilityCallable"
  | "mainThreadCompatibilityScoped"
  | "workerFirstCallable"
  | "workerFirstScoped";

export interface SignalsRuntimeCapabilities {
  callableSurface: boolean;
  scopedAuthoring: boolean;
  specNamespace: boolean;
  workerRuntime: boolean;
}

export interface SignalsRuntimeContract {
  surfaceFamily: SignalsRuntimeSurfaceFamily;
  surfaceVersion: "1";
  deployment: SignalsDeployment;
  scopeId: string | null;
  capabilities: Readonly<SignalsRuntimeCapabilities>;
}

export interface SignalsCompatibilityAssertionOptions {
  requires?: ReadonlyArray<SignalsRuntimeCapabilityName>;
}

export interface SignalsConstructionArtifact {
  artifactFamily: "workerUnavailableConstruction" | "signalsConstructionDenied";
  requestedDeployment: SignalsDeployment;
  reason: string;
  message: string;
  compatibilityRecovery: SignalsCompatibilityRecovery;
}

export interface SignalsConstructionExplanation {
  requestedDeployment: SignalsDeployment;
  selectedFamily:
    | "workerFirst"
    | "mainThreadCompatibility"
    | "workerUnavailable"
    | "denied";
  selectedDeployment: SignalsDeployment | null;
  reason: string;
  compatibilityRecovery: SignalsCompatibilityRecovery | null;
}

export interface InputAuthoringOptions extends InputOptions {
  debugName?: string;
}

export interface SignalAuthoringOptions {
  debugName?: string;
}

export interface CallbackSignalAuthoringOptions {
  debugName?: string;
}

export interface ExplicitSignalSpecNamespace {
  input<T = SignalValue>(id: string, initial: T, options?: InputAuthoringOptions): InputSignalHandle<T>;
  computed<T = SignalValue>(
    id: string,
    spec: ComputedSpec | NamedComputedCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  computedCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  output<T = SignalValue>(
    id: string,
    spec: OutputSpec | NamedOutputCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): OutputSignalHandle<T>;
  outputCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): OutputSignalHandle<T>;
}

export interface ViewportCapabilityDescriptor {
  family: "viewport";
  compatibility: HostCapabilityCompatibility;
  registrationId: "viewport";
}

export interface HostViewportCapability {
  size(): ViewportCapabilityState;
  width(): number;
  height(): number;
  descriptor(): ViewportCapabilityDescriptor;
  readonly [WorthSignalViewportCapabilityHandleBrand]: "viewportCapabilityHandle";
}

export interface VisibilityCapabilityDescriptor {
  family: "visibility";
  compatibility: HostCapabilityCompatibility;
  registrationId: "visibility";
}

export interface HostVisibilityCapability {
  state(): "visible" | "hidden";
  isVisible(): boolean;
  descriptor(): VisibilityCapabilityDescriptor;
  readonly [WorthSignalVisibilityCapabilityHandleBrand]: "visibilityCapabilityHandle";
}

export interface OnlineCapabilityDescriptor {
  family: "online";
  compatibility: HostCapabilityCompatibility;
  registrationId: "online";
}

export interface HostOnlineCapability {
  state(): "online" | "offline";
  isOnline(): boolean;
  descriptor(): OnlineCapabilityDescriptor;
  readonly [WorthSignalOnlineCapabilityHandleBrand]: "onlineCapabilityHandle";
}

export interface ClockCapabilityDescriptor {
  family: "clock";
  compatibility: HostCapabilityCompatibility;
  registrationId: "clock";
}

export interface HostClockCapability {
  now(): number;
  descriptor(): ClockCapabilityDescriptor;
  readonly [WorthSignalClockCapabilityHandleBrand]: "clockCapabilityHandle";
}

export interface PersistenceCapabilityDescriptor {
  family: "persistence";
  compatibility: HostCapabilityCompatibility;
  registrationId: "persistence";
}

export interface HostPersistenceCapability<T = SignalValue> {
  value(): T;
  commit(): RunSummary | Promise<RunSummary>;
  descriptor(): PersistenceCapabilityDescriptor;
  readonly [WorthSignalPersistenceCapabilityHandleBrand]: "persistenceCapabilityHandle";
}

export interface CallableSignalsHost<TPersistence = SignalValue> {
  readonly viewport?: HostViewportCapability;
  readonly visibility?: HostVisibilityCapability;
  readonly online?: HostOnlineCapability;
  readonly clock?: HostClockCapability;
  readonly persistence?: HostPersistenceCapability<TPersistence>;
}

export type CallableSignalTarget =
  | string
  | InputSignalHandle
  | ComputedSignalHandle
  | OutputSignalHandle;

export interface CallableSignalsTransaction {
  set<T = SignalValue>(input: InputSignalHandle<T>, value: T): void;
  patch<T = SignalValue>(input: InputSignalHandle<T>, value: PatchSignalValue<T>): void;
  setWithAspects<T = SignalValue>(input: InputSignalHandle<T>, value: T, aspects: ReadonlyArray<AspectId>): void;
  setWithRegions<T = SignalValue>(input: InputSignalHandle<T>, value: T, changedRegions: unknown): void;
  setWithRegionsAndAspects(
    input: InputSignalHandle,
    value: SignalValue,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): void;
  free(): void;
  [Symbol.dispose](): void;
}

export interface AsyncCallableSignalsTransaction {
  set<T = SignalValue>(input: InputSignalHandle<T>, value: T): void;
  patch<T = SignalValue>(input: InputSignalHandle<T>, value: PatchSignalValue<T>): void;
  setWithAspects<T = SignalValue>(input: InputSignalHandle<T>, value: T, aspects: ReadonlyArray<AspectId>): void;
  setWithRegions<T = SignalValue>(input: InputSignalHandle<T>, value: T, changedRegions: unknown): void;
  setWithRegionsAndAspects(
    input: InputSignalHandle,
    value: SignalValue,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): void;
  free(): void;
  [Symbol.dispose](): void;
}

export interface CallableSignalAdapters {
  exportDefinitions(): RuntimeDefinitionEnvelope;
  exportRuntimeEnvelope(): RuntimeEnvelopeArtifact;
  replaceRuntimeEnvelope(envelope: RuntimeEnvelope): Promise<void>;
  restoreExactRuntimeEnvelope(envelope: RuntimeEnvelopeArtifact): Promise<void>;
  /** Release the artifact's pending exact restore token; `false` when it is no longer pending. */
  discardExactRuntimeEnvelope(envelope: RuntimeEnvelopeArtifact): boolean | Promise<boolean>;
  runtimeProofReport(): RuntimeProofReport;
  hostCapabilityTransportReport(envelope?: RuntimeEnvelope): HostCapabilityTransportReport;
  free(): void;
  [Symbol.dispose](): void;
}

export type CallableSignalDiagnostics =
  SignalDiagnostics & {
    hostCapabilityReport(): HostCapabilityDiagnosticsReport;
  };

export type CallableBranchId = number | bigint;

export interface RuntimeEnvelopeArtifact extends RuntimeEnvelope {
  runtimeEnvelopeRestoreToken: string;
  runtimeEnvelopeRestoreMode: "SameRuntimeExact";
  runtimeEnvelopePortableWire: string;
}

export interface RuntimeSnapshotEnvelopeArtifact extends RuntimeSnapshotEnvelope {
  snapshotEnvelopeRestoreToken: string;
  snapshotEnvelopeRestoreMode: "SameRuntimeExact";
  snapshotEnvelopePortableWire: string;
}

export interface RuntimeSnapshotArtifactWithWire extends RuntimeSnapshotArtifact {
  snapshotRestoreToken: string;
  snapshotRestoreMode: "SameRuntimeExact";
  snapshotPortableWire: string;
}

export interface CallableSignalHistory {
  replay_for(id: string): ReplaySummary;
  lineage_for(id: string): LineageSummary;
  snapshot(): RuntimeSnapshotEnvelopeArtifact;
  restore_snapshot(snapshot: RuntimeSnapshotEnvelope): void | Promise<void>;
  restore_exact_snapshot(snapshot: RuntimeSnapshotEnvelopeArtifact): void | Promise<void>;
  /** Release the artifact's pending exact restore token; `false` when it is no longer pending. */
  discard_exact_artifact(
    artifact: RuntimeSnapshotEnvelopeArtifact | RuntimeSnapshotArtifactWithWire,
  ): boolean | Promise<boolean>;
  current_branch(): RuntimeBranchHandle;
  branches(): ReadonlyArray<RuntimeBranchHandle>;
  create_branch(name: string): RuntimeBranchHandle | Promise<RuntimeBranchHandle>;
  switch_branch(branchId: CallableBranchId): void | Promise<void>;
  replay_for_branch(branchId: CallableBranchId): ReplaySummary;
  branch_snapshot(branchId: CallableBranchId): RuntimeSnapshotArtifactWithWire;
  branch_snapshot_id(branchId: CallableBranchId): bigint;
  branch_snapshot_envelope(branchId: CallableBranchId): RuntimeSnapshotEnvelopeArtifact;
  restore_branch_snapshot(
    branchId: CallableBranchId,
    snapshot: RuntimeSnapshotArtifact,
  ): void | Promise<void>;
  restore_exact_branch_snapshot(
    branchId: CallableBranchId,
    snapshot: RuntimeSnapshotArtifactWithWire,
  ): void | Promise<void>;
  restore_branch_snapshot_by_id(
    branchId: CallableBranchId,
    snapshotId: number | bigint,
  ): void | Promise<void>;
  merge_branches(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergeResultArtifact | Promise<MergeResultArtifact>;
  merge_branches_with_proof(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergeResultProofEnvelope | Promise<MergeResultProofEnvelope>;
  plan_merge_branches(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergePlanArtifact | Promise<MergePlanArtifact>;
  plan_merge_branches_with_proof(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergePlanProofEnvelope | Promise<MergePlanProofEnvelope>;
  plan_merge_policy_preview(
    request: MergePolicyPreviewRequest,
  ): MergePlanArtifact | Promise<MergePlanArtifact>;
  plan_merge_policy_preview_with_proof(
    request: MergePolicyPreviewRequest,
  ): MergePlanProofEnvelope | Promise<MergePlanProofEnvelope>;
  merge_branches_policy_preview(
    request: MergePolicyPreviewRequest,
  ): MergeResultArtifact | Promise<MergeResultArtifact>;
  merge_branches_policy_preview_with_proof(
    request: MergePolicyPreviewRequest,
  ): MergeResultProofEnvelope | Promise<MergeResultProofEnvelope>;
  branch_state_proof(branchId: CallableBranchId): BranchStateProofReport;
  replay_parity_proof(
    expectedBranchId: CallableBranchId,
    replayedBranchId: CallableBranchId,
  ): ReplayParityProofReport;
  replay_artifact_proof(
    expected: ReplayArtifactProofInput,
    replayedBranchId: CallableBranchId,
  ): ReplayArtifactProofReport;
  subscribe(listener: () => void): () => void;
  free(): void;
  [Symbol.dispose](): void;
}

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

export function viewportCapability(options: ViewportCapabilityOptions): ViewportCapabilityRegistration;
export function visibilityCapability(options: VisibilityCapabilityOptions): VisibilityCapabilityRegistration;
export function onlineCapability(options: OnlineCapabilityOptions): OnlineCapabilityRegistration;
export function clockCapability(options: ClockCapabilityOptions): ClockCapabilityRegistration;
export function persistenceCapability<T = SignalValue>(options: PersistenceCapabilityOptions<T>): PersistenceCapabilityRegistration<T>;
export function hostCapabilityPlan<TPersistence = SignalValue>(input?: HostCapabilityPlanInput<TPersistence>): HostCapabilityPlan<TPersistence>;
export function explainCreateSignalsConstruction<TPersistence = SignalValue>(
  options?: CreateSignalsOptions<TPersistence>,
): SignalsConstructionExplanation;
export function planCreateSignalsDeployment<TPersistence = SignalValue>(
  options?: CreateSignalsOptions<TPersistence>,
): { explanation: SignalsConstructionExplanation };
export function createSignals<TPersistence = SignalValue>(options?: CreateSignalsOptions<TPersistence>): Promise<CallableSignals<TPersistence>>;
export function createCallableSignals<TPersistence = SignalValue>(options?: CreateSignalsOptions<TPersistence>): Promise<CallableSignals<TPersistence>>;
export function wrapSignals<TPersistence = SignalValue>(signals: Signals, options?: WrapSignalsOptions<TPersistence>): CallableSignals<TPersistence>;

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
