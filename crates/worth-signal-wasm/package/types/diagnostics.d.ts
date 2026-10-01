import type {
  KeyedRecipeFamilySpec,
  KeyedSourceFamilySpec,
  RecipeSpec,
  SignalValue,
  SourceSpec,
} from "./model.js";

export type DiagnosticsTier = "Operational" | "Development" | "Forensic" | string;
export type NodeIdSummary = string;
export type DiagnosticsCounterMap = Readonly<Record<string, number>>;
export type DiagnosticsVariant = string | Readonly<Record<string, unknown>>;
export type ArtifactRetentionPolicy = "Retain" | "Reconstruct" | "Omit";
export type ReplayDetailPolicy = "Minimal" | "Standard" | "Forensic";
export type SemanticRetentionPolicy = "Minimal" | "Development" | "Forensic";
export type SnapshotRestoreLineageMode = "CompactGlobal" | "PerNode";
export type FrontierTracingPolicy = "SummaryOnly" | "RetainWaveRecords" | "FullForensic";
export type FrontierPropagationPolicy = "CanonicalFrontier";
export type FrontierCyclePolicy = "ReachableCycleCheck";
export type ExecutionObjectiveProfile = "LatencyBounded" | "Balanced" | "Throughput";
export type ObservationActivationProfile = "OnDemand" | "Continuous";
export type HostCapabilityCompatibility =
  | "LiveOnly"
  | "Reattachable"
  | "SnapshotPortable"
  | "ImportDenied";
export type RuntimePolicyPreset =
  | "development"
  | "operational"
  | "forensic"
  | "webDevelopment"
  | "fintech"
  | "kernel"
  | "gameEngine";

export interface RuntimePolicySpec {
  preset: RuntimePolicyPreset;
}

export interface RetentionBudgetSummary {
  history_limit: number;
  detail_limit: number;
  retain_history_details: boolean;
  retain_flow_explanation: boolean;
  retain_latest_failure_context: boolean;
  retain_stage_details: boolean;
  capture_forensic_failure_context: boolean;
  explanation_retention: ArtifactRetentionPolicy;
  provenance_retention: ArtifactRetentionPolicy;
  replay_detail: ReplayDetailPolicy;
  semantic_detail: SemanticRetentionPolicy;
}

export interface ReconstructionBudgetSummary {
  allow_explanation_reconstruction: boolean;
  allow_provenance_reconstruction: boolean;
  allow_replay_reconstruction: boolean;
  allow_certification_materialization: boolean;
}

export interface ParallelAdmissionPolicySummary {
  throughput_min_parallel_tasks: number;
  balanced_min_parallel_tasks: number;
  latency_bounded_min_parallel_tasks: number;
  full_parallel_min_tasks: number;
}

export interface SignalRuntimePolicySummary {
  tier: DiagnosticsTier;
  execution_objective: ExecutionObjectiveProfile;
  observation_activation: ObservationActivationProfile;
  retention_budget: RetentionBudgetSummary;
  reconstruction_budget: ReconstructionBudgetSummary;
  snapshot_restore_lineage_mode: SnapshotRestoreLineageMode;
  frontier_tracing_policy: FrontierTracingPolicy;
  frontier_propagation_policy: FrontierPropagationPolicy;
  frontier_cycle_policy: FrontierCyclePolicy;
  parallel_admission: ParallelAdmissionPolicySummary;
}

export interface HealthSummary {
  activeNodeCount: number;
  cleanNodeCount: number;
  maybeStaleNodeCount: number;
  dirtyNodeCount: number;
  dependencyEdgeCount: number;
  subscriberEdgeCount: number;
}

export interface WhySummary {
  id: string;
  node: string;
  apiFamily: string | null;
  recipeFamily: string | null;
  state: string;
  upstream: ReadonlyArray<string>;
  changedRegions: ReadonlyArray<string>;
  propagationSuppressed: boolean;
  outputChange: string | null;
  outputIdentity: string | null;
  callback: CallbackWhySummary | null;
}

export interface CallbackWhySummary {
  purityPosture: string;
  currentReads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
  registered: boolean;
  unavailableReason?: string;
  tokenSlot?: number;
  tokenGeneration?: number;
  lastRuntimeReadBreadth: number;
  lastDependencyPatch: CallbackDependencyPatchSummary | null;
  lastFailure: CallbackFailureSummary | null;
}

export interface CallbackRuntimeNodeSummary {
  id: string;
  node: string;
  apiFamily: string | null;
  recipeFamily: string | null;
  purityPosture: string;
  currentReads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
  registered: boolean;
  unavailableReason?: string;
  tokenSlot?: number;
  tokenGeneration?: number;
  lastRuntimeReadBreadth: number;
  lastDependencyPatch: CallbackDependencyPatchSummary | null;
  lastFailure: CallbackFailureSummary | null;
}

export interface CallbackDependencyPatchSummary {
  previousReads: ReadonlyArray<string>;
  currentReads: ReadonlyArray<string>;
  addedCount: number;
  removedCount: number;
  retainedCount: number;
  runtimeReadBreadth: number;
}

export interface CallbackFailureSummary {
  class: string;
  message: string;
  code: string | null;
}

export interface HostCapabilityReadSummary {
  family: string;
  registrationId: string;
  compatibility: HostCapabilityCompatibility;
}

export type HostCapabilityEventKind =
  | "InvalidationFlushed"
  | "InvalidationNoOpSuppressed"
  | "InvalidationIgnoredStale"
  | "PortableImportDenied";

export type HostCapabilityInvalidationMode = "push-driven" | "polled" | "manually-committed";

export type HostCapabilityPortableImportOutcome =
  | "Live"
  | "Reattached"
  | "Unavailable"
  | "Incompatible"
  | "Denied";

export interface HostCapabilityTransportArtifact {
  family: string;
  registrationId: string;
  compatibility: HostCapabilityCompatibility;
  exactRestoreOutcome: "Live" | "Reattached" | "Unavailable" | "Incompatible";
  portableImportOutcome: HostCapabilityPortableImportOutcome;
  portableImportReason: string;
}

export interface HostCapabilityDiagnosticsEvent {
  sequence: number;
  kind: HostCapabilityEventKind;
  family: string;
  registrationId: string;
  compatibility: HostCapabilityCompatibility;
  invalidationMode: HostCapabilityInvalidationMode | null;
  queuedInvalidationCount: number;
  previousState: SignalValue | null;
  nextState: SignalValue | null;
  touchedNodes: number;
  reevaluatedNodes: number;
  portableImportOutcome?: HostCapabilityPortableImportOutcome;
  portableImportReason?: string;
  deniedCallbackIds?: ReadonlyArray<string>;
}

export interface HostCapabilityDiagnosticsFamilyReport {
  family: string;
  eventCount: number;
  latestKind: HostCapabilityEventKind | null;
  latestCompatibility: HostCapabilityCompatibility | null;
  invalidationModes: ReadonlyArray<HostCapabilityInvalidationMode>;
  maxQueuedInvalidationCount: number;
  maxTouchedNodes: number;
  maxReevaluatedNodes: number;
  deniedCallbackIds: ReadonlyArray<string>;
}

export interface HostCapabilityLineageEntry {
  sequence: number;
  family: string;
  registrationId: string;
  kind: HostCapabilityEventKind | null;
  compatibility: HostCapabilityCompatibility | null;
  invalidationMode: HostCapabilityInvalidationMode | null;
  queuedInvalidationCount: number;
  touchedNodes: number;
  reevaluatedNodes: number;
  portableImportOutcome: HostCapabilityPortableImportOutcome | null;
  deniedCallbackIds: ReadonlyArray<string>;
}

export interface HostCapabilityBreadthFamilyReport {
  family: string;
  eventCount: number;
  maxQueuedInvalidationCount: number;
  maxTouchedNodes: number;
  maxReevaluatedNodes: number;
}

export interface HostCapabilityBreadthReport {
  maxQueuedInvalidationCount: number;
  maxTouchedNodes: number;
  maxReevaluatedNodes: number;
  families: ReadonlyArray<HostCapabilityBreadthFamilyReport>;
}

export interface HostCapabilityDiagnosticsReport {
  totals: {
    registrationCount: number;
    disposalCount: number;
    readCount: number;
    pollCount: number;
    noOpPollCount: number;
    manualCommitCount: number;
    noOpManualCommitCount: number;
    invalidationCount: number;
    invalidationBatchFlushCount: number;
    reevaluationCount: number;
    invalidationTouchedNodeCount: number;
    noOpInvalidationSuppressedCount: number;
    staleInvalidationIgnoredCount: number;
    compatibilityDenialCount: number;
    unavailabilityArtifactCount: number;
    broadFanoutDenialCount: number;
    retainedEventCount: number;
  };
  lineage: ReadonlyArray<HostCapabilityLineageEntry>;
  lineageDigest: string;
  breadth: HostCapabilityBreadthReport;
  breadthDigest: string;
  families: ReadonlyArray<HostCapabilityDiagnosticsFamilyReport>;
  digest: string;
}

export interface HostCapabilityTransportFamilyReport {
  family: string;
  callbackIds: ReadonlyArray<string>;
  compatibilities: ReadonlyArray<HostCapabilityCompatibility>;
  exactRestoreOutcomes: ReadonlyArray<string>;
  portableImportOutcomes: ReadonlyArray<HostCapabilityPortableImportOutcome>;
  deniedCallbackIds: ReadonlyArray<string>;
  unavailableCallbackIds: ReadonlyArray<string>;
}

export interface HostCapabilityTransportReport {
  totals: {
    unavailableArtifactCount: number;
    transportEntryCount: number;
    deniedFamilyCount: number;
    unavailableFamilyCount: number;
    snapshotPortableFamilyCount: number;
  };
  families: ReadonlyArray<HostCapabilityTransportFamilyReport>;
  digest: string;
}

export interface UnavailableCallbackArtifact {
  id: string;
  signalKind: string;
  reason: string;
  currentReads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
  hostCapabilityTransports: ReadonlyArray<HostCapabilityTransportArtifact>;
}

export interface RuntimeDefinitionEnvelope {
  policy: RuntimePolicySpec;
  sources: ReadonlyArray<SourceSpec>;
  recipes: ReadonlyArray<RecipeSpec>;
  sourceFamilies: ReadonlyArray<KeyedSourceFamilySpec>;
  recipeFamilies: ReadonlyArray<KeyedRecipeFamilySpec>;
  unavailableCallbacks: ReadonlyArray<UnavailableCallbackArtifact>;
}

export interface RuntimeSnapshotAspectVersionSummary {
  aspect: number;
  version: number;
}

export interface StoredSourceSnapshot {
  id: string;
  value: SignalValue;
  version: number;
  producesAspects?: ReadonlyArray<number>;
  aspectVersions: ReadonlyArray<RuntimeSnapshotAspectVersionSummary>;
}

export interface StoredCallbackRecipeSnapshot {
  tokenSlot: number;
  tokenGeneration: number;
  reads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
}

export interface StoredRecipeSnapshot {
  id: string;
  value: SignalValue;
  version: number;
  producesAspects?: ReadonlyArray<number>;
  aspectVersions: ReadonlyArray<RuntimeSnapshotAspectVersionSummary>;
  initialized: boolean;
  outputIdentity: string | null;
  callback?: StoredCallbackRecipeSnapshot | null;
}

export interface RuntimeStoreSnapshot {
  sources: ReadonlyArray<StoredSourceSnapshot>;
  recipes: ReadonlyArray<StoredRecipeSnapshot>;
}

export interface RuntimeSnapshotArtifactRetentionPolicy {
  explanation_retention: ArtifactRetentionPolicy;
  provenance_retention: ArtifactRetentionPolicy;
}

export interface RuntimeCheckpointImageArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeDiagnosticGraphArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeSnapshotDiagnosticsArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeTelemetryArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeReconstructabilityArtifact extends Readonly<Record<string, unknown>> {}

export interface RuntimeSnapshotMeta {
  schema_version: number;
  snapshot_id: number;
  branch_id: number;
  branch_name: string;
  core_storage_profile: string;
  replay_head: number | null;
  runtime_policy: SignalRuntimePolicySummary;
  artifact_retention: RuntimeSnapshotArtifactRetentionPolicy;
}

export interface RuntimeSnapshotArtifact {
  meta: RuntimeSnapshotMeta;
  checkpoint_image: RuntimeCheckpointImageArtifact;
  diagnostic_graph: RuntimeDiagnosticGraphArtifact;
  diagnostics: RuntimeSnapshotDiagnosticsArtifact;
  graph_telemetry: RuntimeTelemetryArtifact;
  runtime_telemetry?: RuntimeTelemetryArtifact;
  reconstructability?: RuntimeReconstructabilityArtifact;
}

export interface RuntimeSnapshotEnvelope {
  snapshot: RuntimeSnapshotArtifact;
  state: RuntimeStoreSnapshot;
}

export interface RuntimeEnvelope {
  definitions: RuntimeDefinitionEnvelope;
  snapshot: RuntimeSnapshotEnvelope;
}

export interface RuntimeProofReport {
  proofSchemaVersion: string;
  schemaRegistryDigest: string;
  mergeStrategyRegistryDigest: string;
  mergeBaseStrategyRegistryDigest: string;
  aspectMergePolicyRegistryDigest: string;
  conflictIsolationRegistryDigest: string;
  conflictPolicyRegistryDigest: string;
  identityMatcherRegistryDigest: string;
  sourceOnlyPolicyRegistryDigest: string;
  deletionPolicyRegistryDigest: string;
  registryBundleDigest: string;
}

export interface RuntimeBranchHandle {
  id: number;
  name: string;
  parent_branch_id: number | null;
  head_snapshot_id: number | null;
}

export interface ReplayFrameSummary {
  cursor: number;
  kind: string;
  branchId: number;
  snapshotId: number | null;
  node: string | null;
  detail: string | null;
  callback?: CallbackRuntimeNodeSummary | null;
}

export interface ReplaySummary {
  frames: ReadonlyArray<ReplayFrameSummary>;
}

export interface LineageEventSummary {
  sequence: number;
  label: string;
  emittedOnBranchId: number;
  node: string | null;
  subjectArtifactId: number | null;
  parentArtifactId: number | null;
  snapshotId: number | null;
  callback?: CallbackRuntimeNodeSummary | null;
}

export interface LineageSummary {
  events: ReadonlyArray<LineageEventSummary>;
}

export type ReplayMismatchClass =
  | "LegacyMergeArtifactUnsupported"
  | "ProofSchemaVersionMismatch"
  | "MissingRegistryBundleDigest"
  | "RegistryBundleDigestMismatch"
  | "MissingLoweredStrategyBundleDigest"
  | "LoweredStrategyBundleDigestMismatch"
  | "MissingMergePlanDigest"
  | "MergePlanDigestMismatch"
  | "MissingMergeResultDigest"
  | "MergeResultDigestMismatch"
  | "MissingLineageDigest"
  | "LineageDigestMismatch"
  | "BranchStateDigestMismatch";

export interface BranchStateProofReport {
  proofSchemaVersion: string;
  branchId: number;
  branchName: string;
  snapshotId: number | null;
  stateDigest: string;
}

export interface ReplayArtifactProofInput {
  proofSchemaVersion: string;
  registryBundleDigest: string | null;
  loweredStrategyBundleDigest: string | null;
  mergePlanDigest: string | null;
  mergeResultDigest: string | null;
  lineageDigest: string | null;
  branchStateDigest: string;
}

export interface ReplayParityProofReport {
  proofSchemaVersion: string;
  expectedBranchId: number;
  expectedBranchName: string;
  expectedSnapshotId: number | null;
  expectedStateDigest: string;
  replayedBranchId: number;
  replayedBranchName: string;
  replayedSnapshotId: number | null;
  replayedStateDigest: string;
  parity: boolean;
  mismatchClasses: ReadonlyArray<ReplayMismatchClass>;
}

export interface ReplayArtifactProofReport {
  proofSchemaVersion: string;
  expected: ReplayArtifactProofInput;
  replayed: ReplayArtifactProofInput;
  parity: boolean;
  mismatchClasses: ReadonlyArray<ReplayMismatchClass>;
}

export interface MergePolicyPreviewRequest {
  source_branch_id: number;
  target_branch_id: number;
  conflict_policy_name?: string | null;
  conflict_isolation_policy_name?: string | null;
  identity_matcher_name?: string | null;
  deletion_policy_name?: string | null;
}

export interface MergeSemanticsSummary {
  strategy_name: string;
  strategy_digest: string;
  strategy_basis: string;
  merge_base_name: string;
  merge_base_digest: string;
  merge_base_basis: string;
  conflict_policy_name: string;
  conflict_policy_digest: string;
  conflict_policy_basis: string;
  conflict_isolation_name: string;
  conflict_isolation_digest: string;
  conflict_isolation_basis: string;
  identity_matcher_name: string;
  identity_matcher_digest: string;
  identity_matcher_basis: string;
  source_only_policy_name: string;
  source_only_policy_digest: string;
  source_only_policy_basis: string;
  deletion_policy_name: string;
  deletion_policy_digest: string;
  deletion_policy_basis: string;
}

export interface MergeBaseSummary {
  source_branch_id: number;
  target_branch_id: number;
  forked_from_snapshot_id: number | null;
  source_snapshot_id: number | null;
  target_snapshot_id_before: number | null;
}

export interface LoweredMergeBaseSummary {
  resolved_base: MergeBaseSummary;
  selected_merge_base_name: string;
  selected_merge_base_digest: string;
  selected_merge_base_basis: string;
}

export interface ConflictResolutionRecordSummary {
  source_node: string;
  target_node: string;
  required_resolution: ReadonlyArray<string>;
  supported_strategies: ReadonlyArray<string>;
}

export interface ConflictResolutionPlanSummary {
  source_branch_id: number;
  target_branch_id: number;
  divergence: string;
  records: ReadonlyArray<ConflictResolutionRecordSummary>;
}

export interface MergeNodeMapEntrySummary {
  source_node: string;
  target_node: string;
}

export interface MergeDependencyFingerprintSummary {
  dependency_count: number;
  meaningful_input_changes: number;
  output_hash: string;
}

export interface MergeArtifactAuthoritySummary {
  authority_class: string;
  adoptability: string;
}

export interface MergeComparableSummary {
  output_identity: string | null;
  continuity_token: string | null;
  dependency_fingerprint: MergeDependencyFingerprintSummary;
  authority: MergeArtifactAuthoritySummary;
}

export interface NodeMergeInputStateSummary {
  current_artifact_id: number | null;
  comparable: MergeComparableSummary | null;
  authority: MergeArtifactAuthoritySummary | null;
  exists_in_branch: boolean;
}

export interface NodeMergePlanSummary {
  source_node: string;
  shape_kind: string;
  target_node: string | null;
  source_state: NodeMergeInputStateSummary;
  target_state: NodeMergeInputStateSummary;
  decision: string;
  resolved_conflict_kinds: ReadonlyArray<string>;
}

export interface AdoptionTargetIdentitySummary {
  kind: string;
  mapped_target_node: string | null;
}

export interface AdoptedNodeContractSummary {
  merge_strategy_name: string | null;
  conflict_policy_name: string | null;
  identity_matcher_name: string | null;
  source_only_policy_name: string | null;
  deletion_policy_name: string | null;
  conflict_isolation_policy_name: string | null;
  aspect_merge_policy_binding_count: number;
  condition: string;
  comparator: string | null;
  partitioned_output: boolean;
}

export interface AdoptionPlanCoreSummary {
  source_node: string;
  target_identity: AdoptionTargetIdentitySummary;
  authority: MergeArtifactAuthoritySummary;
  entry_contract: AdoptedNodeContractSummary;
  dependency_count: number;
  dependency_snapshot_edge_count: number;
}

export interface AdoptionCarryPolicySummary {
  runtime_artifact: string;
  retained_artifact: string;
  causality: string;
}

export interface MergePlanArtifact {
  source_branch_id: number;
  target_branch_id: number;
  schema_registry_digest: string;
  registry_bundle_digest: string;
  lowered_strategy_bundle_digest: string;
  merge_kind: string;
  selected_semantics: MergeSemanticsSummary;
  source_snapshot_id: number | null;
  target_snapshot_id_before: number | null;
  merge_base: MergeBaseSummary | null;
  lowered_merge_base: LoweredMergeBaseSummary | null;
  resolution_plan: ConflictResolutionPlanSummary | null;
  node_map: ReadonlyArray<MergeNodeMapEntrySummary>;
  node_plan: ReadonlyArray<NodeMergePlanSummary>;
  adoption_core: ReadonlyArray<AdoptionPlanCoreSummary>;
  adoption_policy: ReadonlyArray<AdoptionCarryPolicySummary>;
}

export interface MergeArtifactRecord {
  source_node: string;
  target_node: string | null;
  source_artifact_id: number | null;
  target_artifact_id_before: number | null;
  target_artifact_id_after: number | null;
  action: string;
  basis: string;
  source_comparable: MergeComparableSummary | null;
  target_comparable: MergeComparableSummary | null;
  identity_basis: string | null;
  identity_status: string | null;
  identity_candidate_count: number;
  resolved_conflict_kinds: ReadonlyArray<string>;
}

export interface MergeCountersSummary {
  boundary_witness_kind: string;
  source_slice_breadth: number;
  proof_minimal_overlap_breadth: number;
  conservative_overlap_expansion_breadth: number;
  final_candidate_breadth: number;
  reconciliation_breadth: number;
  candidate_node_count: number;
  examined_node_count: number;
  adopted_count: number;
  introduced_node_count: number;
  replaced_count: number;
  preserved_target_count: number;
  skipped_non_adoptable_count: number;
  equivalent_unchanged_count: number;
  source_only_count: number;
  target_only_count: number;
  dependency_remap_count: number;
  identity_target_candidates_indexed: number;
  identity_source_lookups: number;
  identity_ambiguous_match_count: number;
  identity_rejected_admissibility_count: number;
  conflict_isolation_record_count: number;
  conflict_isolation_expansion_breadth: number;
  subscriber_repair_breadth: number;
  merge_lineage_record_count: number;
  replay_event_count: number;
}

export interface MergeResultArtifact {
  source_branch: number;
  target_branch: number;
  schema_registry_digest: string;
  registry_bundle_digest: string;
  lowered_strategy_bundle_digest: string;
  merge_kind: string;
  selected_semantics: MergeSemanticsSummary;
  merged_snapshot_id: number | null;
  source_snapshot_id: number | null;
  target_snapshot_id_before: number | null;
  target_snapshot_id_after: number | null;
  lowered_merge_base: LoweredMergeBaseSummary | null;
  resolution_plan: ConflictResolutionPlanSummary | null;
  records: ReadonlyArray<MergeArtifactRecord>;
  counters: MergeCountersSummary;
}

export interface MergePlanProofReport {
  proofSchemaVersion: string;
  registryBundleDigest: string;
  planDigest: string;
  semanticsDigest: string;
  loweredStrategyBundleDigest: string;
  selectedStrategyDigest: string;
  selectedMergeBaseDigest: string;
  selectedConflictPolicyDigest: string;
  selectedConflictIsolationDigest: string;
  selectedIdentityMatcherDigest: string;
  selectedSourceOnlyPolicyDigest: string;
  selectedDeletionPolicyDigest: string;
}

export interface MergeResultProofReport {
  proofSchemaVersion: string;
  registryBundleDigest: string;
  resultDigest: string;
  semanticsDigest: string;
  loweredStrategyBundleDigest: string;
  lineageDigest: string;
  selectedStrategyDigest: string;
  selectedMergeBaseDigest: string;
  selectedConflictPolicyDigest: string;
  selectedConflictIsolationDigest: string;
  selectedIdentityMatcherDigest: string;
  selectedSourceOnlyPolicyDigest: string;
  selectedDeletionPolicyDigest: string;
}

export interface MergePlanProofEnvelope {
  plan: MergePlanArtifact;
  proof: MergePlanProofReport;
}

export interface MergeResultProofEnvelope {
  result: MergeResultArtifact;
  proof: MergeResultProofReport;
}

export interface EventSubscriberOutcome {
  subscriber_name: string;
  outcome: "Committed" | "Failed" | string;
  requires_data_ids: ReadonlyArray<string>;
  provides_data_ids: ReadonlyArray<string>;
  staged_data_ids: ReadonlyArray<string>;
}

export interface EventEpochSummary {
  ordinal: number;
  barrier: "PerMutation" | "PerOperation" | "PerCommit" | "OnDemandRead" | string;
  emitted_event_count: number;
  subscriber_count: number;
  committed_subscriber_count: number;
  failed_subscriber_position: number | null;
  subscriber_outcomes: ReadonlyArray<EventSubscriberOutcome>;
  outcome: "Committed" | "RolledBack" | "Failed" | string;
  failure_subscriber: string | null;
  message: string | null;
}

export interface ObservationPolicySummary {
  trigger: "Touched" | "Recomputed" | "MeaningfulChange" | string;
  delivery_mode: "PerCommittedTransaction" | string;
}

export interface ObservedNodeSetSummary {
  nodes: ReadonlyArray<NodeIdSummary>;
}

export interface CommittedObservationEventSummary {
  observer_id: number;
  handle_id: number;
  policy: ObservationPolicySummary;
  observed_nodes: ObservedNodeSetSummary;
  matched_nodes: ObservedNodeSetSummary;
  touched: boolean;
  recomputed: boolean;
  meaningful_change: boolean;
  trigger_matched: boolean;
  outcome: "Delivered" | "RollbackSuppressed" | string;
}

export interface ObservationBoundarySummary {
  classified_event_count: number;
  trigger_matched_event_count: number;
  delivered_event_count: number;
  rollback_suppressed_event_count: number;
  boundary_events: ReadonlyArray<CommittedObservationEventSummary>;
}

export interface ChangeInputSummary {
  changed_nodes: ReadonlyArray<NodeIdSummary>;
  changed_aspects: ReadonlyArray<number>;
  changed_region_count: number;
  causality_kind: string | null;
}

export interface InvalidationSummary {
  invalidated_direct_subscribers: number;
  maybe_stale_direct_subscribers: number;
  partition_scoped_checks: number;
  narrowed_frontier_width: number;
  transitive_frontier_width: number;
  frontier_seed_count: number;
  frontier_group_count: number;
  frontier_direct_wave_count: number;
  frontier_transitive_wave_count: number;
  frontier_partition_match_count: number;
  frontier_detail_match_count: number;
  frontier_cycle_check_candidate_count: number;
  frontier_cycle_check_visited_count: number;
  frontier_trace_retained_count: number;
}

export interface EvaluationPlanSummary {
  profile: DiagnosticsTier;
  requested_target_count: number;
  stage_count: number;
  task_count: number;
  max_stage_width: number;
  contract_pruned_count: number;
  stage_widths: ReadonlyArray<number>;
  direct_request_count: number;
  transitive_task_count: number;
  task_reason_counts: DiagnosticsCounterMap;
}

export interface PlanningSummary {
  plan: EvaluationPlanSummary;
}

export interface TemporalExecutionSummary {
  ready_count: number;
  deferred_count: number;
  runtime_clock_authority_count: number;
  resolver_fallback_count: number;
  runtime_scheduled_wake_count: number;
}

export interface ExecutionReportSummary {
  profile: DiagnosticsTier;
  stage_count: number;
  task_count: number;
  tasks_executed: number;
  tasks_pruned: number;
  tasks_validated_clean: number;
  tasks_deferred_by_condition: number;
  tasks_reverted_clean_by_condition: number;
  tasks_satisfied_by_memoization: number;
  tasks_with_suppressed_propagation: number;
  prepared_evaluations_produced: number;
  prepared_evaluations_applied: number;
  dependency_capture_updates: number;
  semantic_segment_count: number;
  temporal_summary: TemporalExecutionSummary;
  task_outcome_counts: DiagnosticsCounterMap;
  stage_outcome_counts: DiagnosticsCounterMap;
}

export interface PrecomputeSummary {
  executor: DiagnosticsVariant | null;
  stage_count: number;
  task_count: number;
  prepared_evaluations_produced: number;
  tasks_deferred_by_condition: number;
  tasks_satisfied_by_memoization: number;
}

export interface ApplySummary {
  report: ExecutionReportSummary;
  prepared_evaluations_applied: number;
  dependency_capture_updates: number;
  tasks_validated_clean: number;
  tasks_pruned: number;
  tasks_with_suppressed_propagation: number;
}

export interface RollbackSummary {
  rolled_back: boolean;
  staged_node_patch_count: number;
  max_touched_nodes_in_txn: number;
  reason: string | null;
}

export interface FlowCauseSample {
  node: NodeIdSummary;
  cause_kinds: ReadonlyArray<string>;
  scope_kinds: ReadonlyArray<string>;
  scope_notes: ReadonlyArray<string>;
  suspect_classes: ReadonlyArray<string>;
  rewired: boolean;
  conservative_recompute: boolean;
}

export interface ReuseBasisSummary {
  strategy?: string | null;
  source: DiagnosticsVariant;
  crossing: DiagnosticsVariant;
  dependency_snapshot_basis?: number | string | null;
  topology_regime_basis?: number | null;
  structural_dependency_basis?: number | string | null;
  artifact_family_basis?: string | null;
  partition_region_basis_count: number;
}

export interface ExplanationSummary {
  profile: DiagnosticsTier;
  node: NodeIdSummary;
  materialization_mode: DiagnosticsVariant;
  state: DiagnosticsVariant;
  dirty_aspect_count: number;
  upstream_count: number;
  changed_upstream_count: number;
  skipped_upstream_count: number;
  condition_deferred_count: number;
  clean_upstream_count: number;
  missing_snapshot_count: number;
  dependency_removed_count: number;
  conservative_cause_count: number;
  direct_scope_count: number;
  translated_scope_count: number;
  discarded_scope_count: number;
  insufficient_scope_count: number;
  rewired_dependency_count: number;
  direct_cause_kinds: ReadonlyArray<DiagnosticsVariant>;
  scope_provenance_kinds: ReadonlyArray<string>;
  cause_note_samples: ReadonlyArray<string>;
  triage_classes: ReadonlyArray<string>;
  propagation_suppressed: boolean;
  contract_reads_mask: number | string;
  contract_produces_mask: number | string;
  contract_partition_scope_count: number;
  required_context: DiagnosticsVariant;
  execution_record_id: number | null;
  semantic_segment_id: number | null;
  output_change: string | null;
  memoized_origin: string | null;
  reuse_basis: ReuseBasisSummary | null;
  reuse_origin: string | null;
  reuse_certification_proof_count: number;
  changed_region_count: number;
  causality_kind: string | null;
}

export interface FlowSummary {
  profile: DiagnosticsTier;
  change: ChangeInputSummary;
  invalidation: InvalidationSummary;
  planning: PlanningSummary;
  precompute: PrecomputeSummary;
  apply: ApplySummary;
  cause_samples: ReadonlyArray<FlowCauseSample>;
  event_epochs: ReadonlyArray<EventEpochSummary>;
  observation: ObservationBoundarySummary | null;
  rollback: RollbackSummary | null;
  explanation: ExplanationSummary | null;
}

export interface FlowSurfaceSummary {
  flow: FlowSummary;
  callbackNodes: ReadonlyArray<CallbackRuntimeNodeSummary>;
}

export interface ObservationSurfaceSummary {
  observation: ObservationBoundarySummary;
  callbackNodes: ReadonlyArray<CallbackRuntimeNodeSummary>;
}

export interface ExecutionHistoryNodeSummary {
  node: NodeIdSummary;
  execution_record_id: number | null;
  semantic_segment_id: number | null;
  output_change: string | null;
  memoized_origin: string | null;
  reuse_basis: ReuseBasisSummary | null;
  reuse_origin: string | null;
  persistent_correspondence_kind: string | null;
  composition_region_count: number;
  reuse_certification_proof_count: number;
  changed_partition_count: number;
  causality_kind: string | null;
}

export interface ExecutionHistorySummary {
  profile: DiagnosticsTier;
  traced_node_count: number;
  execution_record_count: number;
  latest_execution_record_id: number | null;
  reuse_origin_counts: DiagnosticsCounterMap;
  nodes: ReadonlyArray<ExecutionHistoryNodeSummary>;
}

export interface ExecutionHistorySurfaceSummary {
  history: ExecutionHistorySummary;
  callbackNodes: ReadonlyArray<CallbackRuntimeNodeSummary>;
}

export interface FailureSummary {
  profile: DiagnosticsTier;
  phase: string;
  stage_index: number | null;
  node: NodeIdSummary | null;
  executor: DiagnosticsVariant | null;
  execution_record_id: number | null;
  has_plan_summary: boolean;
  rolled_back: boolean;
  staged_node_patch_count: number | null;
  max_touched_nodes_in_txn: number | null;
  event_epochs: ReadonlyArray<EventEpochSummary>;
  message: string;
}

export interface RollbackDiagnostic {
  rolled_back: boolean;
  staged_node_patch_count: number;
  max_touched_nodes_in_txn: number;
  reason: string | null;
  event_epochs: ReadonlyArray<EventEpochSummary>;
}

export interface TouchedScopeSummary {
  seed_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  inclusion_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  transitive_reached_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  direct_dirty_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  maybe_stale_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
  touched_nodes: ReadonlyArray<NodeIdSummary>;
  touched_sources: ReadonlyArray<NodeIdSummary>;
}

export interface PartitionSubscriptionSummary {
  partition: string;
  detail?: string | null;
  match_mode: "WholePartition" | "PartitionAndDetail" | string;
}

export interface FrontierExecutionCounters {
  frontier_seed_count: number;
  frontier_group_count: number;
  frontier_direct_wave_count: number;
  frontier_transitive_wave_count: number;
  frontier_partition_scoped_check_count: number;
  frontier_direct_dirty_count: number;
  frontier_maybe_stale_count: number;
  frontier_partition_match_count: number;
  frontier_detail_match_count: number;
  frontier_cycle_check_candidate_count: number;
  frontier_cycle_check_visited_count: number;
  frontier_trace_retained_count: number;
}

export interface FrontierWaveEntrySummary {
  node: NodeIdSummary;
  classification: string;
  inclusion_basis: string;
  narrowed_scopes: ReadonlyArray<PartitionSubscriptionSummary>;
}

export interface FrontierWaveSummary {
  wave_index: number;
  aspect: number;
  entries: ReadonlyArray<FrontierWaveEntrySummary>;
}

export interface InvalidationPlanningEstimate {
  seed_count: number;
  group_count: number;
  direct_wave_count: number;
  transitive_wave_count: number;
  direct_dirty_count: number;
  maybe_stale_count: number;
  partition_scoped_checks: number;
  partition_match_count: number;
  detail_match_count: number;
  cycle_check_candidate_count: number;
}

export interface InvalidationTraceRecord {
  node: NodeIdSummary;
  aspect: number;
  wave_index: number;
  classification: string;
  inclusion_basis: string;
}

export interface GraphSummary {
  profile: DiagnosticsTier;
  active_node_count: number;
  arena_capacity: number;
  tombstone_count: number;
  clean_node_count: number;
  maybe_stale_node_count: number;
  dirty_node_count: number;
  dependency_edge_count: number;
  subscriber_edge_count: number;
  nodes_with_partition_scopes: number;
  nodes_with_trace_summary: number;
  nodes_with_execution_record: number;
  nodes_with_causality: number;
  partition_interner_size: number;
  sample_dirty_nodes: ReadonlyArray<NodeIdSummary>;
  sample_nodes_with_execution_record: ReadonlyArray<NodeIdSummary>;
  metrics: Readonly<Record<string, unknown>>;
}

export interface WebPerformanceSummary {
  activeHandleCount: number;
  activeCallbackCount: number;
  activeComputeCallbackCount: number;
  activeComputeCollectorCount: number;
  matchedWatcherBreadth: number;
  deliveredObservationCount: number;
  rollbackSuppressedDeliveryCount: number;
  serialExecutorUsageCount: number;
  executionReport: WebExecutionReportSummary | null;
  outputSerializationCount: number;
  outputSerializationBreadth: number;
  jsCallbackInvocationCount: number;
  jsCallbackFailureCount: number;
  observationCallbackRegistrationCount: number;
  observationCallbackDisposalCount: number;
  observationCallbackGenerationMismatchDenialCount: number;
  observationCallbackAllocationCount: number;
  observationCallbackReuseCount: number;
  computeCallbackRegistrationCount: number;
  computeCallbackDisposalCount: number;
  computeCallbackInvocationCount: number;
  computeCallbackFailureCount: number;
  computeCallbackGenerationMismatchDenialCount: number;
  computeCallbackSelfReadDenialCount: number;
  computeCallbackDynamicCycleDenialCount: number;
  computeCallbackPromiseReturnDenialCount: number;
  computeCallbackInvalidReturnDenialCount: number;
  computeCallbackCollectorInstallationCount: number;
  computeCallbackCaptureCount: number;
  computeCallbackCapturedReadCount: number;
  computeCallbackReturnSerializationBreadth: number;
  computeCallbackAllocationCount: number;
  computeCallbackReuseCount: number;
  computeCallbackDependencyPatchCount: number;
  computeCallbackDependencyPatchAddedCount: number;
  computeCallbackDependencyPatchRemovedCount: number;
  computeCallbackDependencyPatchRetainedCount: number;
  computeCallbackRuntimeReadBreadth: number;
  computeCallbackConstantNoSignalReadClassificationCount: number;
  computeCallbackSignalTrackedClassificationCount: number;
  computeCallbackMissingUnavailabilityCount: number;
  compatibilityReadCount: number;
  compatibilityReadBreadth: number;
  hostCapabilityRegistrationCount?: number;
  hostCapabilityDisposalCount?: number;
  hostCapabilityReadCount?: number;
  hostCapabilityPollCount?: number;
  hostCapabilityNoOpPollCount?: number;
  hostCapabilityManualCommitCount?: number;
  hostCapabilityNoOpManualCommitCount?: number;
  hostCapabilityInvalidationCount?: number;
  hostCapabilityInvalidationBatchFlushCount?: number;
  hostCapabilityReevaluationCount?: number;
  hostCapabilityInvalidationTouchedNodeCount?: number;
  hostCapabilityNoOpInvalidationSuppressedCount?: number;
  hostCapabilityStaleInvalidationIgnoredCount?: number;
  hostCapabilityCompatibilityDenialCount?: number;
  hostCapabilityUnavailabilityArtifactCount?: number;
  hostCapabilityBroadFanoutDenialCount?: number;
}

export interface WebExecutionReportSummary {
  resolvedPosture: string;
  chargedWork: number;
  chargedSpan: number;
  activeWorkersHighWatermark: number;
  peakChargedMemoryBytes: number;
  peakQueueWidth: number;
  discardedInFlightWork: number;
  fallback: string | null;
}
