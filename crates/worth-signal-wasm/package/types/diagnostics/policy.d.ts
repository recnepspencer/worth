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
