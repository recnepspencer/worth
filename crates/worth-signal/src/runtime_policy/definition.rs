use serde::{Deserialize, Serialize};

use crate::diagnostics::policy::{
    FrontierCyclePolicy, FrontierPropagationPolicy, FrontierTracingPolicy, ReconstructionBudget,
    RetentionBudget, SnapshotRestoreLineageMode,
};
use crate::diagnostics::profile::DiagnosticsTier;
use worth_foundational::{ExecutionObjectiveProfile, ObservationActivationProfile};

/// The caller-authored policy request that Signal compiles into one installed
/// runtime authority.  The definition lives with the compiler, while the
/// diagnostics module only consumes its retention projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalRuntimePolicy {
    /// Required on decoded requests; presets choose an explicit finite limit.
    pub maximum_waiter_resolution_visits: usize,
    pub maximum_upstream_dependency_visits: usize,
    pub conditional_evaluation_budget: super::SignalConditionalEvaluationBudget,
    pub conditional_temporal_budget: super::SignalConditionalTemporalBudget,
    pub tier: DiagnosticsTier,
    #[serde(default = "default_execution_objective")]
    pub execution_objective: ExecutionObjectiveProfile,
    #[serde(default = "default_observation_activation")]
    pub observation_activation: ObservationActivationProfile,
    pub retention_budget: RetentionBudget,
    pub reconstruction_budget: ReconstructionBudget,
    pub snapshot_restore_lineage_mode: SnapshotRestoreLineageMode,
    #[serde(default)]
    pub frontier_tracing_policy: FrontierTracingPolicy,
    #[serde(default)]
    pub frontier_propagation_policy: FrontierPropagationPolicy,
    #[serde(default)]
    pub frontier_cycle_policy: FrontierCyclePolicy,
    pub parallel_admission: super::parallel::ParallelAdmissionPolicy,
}

fn default_execution_objective() -> ExecutionObjectiveProfile {
    ExecutionObjectiveProfile::Balanced
}

fn default_observation_activation() -> ObservationActivationProfile {
    ObservationActivationProfile::Continuous
}
