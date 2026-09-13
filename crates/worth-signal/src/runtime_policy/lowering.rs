use super::admission::AdmittedSignalRuntimePolicy;
use super::definition::SignalRuntimePolicy;
use super::resolved::ResolvedSignalRuntimePolicy;
use crate::data::node::{ArtifactPolicyClass, AuthorityPolicy, PathClass};
use crate::data::performance::ResolvedPerformancePolicy;
use crate::diagnostics::policy::ArtifactRetentionPolicy;
use crate::diagnostics::profile::DiagnosticsTier;

impl SignalRuntimePolicy {
    pub fn foundational_profile(self) -> worth_foundational::FoundationalProfileSet {
        worth_foundational::FoundationalProfileSet::new(
            worth_foundational::FoundationalProfileSetInput {
                diagnostic_richness: foundational_diagnostic_richness(self.tier),
                support_posture: worth_foundational::SupportPostureProfile::SupportReady,
                compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
                admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
                retention_delivery: foundational_retention_delivery(
                    self.retention_budget.explanation_retention,
                ),
                certification_posture: worth_foundational::CertificationPostureProfile::Uncertified,
                execution_objective: self.execution_objective,
                observation_activation: self.observation_activation,
            },
        )
        .expect("runtime policy lowering produces a coherent foundational profile")
    }

    pub(crate) fn default_path_class(self) -> PathClass {
        match self.tier {
            DiagnosticsTier::Operational => PathClass::Operational,
            DiagnosticsTier::Development | DiagnosticsTier::Forensic => PathClass::Rich,
        }
    }

    pub(crate) fn default_artifact_policy_class(self) -> ArtifactPolicyClass {
        match (
            self.tier,
            self.retention_budget.explanation_retention,
            self.retention_budget.provenance_retention,
        ) {
            (
                DiagnosticsTier::Development,
                ArtifactRetentionPolicy::Retain,
                ArtifactRetentionPolicy::Retain,
            ) => ArtifactPolicyClass::DevelopmentRetained,
            (DiagnosticsTier::Forensic, _, _) => ArtifactPolicyClass::ForensicReconstructable,
            _ => ArtifactPolicyClass::OperationalMinimal,
        }
    }

    pub(crate) const fn default_authority_policy(self) -> AuthorityPolicy {
        AuthorityPolicy::SpeculativeThenReconcile
    }

    pub(crate) fn resolve_performance_policy(self) -> ResolvedPerformancePolicy {
        ResolvedPerformancePolicy {
            path_class: self.default_path_class(),
            artifact_policy: self.default_artifact_policy_class(),
            execution_strategy: self.default_execution_strategy(),
            maintenance_strategy: self.default_maintenance_strategy(),
            authority_policy: self.default_authority_policy(),
        }
    }
}

pub(super) fn resolve_signal_runtime_policy(
    admitted: AdmittedSignalRuntimePolicy,
) -> ResolvedSignalRuntimePolicy {
    let request = admitted.request().policy();
    let parallel_min_tasks = request
        .parallel_admission
        .min_parallel_tasks_for_objective(request.execution_objective);
    ResolvedSignalRuntimePolicy {
        maximum_waiter_resolution_visits: request.maximum_waiter_resolution_visits,
        maximum_upstream_dependency_visits: request.maximum_upstream_dependency_visits,
        conditional_evaluation_budget: request.conditional_evaluation_budget,
        conditional_temporal_budget: request.conditional_temporal_budget,
        execution_objective: request.execution_objective,
        observation_activation: request.observation_activation,
        observation_capture_plan: super::observation::SignalObservationCapturePlan::from_activation(
            request.observation_activation,
        ),
        performance: request.resolve_performance_policy(),
        retention_budget: request.retention_budget,
        reconstruction_budget: request.reconstruction_budget,
        snapshot_restore_lineage_mode: request.snapshot_restore_lineage_mode,
        frontier_tracing_policy: request.frontier_tracing_policy,
        frontier_propagation_policy: request.frontier_propagation_policy,
        frontier_cycle_policy: request.frontier_cycle_policy,
        parallel_min_tasks,
        full_parallel_min_tasks: request.parallel_admission.full_parallel_min_tasks,
        tier: request.tier,
    }
}

const fn foundational_diagnostic_richness(
    tier: DiagnosticsTier,
) -> worth_foundational::DiagnosticRichnessProfile {
    match tier {
        DiagnosticsTier::Operational => {
            worth_foundational::DiagnosticRichnessProfile::OperationalMinimal
        }
        DiagnosticsTier::Development => worth_foundational::DiagnosticRichnessProfile::Standard,
        DiagnosticsTier::Forensic => worth_foundational::DiagnosticRichnessProfile::Forensic,
    }
}

const fn foundational_retention_delivery(
    retention: ArtifactRetentionPolicy,
) -> worth_foundational::RetentionDeliveryProfile {
    match retention {
        ArtifactRetentionPolicy::Retain => worth_foundational::RetentionDeliveryProfile::Retained,
        ArtifactRetentionPolicy::Reconstruct | ArtifactRetentionPolicy::Omit => {
            worth_foundational::RetentionDeliveryProfile::Ephemeral
        }
    }
}
