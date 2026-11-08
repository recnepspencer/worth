//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalRuntimePolicy>(
            "crates/worth-signal/src/runtime_policy/definition.rs",
            "SignalRuntimePolicy",
            128 * 1024,
        );
        owner.field::<Option<u64>>("serial_memory_bytes");
        owner.field::<Option<usize>>("maximum_waiter_resolution_visits");
        owner.field::<Option<usize>>("maximum_upstream_dependency_visits");
        owner.field::<super::super::SignalConditionalEvaluationBudget>(
            "conditional_evaluation_budget",
        );
        owner.field::<super::super::SignalConditionalTemporalBudget>("conditional_temporal_budget");
        owner.field::<DiagnosticsTier>("tier");
        owner.field::<ExecutionObjectiveProfile>("execution_objective");
        owner.field::<ObservationActivationProfile>("observation_activation");
        owner.field::<RetentionBudget>("retention_budget");
        owner.field::<ReconstructionBudget>("reconstruction_budget");
        owner.field::<SnapshotRestoreLineageMode>("snapshot_restore_lineage_mode");
        owner.field::<FrontierTracingPolicy>("frontier_tracing_policy");
        owner.field::<FrontierPropagationPolicy>("frontier_propagation_policy");
        owner.field::<FrontierCyclePolicy>("frontier_cycle_policy");
        owner.field::<super::super::parallel::ParallelAdmissionPolicy>("parallel_admission");
    }
}
