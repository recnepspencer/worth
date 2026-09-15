import type {
  BranchStateProofReport,
  ExecutionHistorySummary,
  LineageSummary,
  FailureSummary,
  FlowSurfaceSummary,
  InvalidationPlanningEstimate,
  ExecutionHistorySurfaceSummary,
  GraphSummary,
  HealthSummary,
  HostCapabilityTransportReport,
  InvalidationTraceRecord,
  ObservationSurfaceSummary,
  ReplayArtifactProofInput,
  ReplayArtifactProofReport,
  ReplayParityProofReport,
  ReplaySummary,
  RollbackDiagnostic,
  RuntimeDefinitionEnvelope,
  RuntimeEnvelope,
  RuntimeProofReport,
  RuntimeBranchHandle,
  RuntimeSnapshotArtifact,
  RuntimeSnapshotEnvelope,
  RuntimePolicySpec,
  MergePlanArtifact,
  MergePlanProofEnvelope,
  MergePolicyPreviewRequest,
  MergeResultArtifact,
  MergeResultProofEnvelope,
  WebPerformanceSummary,
  WhySummary,
} from "./diagnostics.js";
import type {
  RecipeSpec,
  RunSummary,
  SourceSpec,
  TransactionOp,
  VersionSummary,
} from "./model.js";
import type {
  WorkerBrowserHistoryIngress,
  WorkerBrowserHistoryIngressReport,
  WorkerBrowserHistoryStory,
  WorkerBrowserHistoryWriteback,
  WorkerBrowserHistoryWritebackReport,
  WorkerHostBoundaryCausality,
  WorkerHostBoundaryPerformanceEnvelope,
  WorkerHostCapabilityBoundaryArtifact,
  WorkerHostCapabilityIngressBatch,
  WorkerHostCapabilityIngressReport,
  WorkerHostCapabilityUpdate,
  WorkerHostEffectAcknowledgement,
  WorkerHostEffectAcknowledgementReport,
  WorkerHostEffectOutcome,
  WorkerHostEffectRequest,
  WorkerHostEffectRequestEnvelope,
} from "./worker_runtime_bridge_boundary.js";
import type {
  WorkerCommittedProjectionPacket,
  WorkerCommittedProjectionRequest,
  WorkerCommittedTransactionEnvelope,
  WorkerDiagnosticsHistoryReadPacket,
  WorkerDiagnosticsSummaryReadPacket,
  WorkerDeliveredOutput,
  WorkerLifecycleControlPacket,
  WorkerObservationDeliveryAttachRequest,
  WorkerObservationDeliveryDetachRequest,
  WorkerObservationDeliveryPacket,
  WorkerOutputDeliveryPacket,
  WorkerOutputDeliveryRequest,
  WorkerReadbackSignal,
  WorkerSignalReadbackPacket,
  WorkerSignalReadbackRequest,
} from "./worker_runtime_bridge_delivery.js";

export type * from "./worker_runtime_bridge_boundary.js";
export type * from "./worker_runtime_bridge_delivery.js";

export interface WorkerRuntimeIdentity {
  deploymentPosture: "workerFirst";
  runtimeAuthority: "workerOwnedRuntime";
  topology: string;
}

export interface WorkerRuntimeShellLock {
  identity: WorkerRuntimeIdentity;
  graphPublicationAdmission: string;
  committedEnvelopeFamily: string;
  callbackPublicationBeforeLowering: string;
}

export interface WorkerRuntimeBootstrapRecord {
  shellLock: WorkerRuntimeShellLock;
  boundarySurface: "workerFirstConstruction";
  transportPosture: string;
  hostCapabilityIngress: string;
  hostEffectEgress: string;
}

export interface WorkerFirstHostCapabilityDiagnosticsReport {
  posture: "workerFirstUnavailable";
  reason: "workerFirstHostCapabilityEventReplayNotImplemented";
  message: string;
}

export interface WorkerFirstHostCapabilityDiagnosticsEvent {
  kind: "unavailable";
  reason: WorkerFirstHostCapabilityDiagnosticsReport["reason"];
  message: string;
}

export interface WorkerPortableGraphPublication {
  policy: RuntimePolicySpec;
  sources: ReadonlyArray<SourceSpec>;
  recipes: ReadonlyArray<RecipeSpec>;
  outputIds?: ReadonlyArray<string>;
}

export interface WorkerGraphPublicationSummary {
  publishedSourceCount: number;
  publishedRecipeCount: number;
  admittedCallbackCount: number;
  deniedCallbackCount: number;
}

export interface WorkerSnapshotEnvelopeArtifact {
  snapshotEnvelope: RuntimeSnapshotEnvelope;
  snapshotEnvelopeRestoreToken: string;
  snapshotEnvelopePortableWire: string;
}

export interface WorkerSnapshotArtifact {
  snapshot: RuntimeSnapshotArtifact;
  snapshotRestoreToken: string;
  snapshotPortableWire: string;
}

export interface WorkerRuntimeEnvelopeImportReport {
  envelopeFamily: "workerRuntimeEnvelopeImport";
  importOutcome: string;
  rejectedCallbackCount: number;
  reattachedCallbackCount: number;
  hostCapabilityTransportCount: number;
  fallbackCount: number;
  rejectedCallbackIds: ReadonlyArray<string>;
  reattachedCallbackIds: ReadonlyArray<string>;
  workerFirstTruthDigest: string;
  importDigest: string;
}

export interface WorkerFirstRuntimeEnvelopeArtifact extends RuntimeEnvelope {
  runtimeEnvelopeRestoreToken: string;
  runtimeEnvelopeRestoreMode: "SameRuntimeExact";
  runtimeEnvelopePortableWire: string;
}

export interface WorkerMainThreadHostBridgeCertificationPackage {
  certificationFamily: "mainThreadHostBridgeCertification";
  coveredSuiteCount: number;
  hostCapabilityEnvelopeDigest: string;
  hostCapabilityLifecycleDigest: string;
  hostCapabilityTruthDigest: string;
  hostCapabilityCoalescingDigest: string;
  hostCapabilityArtifactDigest: string;
  browserHistoryEnvelopeDigest: string;
  browserHistoryRouteTruthDigest: string;
  browserHistoryContinuityDigest: string;
  browserHistoryReplayRestoreDigest: string;
  hostEffectRequestDigest: string;
  hostEffectAcknowledgedRequestDigest: string;
  hostEffectAcknowledgementDigest: string;
  hostEffectLifecycleArtifact: string;
  hostEffectLifecycleIntegrityDigest: string;
  WorthProofReadmissionDigest: string;
  hostBoundaryCausalityDigest: string;
  boundaryPerformanceDigest: string;
  workerFirstTruthDigest: string;
  ambientHostReadDenied: boolean;
  hostAcknowledgementIsAuthoritative: boolean;
  certificationDigest: string;
}

export interface CreateWorkerRuntimeBridgeOptions {
  workerUrl?: string | URL;
  /** Optional explicit WASM URL forwarded to the worker before runtime bootstrap. */
  wasmUrl?: string | URL;
}

export interface WorkerRuntimeBridge {
  bootstrapRecord(): Promise<WorkerRuntimeBootstrapRecord>;
  workerRuntimeShellLock(): Promise<WorkerRuntimeShellLock>;
  publishPortableGraph(
    publication: WorkerPortableGraphPublication,
  ): Promise<WorkerGraphPublicationSummary>;
  applyTransaction(
    transactionOps: ReadonlyArray<TransactionOp>,
  ): Promise<WorkerCommittedTransactionEnvelope>;
  applyTransactionProjection(
    request: WorkerCommittedProjectionRequest,
  ): Promise<WorkerCommittedProjectionPacket>;
  attachObservationDelivery(
    request: WorkerObservationDeliveryAttachRequest,
  ): Promise<WorkerLifecycleControlPacket>;
  detachObservationDelivery(
    request: WorkerObservationDeliveryDetachRequest,
  ): Promise<WorkerLifecycleControlPacket>;
  why(id: string): Promise<WhySummary>;
  health(): Promise<HealthSummary>;
  latestFlow(): Promise<FlowSurfaceSummary | null>;
  latestObservation(): Promise<ObservationSurfaceSummary | null>;
  performanceSummary(): Promise<WebPerformanceSummary>;
  latestFailure(): Promise<FailureSummary | null>;
  latestRollback(): Promise<RollbackDiagnostic | null>;
  latestInvalidationPlanningEstimate(): Promise<InvalidationPlanningEstimate | null>;
  latestInvalidationTraceRecords(): Promise<ReadonlyArray<InvalidationTraceRecord>>;
  recentHistory(): Promise<ReadonlyArray<ExecutionHistorySummary>>;
  currentBranch(): Promise<RuntimeBranchHandle>;
  branches(): Promise<ReadonlyArray<RuntimeBranchHandle>>;
  createBranch(name: string): Promise<RuntimeBranchHandle>;
  switchBranch(branchId: bigint | number): Promise<unknown>;
  planMergeBranches(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergePlanArtifact>;
  planMergeBranchesWithProof(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergePlanProofEnvelope>;
  mergeBranches(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergeResultArtifact>;
  mergeBranchesWithProof(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergeResultProofEnvelope>;
  planMergePolicyPreview(request: MergePolicyPreviewRequest): Promise<MergePlanArtifact>;
  planMergePolicyPreviewWithProof(
    request: MergePolicyPreviewRequest,
  ): Promise<MergePlanProofEnvelope>;
  mergeBranchesPolicyPreview(request: MergePolicyPreviewRequest): Promise<MergeResultArtifact>;
  mergeBranchesPolicyPreviewWithProof(
    request: MergePolicyPreviewRequest,
  ): Promise<MergeResultProofEnvelope>;
  replayForBranch(branchId: bigint | number): Promise<ReplaySummary>;
  branchSnapshotId(branchId: bigint | number): Promise<bigint | number>;
  branchSnapshotEnvelope(branchId: bigint | number): Promise<RuntimeSnapshotEnvelope>;
  branchSnapshotArtifact(branchId: bigint | number): Promise<WorkerSnapshotArtifact>;
  branchSnapshotEnvelopeArtifact(
    branchId: bigint | number,
  ): Promise<WorkerSnapshotEnvelopeArtifact>;
  branchSnapshotEnvelopeWire(branchId: bigint | number): Promise<string>;
  branchSnapshotEnvelopePortableWire(branchId: bigint | number): Promise<string>;
  restoreBranchSnapshotArtifact(
    branchId: bigint | number,
    snapshot: RuntimeSnapshotArtifact,
  ): Promise<unknown>;
  restoreBranchSnapshotWire(branchId: bigint | number, snapshot: string): Promise<unknown>;
  restoreBranchSnapshotPortableWire(branchId: bigint | number, snapshot: string): Promise<unknown>;
  restoreBranchSnapshotById(branchId: bigint | number, snapshotId: bigint | number): Promise<unknown>;
  branchStateProof(branchId: bigint | number): Promise<BranchStateProofReport>;
  replayFor(id: string): Promise<ReplaySummary>;
  lineageFor(id: string): Promise<LineageSummary>;
  readVersions(ids: ReadonlyArray<string>): Promise<ReadonlyArray<VersionSummary>>;
  evaluateDirty(): Promise<RunSummary>;
  exportDefinitions(): Promise<RuntimeDefinitionEnvelope>;
  exportWorkerRuntimeEnvelope(): Promise<RuntimeEnvelope>;
  exportWorkerSnapshotEnvelope(): Promise<RuntimeSnapshotEnvelope>;
  exportWorkerSnapshotEnvelopeArtifact(): Promise<WorkerSnapshotEnvelopeArtifact>;
  exportWorkerSnapshotEnvelopeWire(): Promise<string>;
  exportWorkerSnapshotEnvelopePortableWire(): Promise<string>;
  restoreSnapshotEnvelope(snapshot: RuntimeSnapshotEnvelope): Promise<unknown>;
  restoreSnapshotEnvelopeWire(snapshot: string): Promise<unknown>;
  restoreSnapshotEnvelopePortableWire(snapshot: string): Promise<unknown>;
  exportWorkerRuntimeEnvelopeWire(): Promise<string>;
  exportWorkerRuntimeEnvelopePortableWire(): Promise<string>;
  discardRestoreToken(token: string): Promise<boolean>;
  admitWorkerRuntimeEnvelopeImportWire(
    envelope: string,
  ): Promise<WorkerRuntimeEnvelopeImportReport>;
  admitWorkerRuntimeEnvelopeImportPortableWire(
    envelope: string,
  ): Promise<WorkerRuntimeEnvelopeImportReport>;
  runtimeProofReport(): Promise<RuntimeProofReport>;
  admitHostCapabilityIngress(
    batch: WorkerHostCapabilityIngressBatch,
  ): Promise<WorkerHostCapabilityIngressReport>;
  admitBrowserHistoryIngress(
    ingress: WorkerBrowserHistoryIngress,
  ): Promise<WorkerBrowserHistoryIngressReport>;
  applyBrowserHistoryWriteback(
    writeback: WorkerBrowserHistoryWriteback,
  ): Promise<WorkerBrowserHistoryWritebackReport>;
  browserHistoryStory(
    initialReport?: WorkerBrowserHistoryIngressReport | WorkerBrowserHistoryWritebackReport,
  ): WorkerBrowserHistoryStory;
  issueHostEffectRequest(
    request: WorkerHostEffectRequest,
  ): Promise<WorkerHostEffectRequestEnvelope>;
  admitHostEffectAcknowledgement(
    acknowledgement: WorkerHostEffectAcknowledgement,
  ): Promise<WorkerHostEffectAcknowledgementReport>;
  certifyMainThreadHostBridge(): Promise<WorkerMainThreadHostBridgeCertificationPackage>;
  deliverLatestObservation(): Promise<WorkerObservationDeliveryPacket>;
  deliverOutputs(request: WorkerOutputDeliveryRequest): Promise<WorkerOutputDeliveryPacket>;
  readSignals(request: WorkerSignalReadbackRequest): Promise<WorkerSignalReadbackPacket>;
  readDiagnosticsSummary(): Promise<WorkerDiagnosticsSummaryReadPacket>;
  readDiagnosticsHistory(): Promise<WorkerDiagnosticsHistoryReadPacket>;
  terminate(): Promise<void>;
}

export interface WorkerFirstDiagnosticsFacade {
  why(id: string): Promise<WhySummary>;
  health(): Promise<HealthSummary>;
  summaryNow(): Promise<GraphSummary>;
  historyNow(): Promise<ExecutionHistorySurfaceSummary>;
  latestFlow(): Promise<FlowSurfaceSummary | null>;
  latestObservation(): Promise<ObservationSurfaceSummary | null>;
  latestHostCapabilityEvent(): WorkerFirstHostCapabilityDiagnosticsEvent | null;
  recentHostCapabilityEvents(): ReadonlyArray<WorkerFirstHostCapabilityDiagnosticsEvent>;
  performanceSummary(): Promise<WebPerformanceSummary>;
  hostCapabilityReport(): Promise<WorkerFirstHostCapabilityDiagnosticsReport>;
  latestFailure(): Promise<FailureSummary | null>;
  latestRollback(): Promise<RollbackDiagnostic | null>;
  latestInvalidationPlanningEstimate(): Promise<InvalidationPlanningEstimate | null>;
  latestInvalidationTraceRecords(): Promise<ReadonlyArray<InvalidationTraceRecord>>;
  recentHistory(): Promise<ReadonlyArray<ExecutionHistorySummary>>;
}

export interface WorkerFirstHistoryFacade {
  replay_for(id: string): Promise<ReplaySummary>;
  lineage_for(id: string): Promise<LineageSummary>;
  recentHistory(): Promise<ReadonlyArray<ExecutionHistorySummary>>;
  snapshot(): Promise<WorkerSnapshotEnvelopeArtifact>;
  restore_snapshot(snapshot: RuntimeSnapshotEnvelope): Promise<unknown>;
  restore_exact_snapshot(snapshot: WorkerSnapshotEnvelopeArtifact): Promise<unknown>;
  discard_exact_artifact(
    artifact: WorkerSnapshotEnvelopeArtifact | WorkerSnapshotArtifact,
  ): Promise<boolean>;
  current_branch(): Promise<RuntimeBranchHandle>;
  branches(): Promise<ReadonlyArray<RuntimeBranchHandle>>;
  create_branch(name: string): Promise<RuntimeBranchHandle>;
  switch_branch(branchId: bigint | number): Promise<unknown>;
  replay_for_branch(branchId: bigint | number): Promise<ReplaySummary>;
  branch_snapshot(branchId: bigint | number): Promise<WorkerSnapshotArtifact>;
  branch_snapshot_id(branchId: bigint | number): Promise<bigint | number>;
  branch_snapshot_envelope(branchId: bigint | number): Promise<WorkerSnapshotEnvelopeArtifact>;
  restore_branch_snapshot(branchId: bigint | number, snapshot: RuntimeSnapshotArtifact): Promise<unknown>;
  restore_exact_branch_snapshot(
    branchId: bigint | number,
    snapshot: WorkerSnapshotArtifact,
  ): Promise<unknown>;
  restore_branch_snapshot_by_id(branchId: bigint | number, snapshotId: bigint | number): Promise<unknown>;
  plan_merge_branches(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergePlanArtifact>;
  plan_merge_branches_with_proof(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergePlanProofEnvelope>;
  merge_branches(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergeResultArtifact>;
  merge_branches_with_proof(
    sourceBranchId: bigint | number,
    targetBranchId: bigint | number,
  ): Promise<MergeResultProofEnvelope>;
  plan_merge_policy_preview(request: MergePolicyPreviewRequest): Promise<MergePlanArtifact>;
  plan_merge_policy_preview_with_proof(
    request: MergePolicyPreviewRequest,
  ): Promise<MergePlanProofEnvelope>;
  merge_branches_policy_preview(request: MergePolicyPreviewRequest): Promise<MergeResultArtifact>;
  merge_branches_policy_preview_with_proof(
    request: MergePolicyPreviewRequest,
  ): Promise<MergeResultProofEnvelope>;
  branch_state_proof(branchId: bigint | number): Promise<BranchStateProofReport>;
  replay_parity_proof(
    expectedBranchId: bigint | number,
    replayedBranchId: bigint | number,
  ): Promise<ReplayParityProofReport>;
  replay_artifact_proof(
    expected: ReplayArtifactProofInput,
    replayedBranchId: bigint | number,
  ): Promise<ReplayArtifactProofReport>;
}

export interface WorkerFirstAdaptersFacade {
  exportDefinitions(): Promise<RuntimeDefinitionEnvelope>;
  exportRuntimeEnvelope(): Promise<WorkerFirstRuntimeEnvelopeArtifact>;
  replaceRuntimeEnvelope(
    envelope: WorkerFirstRuntimeEnvelopeArtifact | RuntimeEnvelope,
  ): Promise<WorkerRuntimeEnvelopeImportReport>;
  restoreExactRuntimeEnvelope(
    envelope: WorkerFirstRuntimeEnvelopeArtifact,
  ): Promise<WorkerRuntimeEnvelopeImportReport>;
  discardExactRuntimeEnvelope(envelope: WorkerFirstRuntimeEnvelopeArtifact): Promise<boolean>;
  runtimeProofReport(): Promise<RuntimeProofReport>;
  hostCapabilityTransportReport(
    envelope?: WorkerFirstRuntimeEnvelopeArtifact | RuntimeEnvelope,
  ): Promise<HostCapabilityTransportReport>;
}

export function createWorkerRuntimeBridge(
  options?: CreateWorkerRuntimeBridgeOptions,
): WorkerRuntimeBridge;

export function createWorkerFirstDiagnosticsFacade(session: {
  bridge: WorkerRuntimeBridge;
}): WorkerFirstDiagnosticsFacade;

export function createWorkerFirstHistoryFacade(session: {
  bridge: WorkerRuntimeBridge;
}): WorkerFirstHistoryFacade;

export function createWorkerFirstAdaptersFacade(session: {
  bridge: WorkerRuntimeBridge;
}): WorkerFirstAdaptersFacade;
