use crate::facade::*;
use crate::tests::support::*;

use crate::data::telemetry::{
    CheckpointTelemetry, EvaluationTelemetry, ExecutionTelemetry, InvalidationTelemetry,
    PlannerTelemetry, StorageTelemetry, TemporalTelemetry, TransactionTelemetry,
};

#[test]
fn diagnostics_profile_reset_restores_stock_tier_policy() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        SignalRuntimePolicy::forensic()
            .with_history_limit(23)
            .with_detail_limit(41)
            .with_history_details(false),
    );

    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Operational);

    assert_eq!(graph.runtime_policy(), SignalRuntimePolicy::operational());
}

#[test]
fn node_contract_and_runtime_policy_expose_s9_1_enforcement_surfaces() {
    let contract = NodeContract::reads([ASPECT_A])
        .with_equivalence(EquivalenceContract::for_comparator_override(
            &VersionComparatorPolicy::Exact,
        ))
        .with_path_class(PathClass::Rich)
        .with_maintenance_mode(MaintenanceMode::RebuildAllowed)
        .with_artifact_policy(ArtifactPolicyClass::DevelopmentRetained);
    let compile_time = contract.compile_time_performance_contract();
    let resolved = SignalRuntimePolicy::development().resolve_performance_policy();

    assert_eq!(compile_time.equivalence, contract.execution.equivalence);
    assert_eq!(compile_time.path_class, PathClass::Rich);
    assert_eq!(
        compile_time.maintenance_mode,
        MaintenanceMode::RebuildAllowed
    );
    assert_eq!(
        compile_time.artifact_policy,
        ArtifactPolicyClass::DevelopmentRetained
    );
    assert_eq!(
        compile_time.authority_policy,
        AuthorityPolicy::SpeculativeThenReconcile
    );

    assert_eq!(resolved.path_class, PathClass::Rich);
    assert_eq!(
        resolved.artifact_policy,
        ArtifactPolicyClass::DevelopmentRetained
    );
    assert_eq!(
        resolved.execution_strategy,
        ResolvedExecutionStrategy::DenseStageBatched
    );
    assert_eq!(
        resolved.maintenance_strategy,
        ResolvedMaintenanceStrategy::Incremental
    );
    assert_eq!(
        resolved.authority_policy,
        AuthorityPolicy::SpeculativeThenReconcile
    );
}

#[test]
fn runtime_telemetry_exposes_performance_counter_surface() {
    let telemetry = RuntimeTelemetry {
        evaluation: EvaluationTelemetry {
            evaluation_calls: 3,
            ..EvaluationTelemetry::default()
        },
        invalidation: InvalidationTelemetry {
            invalidation_nodes_visited: 5,
            ..InvalidationTelemetry::default()
        },
        transaction: TransactionTelemetry {
            transaction_commit_count: 2,
            ..TransactionTelemetry::default()
        },
        planner: PlannerTelemetry {
            stages_built: 7,
            ..PlannerTelemetry::default()
        },
        execution: ExecutionTelemetry {
            rewiring_apply_count: 11,
            ..ExecutionTelemetry::default()
        },
        storage: StorageTelemetry {
            graph_storage_snapshot_rewrites: 13,
            ..StorageTelemetry::default()
        },
        checkpoint: CheckpointTelemetry {
            checkpoint_flushes: 17,
            ..CheckpointTelemetry::default()
        },
        temporal: TemporalTelemetry {
            temporal_wake_count: 19,
            scheduled_frontier_width: 29,
            temporal_eligibility_lowering_count: 31,
            previous_value_reference_count: 23,
            ..TemporalTelemetry::default()
        },
        resource: ResourceTelemetry {
            resource_declaration_lowering_count: 37,
            resource_descriptor_count: 41,
            ..ResourceTelemetry::default()
        },
        host_computed: crate::data::telemetry::HostComputedTelemetry::default(),
    };
    let counters = telemetry.performance_counter_surface();

    assert_eq!(counters.evaluation.evaluation_calls, 3);
    assert_eq!(counters.invalidation.invalidation_nodes_visited, 5);
    assert_eq!(counters.transaction.transaction_commit_count, 2);
    assert_eq!(counters.planner.stages_built, 7);
    assert_eq!(counters.execution.rewiring_apply_count, 11);
    assert_eq!(counters.storage.graph_storage_snapshot_rewrites, 13);
    assert_eq!(counters.checkpoint.checkpoint_flushes, 17);
    assert_eq!(counters.temporal.temporal_wake_count, 19);
    assert_eq!(counters.temporal.scheduled_frontier_width, 29);
    assert_eq!(counters.temporal.temporal_eligibility_lowering_count, 31);
    assert_eq!(counters.temporal.previous_value_reference_count, 23);
    assert_eq!(counters.resource.resource_declaration_lowering_count, 37);
    assert_eq!(counters.resource.resource_descriptor_count, 41);
}
