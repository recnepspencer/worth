use super::definition::SignalRuntimePolicy;
use super::parallel::ParallelAdmissionPolicy;
use crate::diagnostics::policy::{
    ArtifactRetentionPolicy, FrontierCyclePolicy, FrontierPropagationPolicy, FrontierTracingPolicy,
    ReconstructionBudget, ReplayDetailPolicy, RetentionBudget, SemanticRetentionPolicy,
    SnapshotRestoreLineageMode,
};
use crate::diagnostics::profile::DiagnosticsTier;
use worth_foundational::{ExecutionObjectiveProfile, ObservationActivationProfile};

impl Default for SignalRuntimePolicy {
    fn default() -> Self {
        Self::development()
    }
}

impl SignalRuntimePolicy {
    pub fn for_tier(tier: DiagnosticsTier) -> Self {
        let snapshot_restore_lineage_mode = match tier {
            DiagnosticsTier::Forensic => SnapshotRestoreLineageMode::PerNode,
            DiagnosticsTier::Operational | DiagnosticsTier::Development => {
                SnapshotRestoreLineageMode::CompactGlobal
            }
        };
        let frontier_tracing_policy = match tier {
            DiagnosticsTier::Operational => FrontierTracingPolicy::SummaryOnly,
            DiagnosticsTier::Development => FrontierTracingPolicy::RetainWaveRecords,
            DiagnosticsTier::Forensic => FrontierTracingPolicy::FullForensic,
        };
        Self {
            conditional_evaluation_budget: super::SignalConditionalEvaluationBudget::PRESET,
            conditional_temporal_budget: super::SignalConditionalTemporalBudget::PRESET,
            maximum_upstream_dependency_visits:
                super::upstream_traversal::DEFAULT_MAXIMUM_UPSTREAM_DEPENDENCY_VISITS,
            maximum_waiter_resolution_visits:
                super::waiter_resolution::DEFAULT_MAXIMUM_WAITER_RESOLUTION_VISITS,
            tier,
            execution_objective: match tier {
                DiagnosticsTier::Operational => ExecutionObjectiveProfile::Throughput,
                DiagnosticsTier::Development => ExecutionObjectiveProfile::Balanced,
                DiagnosticsTier::Forensic => ExecutionObjectiveProfile::LatencyBounded,
            },
            observation_activation: match tier {
                DiagnosticsTier::Operational => ObservationActivationProfile::OnDemand,
                DiagnosticsTier::Development | DiagnosticsTier::Forensic => {
                    ObservationActivationProfile::Continuous
                }
            },
            retention_budget: RetentionBudget::for_tier(tier),
            reconstruction_budget: ReconstructionBudget::for_tier(tier),
            snapshot_restore_lineage_mode,
            frontier_tracing_policy,
            frontier_propagation_policy: FrontierPropagationPolicy::CanonicalFrontier,
            frontier_cycle_policy: FrontierCyclePolicy::ReachableCycleCheck,
            parallel_admission: ParallelAdmissionPolicy::default(),
        }
    }

    /// Public production constructor: Throughput objective + OnDemand
    /// observation + Operational richness. Do not add a second constructor
    /// that installs this same policy under a performance-implying name.
    pub fn operational() -> Self {
        Self::for_tier(DiagnosticsTier::Operational)
    }
    pub fn development() -> Self {
        Self::for_tier(DiagnosticsTier::Development)
    }
    pub fn forensic() -> Self {
        Self::for_tier(DiagnosticsTier::Forensic)
    }

    pub fn web_development() -> Self {
        Self::operational().with_parallel_admission(ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 4,
            balanced_min_parallel_tasks: 8,
            latency_bounded_min_parallel_tasks: 12,
            full_parallel_min_tasks: 16,
        })
    }

    pub fn kernel() -> Self {
        Self::forensic().with_parallel_admission(ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 4,
            balanced_min_parallel_tasks: 8,
            latency_bounded_min_parallel_tasks: 16,
            full_parallel_min_tasks: 16,
        })
    }

    pub fn fintech() -> Self {
        let mut policy = Self::development();
        policy.retention_budget.replay_detail = ReplayDetailPolicy::Forensic;
        policy.with_parallel_admission(ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 4,
            balanced_min_parallel_tasks: 8,
            latency_bounded_min_parallel_tasks: 12,
            full_parallel_min_tasks: 12,
        })
    }

    pub fn game_engine() -> Self {
        Self::operational().with_parallel_admission(ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 2,
            balanced_min_parallel_tasks: 4,
            latency_bounded_min_parallel_tasks: 8,
            full_parallel_min_tasks: 8,
        })
    }

    pub fn with_explanation_retention(mut self, retention: ArtifactRetentionPolicy) -> Self {
        self.retention_budget.explanation_retention = retention;
        self
    }
    pub fn with_provenance_retention(mut self, retention: ArtifactRetentionPolicy) -> Self {
        self.retention_budget.provenance_retention = retention;
        self
    }
    pub fn with_replay_detail(mut self, replay_detail: ReplayDetailPolicy) -> Self {
        self.retention_budget.replay_detail = replay_detail;
        self
    }
    pub fn with_semantic_retention(mut self, semantic_retention: SemanticRetentionPolicy) -> Self {
        self.retention_budget.semantic_detail = semantic_retention;
        self
    }
    pub fn with_snapshot_restore_lineage_mode(mut self, mode: SnapshotRestoreLineageMode) -> Self {
        self.snapshot_restore_lineage_mode = mode;
        self
    }
    pub fn with_parallel_admission(mut self, parallel_admission: ParallelAdmissionPolicy) -> Self {
        self.parallel_admission = parallel_admission;
        self
    }
    pub fn with_history_limit(mut self, history_limit: usize) -> Self {
        self.retention_budget.history_limit =
            crate::diagnostics::policy::HistoryLimit::new(history_limit);
        self
    }
    pub fn with_detail_limit(mut self, detail_limit: usize) -> Self {
        self.retention_budget.detail_limit =
            crate::diagnostics::policy::DetailLimit::new(detail_limit);
        self
    }
    pub fn with_history_details(mut self, retain_history_details: bool) -> Self {
        self.retention_budget.retain_history_details = retain_history_details;
        self
    }
}
