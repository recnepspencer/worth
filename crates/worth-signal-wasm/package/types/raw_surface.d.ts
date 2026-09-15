import type {
  AspectId,
  ComputedSpec,
  InputOptions,
  KeyedSetValue,
  KeyedRecipeFamilySpec,
  KeyedSourceFamilySpec,
  OutputSpec,
  RecipeSpec,
  RunSummary,
  SignalValue,
  SourceSpec,
  TransactionOp,
  VersionSummary,
  WebObservationNotice,
} from "./model.js";
import type {
  ExecutionHistorySurfaceSummary,
  FailureSummary,
  FlowSurfaceSummary,
  InvalidationPlanningEstimate,
  GraphSummary,
  HealthSummary,
  HostCapabilityDiagnosticsEvent,
  InvalidationTraceRecord,
  ObservationSurfaceSummary,
  RollbackDiagnostic,
  RuntimeBranchHandle,
  RuntimeDefinitionEnvelope,
  RuntimeEnvelope,
  RuntimeProofReport,
  RuntimeSnapshotArtifact,
  RuntimeSnapshotEnvelope,
  ReplaySummary,
  LineageSummary,
  BranchStateProofReport,
  ReplayParityProofReport,
  ReplayArtifactProofInput,
  ReplayArtifactProofReport,
  MergePolicyPreviewRequest,
  MergePlanArtifact,
  MergeResultArtifact,
  MergePlanProofEnvelope,
  MergeResultProofEnvelope,
  ExecutionHistorySummary,
  WebPerformanceSummary,
  WhySummary,
  RuntimePolicySpec,
} from "./diagnostics.js";
import type {
  WorkerBrowserHistoryIngress,
  WorkerBrowserHistoryIngressReport,
  WorkerCommittedProjectionPacket,
  WorkerCommittedProjectionRequest,
  WorkerCommittedTransactionEnvelope,
  WorkerDiagnosticsHistoryReadPacket,
  WorkerDiagnosticsSummaryReadPacket,
  WorkerGraphPublicationSummary,
  WorkerHostCapabilityIngressBatch,
  WorkerHostCapabilityIngressReport,
  WorkerHostEffectAcknowledgement,
  WorkerHostEffectAcknowledgementReport,
  WorkerHostEffectRequest,
  WorkerHostEffectRequestEnvelope,
  WorkerLifecycleControlPacket,
  WorkerMainThreadHostBridgeCertificationPackage,
  WorkerObservationDeliveryAttachRequest,
  WorkerObservationDeliveryDetachRequest,
  WorkerObservationDeliveryPacket,
  WorkerOutputDeliveryRequest,
  WorkerOutputDeliveryPacket,
  WorkerPortableGraphPublication,
  WorkerSignalReadbackPacket,
  WorkerSignalReadbackRequest,
  WorkerRuntimeBootstrapRecord,
  WorkerRuntimeShellLock,
} from "./worker_runtime_bridge.js";

/**
 * Release one pending exact restore artifact. `false` when the token is not
 * pending in this realm (unknown, consumed, discarded, or evicted).
 */
export function discardRestoreToken(token: string): boolean;
/** How many exact restore artifacts this realm currently holds (at most 64). */
export function pendingRestoreTokenCount(): number;

export class InputSignal {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  get(): SignalValue;
  peek(): SignalValue;
  readonly id: string;
}

export class ComputedSignal {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  get(): SignalValue;
  peek(): SignalValue;
  readonly id: string;
}

export class OutputSignal {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  get(): SignalValue;
  peek(): SignalValue;
  readonly id: string;
}

export class DisposableHandle {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}

export class SignalsTransaction {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  set(input: InputSignal, value: SignalValue): void;
  setWithAspects(input: InputSignal, value: SignalValue, aspects: ReadonlyArray<AspectId>): void;
  setWithRegions(input: InputSignal, value: SignalValue, changedRegions: unknown): void;
  setWithRegionsAndAspects(
    input: InputSignal,
    value: SignalValue,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): void;
}

export type SignalTarget = string | InputSignal | ComputedSignal | OutputSignal;

export class Signals {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  input(id: string, initial: SignalValue, options?: InputOptions): InputSignal;
  computedSpec(id: string, spec: ComputedSpec): ComputedSignal;
  computedCallback(id: string, callback: () => SignalValue): ComputedSignal;
  computed(id: string, spec: ComputedSpec): ComputedSignal;
  outputSpec(id: string, spec: OutputSpec): OutputSignal;
  outputCallback(id: string, callback: () => SignalValue): never;
  output(id: string, spec: OutputSpec): OutputSignal;
  read(target: SignalTarget): SignalValue;
  transaction(callback: (tx: SignalsTransaction) => void): RunSummary;
  batch(callback: (tx: SignalsTransaction) => void): RunSummary;
  watch(target: SignalTarget, callback: (notice: WebObservationNotice) => void): DisposableHandle;
  effect(target: SignalTarget, callback: () => void): DisposableHandle;
  nuke(handle: DisposableHandle): boolean;
  diagnostics(): SignalDiagnostics;
  history(): SignalHistory;
  specialist(): SignalSpecialist;
  adapters(): SignalAdapters;
  compatibilityApp(): SignalApp;
  compatibilityRuntime(): SignalRuntime;
}

export class SignalDiagnostics {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  subscribe(listener: () => void): DisposableHandle;
  why(id: string): WhySummary;
  health(): HealthSummary;
  summaryNow(): GraphSummary;
  historyNow(): ExecutionHistorySurfaceSummary;
  latestFlow(): FlowSurfaceSummary | null;
  latestObservation(): ObservationSurfaceSummary | null;
  latestHostCapabilityEvent(): HostCapabilityDiagnosticsEvent | null;
  recentHostCapabilityEvents(): ReadonlyArray<HostCapabilityDiagnosticsEvent>;
  performanceSummary(): WebPerformanceSummary;
  latestFailure(): FailureSummary | null;
  latestRollback(): RollbackDiagnostic | null;
  latestInvalidationPlanningEstimate(): InvalidationPlanningEstimate | null;
  latestInvalidationTraceRecords(): ReadonlyArray<InvalidationTraceRecord>;
  recentHistory(): ReadonlyArray<ExecutionHistorySummary>;
}

export class SignalHistory {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  replay_for(id: string): ReplaySummary;
  lineage_for(id: string): LineageSummary;
  snapshot(): RuntimeSnapshotEnvelope;
  snapshot_wire(): string;
  snapshot_portable_wire(): string;
  discard_restore_token(token: string): boolean;
  restore_snapshot(snapshot: RuntimeSnapshotEnvelope): void;
  restore_snapshot_wire(snapshot: string): void;
  restore_snapshot_portable_wire(snapshot: string): void;
  current_branch(): RuntimeBranchHandle;
  branches(): ReadonlyArray<RuntimeBranchHandle>;
  create_branch(name: string): RuntimeBranchHandle;
  switch_branch(branchId: bigint): void;
  replay_for_branch(branchId: bigint): ReplaySummary;
  branch_snapshot(branchId: bigint): RuntimeSnapshotArtifact;
  branch_snapshot_wire(branchId: bigint): string;
  branch_snapshot_portable_wire(branchId: bigint): string;
  branch_snapshot_id(branchId: bigint): bigint;
  branch_snapshot_envelope(branchId: bigint): RuntimeSnapshotEnvelope;
  branch_snapshot_envelope_wire(branchId: bigint): string;
  branch_snapshot_envelope_portable_wire(branchId: bigint): string;
  restore_branch_snapshot(branchId: bigint, snapshot: RuntimeSnapshotArtifact): void;
  restore_branch_snapshot_wire(branchId: bigint, snapshot: string): void;
  restore_branch_snapshot_portable_wire(branchId: bigint, snapshot: string): void;
  restore_branch_snapshot_by_id(branchId: bigint, snapshotId: bigint): void;
  merge_branches(sourceBranchId: bigint, targetBranchId: bigint): MergeResultArtifact;
  merge_branches_with_proof(sourceBranchId: bigint, targetBranchId: bigint): MergeResultProofEnvelope;
  plan_merge_branches(sourceBranchId: bigint, targetBranchId: bigint): MergePlanArtifact;
  plan_merge_branches_with_proof(sourceBranchId: bigint, targetBranchId: bigint): MergePlanProofEnvelope;
  plan_merge_policy_preview(request: MergePolicyPreviewRequest): MergePlanArtifact;
  plan_merge_policy_preview_with_proof(request: MergePolicyPreviewRequest): MergePlanProofEnvelope;
  merge_branches_policy_preview(request: MergePolicyPreviewRequest): MergeResultArtifact;
  merge_branches_policy_preview_with_proof(request: MergePolicyPreviewRequest): MergeResultProofEnvelope;
  branch_state_proof(branchId: bigint): BranchStateProofReport;
  replay_parity_proof(expectedBranchId: bigint, replayedBranchId: bigint): ReplayParityProofReport;
  replay_artifact_proof(expected: ReplayArtifactProofInput, replayedBranchId: bigint): ReplayArtifactProofReport;
}

export class SignalSpecialist {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  evaluate_dirty(): RunSummary;
  graph_summary(): GraphSummary;
  read_versions(ids: ReadonlyArray<string>): ReadonlyArray<VersionSummary>;
}

export class SignalAdapters {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  export_definitions(): RuntimeDefinitionEnvelope;
  export_runtime_envelope(): RuntimeEnvelope;
  export_runtime_envelope_wire(): string;
  discard_restore_token(token: string): boolean;
  export_runtime_envelope_portable_wire(): string;
  replace_runtime_envelope(envelope: RuntimeEnvelope): void;
  replace_runtime_envelope_wire(envelope: string): void;
  replace_runtime_envelope_portable_wire(envelope: string): void;
  runtime_proof_report(): RuntimeProofReport;
}

export class SignalWorkerRuntime {
  constructor();
  free(): void;
  [Symbol.dispose](): void;
  bootstrapRecord(): WorkerRuntimeBootstrapRecord;
  workerRuntimeShellLock(): WorkerRuntimeShellLock;
  publishPortableGraph(publication: WorkerPortableGraphPublication): WorkerGraphPublicationSummary;
  applyTransaction(transactionOps: ReadonlyArray<TransactionOp>): WorkerCommittedTransactionEnvelope;
  applyTransactionProjection(
    request: WorkerCommittedProjectionRequest,
  ): WorkerCommittedProjectionPacket;
  admitHostCapabilityIngress(batch: WorkerHostCapabilityIngressBatch): WorkerHostCapabilityIngressReport;
  admitBrowserHistoryIngress(ingress: WorkerBrowserHistoryIngress): WorkerBrowserHistoryIngressReport;
  issueHostEffectRequest(request: WorkerHostEffectRequest): WorkerHostEffectRequestEnvelope;
  admitHostEffectAcknowledgement(
    acknowledgement: WorkerHostEffectAcknowledgement,
  ): WorkerHostEffectAcknowledgementReport;
  certifyMainThreadHostBridge(): WorkerMainThreadHostBridgeCertificationPackage;
  attachObservationDelivery(request: WorkerObservationDeliveryAttachRequest): WorkerLifecycleControlPacket;
  detachObservationDelivery(request: WorkerObservationDeliveryDetachRequest): WorkerLifecycleControlPacket;
  why(id: string): WhySummary;
  health(): HealthSummary;
  latestFlow(): FlowSurfaceSummary | null;
  latestObservation(): ObservationSurfaceSummary | null;
  performanceSummary(): WebPerformanceSummary;
  latestFailure(): FailureSummary | null;
  latestRollback(): RollbackDiagnostic | null;
  latestInvalidationPlanningEstimate(): InvalidationPlanningEstimate | null;
  latestInvalidationTraceRecords(): ReadonlyArray<InvalidationTraceRecord>;
  recentHistory(): ReadonlyArray<ExecutionHistorySummary>;
  currentBranch(): RuntimeBranchHandle;
  branches(): ReadonlyArray<RuntimeBranchHandle>;
  replayForBranch(branchId: bigint): ReplaySummary;
  branchSnapshotId(branchId: bigint): bigint;
  branchSnapshotEnvelope(branchId: bigint): RuntimeSnapshotEnvelope;
  branchSnapshotArtifact(branchId: bigint): {
    snapshot: RuntimeSnapshotArtifact;
    snapshotRestoreToken: string;
    snapshotPortableWire: string;
  };
  branchSnapshotEnvelopeArtifact(branchId: bigint): {
    snapshotEnvelope: RuntimeSnapshotEnvelope;
    snapshotEnvelopeRestoreToken: string;
    snapshotEnvelopePortableWire: string;
  };
  branchSnapshotEnvelopeWire(branchId: bigint): string;
  branchSnapshotEnvelopePortableWire(branchId: bigint): string;
  branchStateProof(branchId: bigint): BranchStateProofReport;
  replayFor(id: string): ReplaySummary;
  lineageFor(id: string): LineageSummary;
  readVersions(ids: ReadonlyArray<string>): ReadonlyArray<VersionSummary>;
  evaluateDirty(): RunSummary;
  exportDefinitions(): RuntimeDefinitionEnvelope;
  exportWorkerRuntimeEnvelope(): RuntimeEnvelope;
  exportWorkerSnapshotEnvelope(): RuntimeSnapshotEnvelope;
  exportWorkerSnapshotEnvelopeArtifact(): {
    snapshotEnvelope: RuntimeSnapshotEnvelope;
    snapshotEnvelopeRestoreToken: string;
    snapshotEnvelopePortableWire: string;
  };
  exportWorkerSnapshotEnvelopeWire(): string;
  discardRestoreToken(token: string): boolean;
  exportWorkerSnapshotEnvelopePortableWire(): string;
  exportWorkerRuntimeEnvelopePortableWire(): string;
  admitWorkerRuntimeEnvelopeImport(envelope: RuntimeEnvelope): unknown;
  admitWorkerRuntimeEnvelopeImportPortableWire(envelope: string): unknown;
  runtimeProofReport(): RuntimeProofReport;
  deliverLatestObservation(): WorkerObservationDeliveryPacket;
  deliverOutputs(request: WorkerOutputDeliveryRequest): WorkerOutputDeliveryPacket;
  readSignals(request: WorkerSignalReadbackRequest): WorkerSignalReadbackPacket;
  readDiagnosticsSummary(): WorkerDiagnosticsSummaryReadPacket;
  readDiagnosticsHistory(): WorkerDiagnosticsHistoryReadPacket;
}

export class SignalApp {
  constructor();
  free(): void;
  [Symbol.dispose](): void;
  source(spec: SourceSpec): void;
  recipe(spec: RecipeSpec): void;
  source_family(spec: KeyedSourceFamilySpec): void;
  recipe_family(spec: KeyedRecipeFamilySpec): void;
  batch(ops: ReadonlyArray<TransactionOp>): RunSummary;
  transaction_with_packed_grid_rgba(
    prefixOps: unknown,
    familyId: string,
    width: number,
    height: number,
    rgba: Uint8Array,
    suffixOps: unknown,
  ): RunSummary;
  read(id: string): SignalValue;
  read_many(ids: ReadonlyArray<string>): ReadonlyArray<SignalValue>;
  read_keyed(familyId: string, key: string): SignalValue;
  set_keyed(familyId: string, key: string, value: SignalValue): RunSummary;
  setKeyedWithAspects(
    familyId: string,
    key: string,
    value: SignalValue,
    aspects: ReadonlyArray<AspectId>,
  ): RunSummary;
  read_keyed_many(familyId: string, keys: ReadonlyArray<string>): ReadonlyArray<SignalValue>;
  read_keyed_many_packed_fields(
    familyId: string,
    keys: ReadonlyArray<string>,
    fields: ReadonlyArray<string>,
  ): Uint8Array;
  read_keyed_grid_packed_fields(
    familyId: string,
    columns: number,
    rows: number,
    fields: ReadonlyArray<string>,
  ): Uint8Array;
  read_keyed_rect_packed_fields(
    familyId: string,
    columns: number,
    rows: number,
    row: number,
    startColumn: number,
    width: number,
    height: number,
    fields: ReadonlyArray<string>,
  ): Uint8Array;
  prewarm_keyed_grid(familyId: string, columns: number, rows: number): void;
  seed_keyed_grid_coords(familyId: string, columns: number, rows: number): void;
  take_debug_events(): ReadonlyArray<string>;
  set_keyed_many(familyId: string, values: ReadonlyArray<KeyedSetValue>): RunSummary;
  mark_changed_with_regions(id: string, changedRegions: unknown): RunSummary;
  markChanged(id: string, aspects: ReadonlyArray<AspectId>): RunSummary;
  markChangedWithRegionsAndAspects(
    id: string,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): RunSummary;
  mark_keyed_changed_with_regions(
    familyId: string,
    key: string,
    changedRegions: unknown,
  ): RunSummary;
  markKeyedChanged(familyId: string, key: string, aspects: ReadonlyArray<AspectId>): RunSummary;
  diagnostics(): SignalDiagnostics;
  history(): SignalHistory;
  specialist(): SignalSpecialist;
  adapters(): SignalAdapters;
}

export class SignalRuntime {
  constructor();
  free(): void;
  [Symbol.dispose](): void;
  define_source(spec: SourceSpec): void;
  define_recipe(spec: RecipeSpec): void;
  define_source_family(spec: KeyedSourceFamilySpec): void;
  define_recipe_family(spec: KeyedRecipeFamilySpec): void;
  read(id: string): SignalValue;
  read_many(ids: ReadonlyArray<string>): ReadonlyArray<SignalValue>;
  read_keyed(familyId: string, key: string): SignalValue;
  read_keyed_many(familyId: string, keys: ReadonlyArray<string>): ReadonlyArray<SignalValue>;
  read_keyed_many_packed_fields(
    familyId: string,
    keys: ReadonlyArray<string>,
    fields: ReadonlyArray<string>,
  ): Uint8Array;
  read_keyed_grid_packed_fields(
    familyId: string,
    columns: number,
    rows: number,
    fields: ReadonlyArray<string>,
  ): Uint8Array;
  read_keyed_rect_packed_fields(
    familyId: string,
    columns: number,
    rows: number,
    row: number,
    startColumn: number,
    width: number,
    height: number,
    fields: ReadonlyArray<string>,
  ): Uint8Array;
  prewarm_keyed_grid(familyId: string, columns: number, rows: number): void;
  seed_keyed_grid_coords(familyId: string, columns: number, rows: number): void;
  set_keyed(familyId: string, key: string, value: SignalValue): RunSummary;
  setKeyedWithAspects(
    familyId: string,
    key: string,
    value: SignalValue,
    aspects: ReadonlyArray<AspectId>,
  ): RunSummary;
  set_keyed_many(familyId: string, values: ReadonlyArray<KeyedSetValue>): RunSummary;
  clear_keyed_family_cache(familyId: string): void;
  mark_changed_with_regions(id: string, changedRegions: unknown): RunSummary;
  markChanged(id: string, aspects: ReadonlyArray<AspectId>): RunSummary;
  markChangedWithRegionsAndAspects(
    id: string,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): RunSummary;
  mark_keyed_changed_with_regions(
    familyId: string,
    key: string,
    changedRegions: unknown,
  ): RunSummary;
  markKeyedChanged(familyId: string, key: string, aspects: ReadonlyArray<AspectId>): RunSummary;
  set_runtime_policy(policy: RuntimePolicySpec): void;
  transaction(ops: ReadonlyArray<TransactionOp>): RunSummary;
  transaction_with_packed_grid_rgba(
    prefixOps: unknown,
    familyId: string,
    width: number,
    height: number,
    rgba: Uint8Array,
    suffixOps: unknown,
  ): RunSummary;
  take_debug_events(): ReadonlyArray<string>;
  diagnostics(): SignalDiagnostics;
  history(): SignalHistory;
  specialist(): SignalSpecialist;
  adapters(): SignalAdapters;
}
