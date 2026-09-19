use crate::data::performance::ResolvedPerformancePolicy;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::definition::SignalRuntimePolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSignalRuntimePolicy {
    pub(super) maximum_waiter_resolution_visits: usize,
    pub(super) maximum_upstream_dependency_visits: usize,
    pub(super) conditional_evaluation_budget: super::SignalConditionalEvaluationBudget,
    pub(super) conditional_temporal_budget: super::SignalConditionalTemporalBudget,
    pub(super) execution_objective: worth_foundational::ExecutionObjectiveProfile,
    pub(super) observation_activation: worth_foundational::ObservationActivationProfile,
    #[serde(default)]
    pub(super) observation_capture_plan: super::observation::SignalObservationCapturePlan,
    pub(super) performance: ResolvedPerformancePolicy,
    pub(super) retention_budget: crate::diagnostics::policy::RetentionBudget,
    pub(super) reconstruction_budget: crate::diagnostics::policy::ReconstructionBudget,
    pub(super) snapshot_restore_lineage_mode:
        crate::diagnostics::policy::SnapshotRestoreLineageMode,
    pub(super) frontier_tracing_policy: crate::diagnostics::policy::FrontierTracingPolicy,
    pub(super) frontier_propagation_policy: crate::diagnostics::policy::FrontierPropagationPolicy,
    pub(super) frontier_cycle_policy: crate::diagnostics::policy::FrontierCyclePolicy,
    pub(super) parallel_min_tasks: usize,
    pub(super) full_parallel_min_tasks: usize,
    pub(super) tier: crate::diagnostics::profile::DiagnosticsTier,
}

impl ResolvedSignalRuntimePolicy {
    pub const fn execution_objective(&self) -> worth_foundational::ExecutionObjectiveProfile {
        self.execution_objective
    }

    pub const fn observation_activation(&self) -> worth_foundational::ObservationActivationProfile {
        self.observation_activation
    }

    pub const fn observation_capture_plan(
        &self,
    ) -> super::observation::SignalObservationCapturePlan {
        self.observation_capture_plan
    }

    pub const fn performance(&self) -> ResolvedPerformancePolicy {
        self.performance
    }

    pub const fn retention_budget(&self) -> crate::diagnostics::policy::RetentionBudget {
        self.retention_budget
    }

    pub const fn reconstruction_budget(&self) -> crate::diagnostics::policy::ReconstructionBudget {
        self.reconstruction_budget
    }

    pub const fn parallel_min_tasks(&self) -> usize {
        self.parallel_min_tasks
    }

    pub const fn full_parallel_min_tasks(&self) -> usize {
        self.full_parallel_min_tasks
    }

    pub const fn tier(&self) -> crate::diagnostics::profile::DiagnosticsTier {
        self.tier
    }

    pub const fn frontier_tracing_policy(
        &self,
    ) -> crate::diagnostics::policy::FrontierTracingPolicy {
        self.frontier_tracing_policy
    }

    pub const fn snapshot_restore_lineage_mode(
        &self,
    ) -> crate::diagnostics::policy::SnapshotRestoreLineageMode {
        self.snapshot_restore_lineage_mode
    }

    pub const fn frontier_propagation_policy(
        &self,
    ) -> crate::diagnostics::policy::FrontierPropagationPolicy {
        self.frontier_propagation_policy
    }

    pub const fn frontier_cycle_policy(&self) -> crate::diagnostics::policy::FrontierCyclePolicy {
        self.frontier_cycle_policy
    }

    pub const fn execution_strategy(&self) -> crate::data::performance::ResolvedExecutionStrategy {
        self.performance.execution_strategy
    }

    pub const fn maintenance_strategy(
        &self,
    ) -> crate::data::performance::ResolvedMaintenanceStrategy {
        self.performance.maintenance_strategy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InstalledSignalRuntimePolicy {
    resolved: ResolvedSignalRuntimePolicy,
    requested_policy: SignalRuntimePolicy,
}

impl<'de> Deserialize<'de> for InstalledSignalRuntimePolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            requested_policy: SignalRuntimePolicy,
            resolved: ResolvedSignalRuntimePolicy,
        }

        let wire = Wire::deserialize(deserializer)?;
        let compiled = crate::runtime_policy::compile_signal_runtime_policy(
            crate::runtime_policy::SignalRuntimePolicyRequest::new(wire.requested_policy),
        )
        .map_err(|error| D::Error::custom(format!("runtime policy admission failed: {error:?}")))?;
        if compiled.requested_policy() != wire.requested_policy {
            return Err(D::Error::custom(
                "installed runtime policy request does not match its compiled authority",
            ));
        }
        if compiled.resolved() != wire.resolved {
            return Err(D::Error::custom(
                "installed runtime policy authority does not match compiler output",
            ));
        }
        Ok(compiled)
    }
}

impl Default for InstalledSignalRuntimePolicy {
    fn default() -> Self {
        crate::runtime_policy::compile_signal_runtime_policy(
            crate::runtime_policy::SignalRuntimePolicyRequest::new(SignalRuntimePolicy::default()),
        )
        .expect("default runtime policy compiles")
    }
}

impl InstalledSignalRuntimePolicy {
    pub(crate) fn new(
        resolved: ResolvedSignalRuntimePolicy,
        requested_policy: SignalRuntimePolicy,
    ) -> Self {
        Self {
            resolved,
            requested_policy,
        }
    }

    pub const fn resolved(&self) -> ResolvedSignalRuntimePolicy {
        self.resolved
    }

    pub const fn requested_policy(&self) -> SignalRuntimePolicy {
        self.requested_policy
    }

    pub const fn performance(&self) -> ResolvedPerformancePolicy {
        self.resolved.performance()
    }

    pub const fn execution_strategy(&self) -> crate::data::performance::ResolvedExecutionStrategy {
        self.resolved.execution_strategy()
    }

    pub const fn maintenance_strategy(
        &self,
    ) -> crate::data::performance::ResolvedMaintenanceStrategy {
        self.resolved.maintenance_strategy()
    }

    pub const fn retention_budget(&self) -> crate::diagnostics::policy::RetentionBudget {
        self.resolved.retention_budget()
    }

    pub const fn reconstruction_budget(&self) -> crate::diagnostics::policy::ReconstructionBudget {
        self.resolved.reconstruction_budget()
    }

    pub const fn observation_activation(&self) -> worth_foundational::ObservationActivationProfile {
        self.resolved.observation_activation()
    }

    pub const fn observation_capture_plan(
        &self,
    ) -> super::observation::SignalObservationCapturePlan {
        self.resolved.observation_capture_plan()
    }

    pub const fn execution_objective(&self) -> worth_foundational::ExecutionObjectiveProfile {
        self.resolved.execution_objective()
    }

    pub const fn parallel_admission(&self) -> super::parallel::ParallelAdmissionPolicy {
        self.requested_policy.parallel_admission
    }

    pub const fn parallel_min_tasks(&self) -> usize {
        self.resolved.parallel_min_tasks()
    }

    pub const fn full_parallel_min_tasks(&self) -> usize {
        self.resolved.full_parallel_min_tasks()
    }

    pub const fn frontier_tracing_policy(
        &self,
    ) -> crate::diagnostics::policy::FrontierTracingPolicy {
        self.resolved.frontier_tracing_policy()
    }

    pub const fn snapshot_restore_lineage_mode(
        &self,
    ) -> crate::diagnostics::policy::SnapshotRestoreLineageMode {
        self.resolved.snapshot_restore_lineage_mode()
    }

    pub const fn frontier_propagation_policy(
        &self,
    ) -> crate::diagnostics::policy::FrontierPropagationPolicy {
        self.resolved.frontier_propagation_policy()
    }

    pub const fn frontier_cycle_policy(&self) -> crate::diagnostics::policy::FrontierCyclePolicy {
        self.resolved.frontier_cycle_policy()
    }

    pub const fn tier(&self) -> crate::diagnostics::profile::DiagnosticsTier {
        self.resolved.tier
    }
}
