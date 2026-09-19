use crate::facade::*;
use crate::tests::support::*;

use crate::data::telemetry::{
    CheckpointTelemetry, EvaluationTelemetry, ExecutionTelemetry, InvalidationTelemetry,
    PlannerTelemetry, StorageTelemetry, TemporalTelemetry, TransactionTelemetry,
};

use super::source_corpus::{
    ENTRIES_SOURCE, GRAPH_RUNTIME_SOURCE, PERFORMANCE_BASELINE_SOURCE, PERFORMANCE_SUPPORT_SOURCE,
    PERSISTENT_PAGED_VECTOR_SOURCE, PERSISTENT_VECTOR_SOURCE, SLOT_SOURCE,
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

    assert_eq!(PerformanceEnforcementLayer::CompileTime as u8, 0);
    assert_eq!(PerformanceEnforcementLayer::RuntimePolicy as u8, 1);
    assert_eq!(PerformanceEnforcementLayer::CounterTest as u8, 2);

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

#[test]
fn performance_harness_emits_allocation_and_footprint_metrics() {
    assert!(
        PERFORMANCE_SUPPORT_SOURCE.contains("#[global_allocator]")
            && PERFORMANCE_SUPPORT_SOURCE.contains("StatsAlloc")
            && PERFORMANCE_SUPPORT_SOURCE.contains("INSTRUMENTED_SYSTEM"),
        "performance harness should provide a process-wide allocation instrumentation surface for certification runs"
    );
    assert!(
        PERFORMANCE_SUPPORT_SOURCE.contains("\"allocation_metrics\"")
            && PERFORMANCE_SUPPORT_SOURCE.contains("\"allocated_bytes\"")
            && PERFORMANCE_SUPPORT_SOURCE.contains("\"peak_live_bytes\"")
            && PERFORMANCE_SUPPORT_SOURCE.contains("\"access_counters\""),
        "performance harness should emit allocation, heap-footprint, and compatibility-access counters with each perf sample"
    );
    assert!(
        PERFORMANCE_SUPPORT_SOURCE.contains("PERF_ALLOC_LOCK")
            && PERFORMANCE_SUPPORT_SOURCE.contains("Region::new(GLOBAL_ALLOCATOR)")
            && PERFORMANCE_SUPPORT_SOURCE.contains("snapshot_allocation_stats(&region)")
            && PERFORMANCE_SUPPORT_SOURCE.contains("WORTH_SIGNAL_UPDATE_PERF_BASELINE")
            && PERFORMANCE_SUPPORT_SOURCE.contains("performance_baseline.json"),
        "allocation instrumentation should serialize perf measurements and persist a checked baseline/delta certification surface"
    );
}

#[test]
fn node_storage_is_physically_split_into_index_addressed_lanes() {
    assert!(
        GRAPH_RUNTIME_SOURCE.contains("pub(in crate::data::graph) hot:")
            && GRAPH_RUNTIME_SOURCE.contains("PersistentPagedVector<Option<NodeHotData>>")
            && GRAPH_RUNTIME_SOURCE.contains("pub(in crate::data::graph) warm:")
            && GRAPH_RUNTIME_SOURCE.contains("PersistentPagedVector<NodeWarmData>")
            && GRAPH_RUNTIME_SOURCE.contains("pub(in crate::data::graph) cold:")
            && GRAPH_RUNTIME_SOURCE.contains("PersistentPagedVector<Option<Box<NodeColdData>>>")
            && PERSISTENT_PAGED_VECTOR_SOURCE.contains("PersistentVector<T, 64>")
            && PERSISTENT_VECTOR_SOURCE.contains("Exclusive(Vec<T>)")
            && PERSISTENT_VECTOR_SOURCE.contains("ForkShared {")
            && PERSISTENT_VECTOR_SOURCE.contains("base: Arc<RetainedStorageBacking<Vec<T>>>")
            && PERSISTENT_VECTOR_SOURCE.contains("changed_pages: im::OrdMap")
            && PERSISTENT_VECTOR_SOURCE.contains("install_changed_page::<T, PAGE_LEN>")
            && PERSISTENT_VECTOR_SOURCE.contains("Arc::make_mut(")
            && PERSISTENT_VECTOR_SOURCE.contains("pub(crate) fn get_mut"),
        "node arena should stay flat ordinarily and detach bounded COW pages after exact forks"
    );
    assert!(
        !SLOT_SOURCE.contains("Option<NodeEntry>"),
        "slot metadata should no longer store whole NodeEntry payloads inline"
    );
    assert!(
        ENTRIES_SOURCE.contains("NodeEntry::from_storage_parts(")
            && ENTRIES_SOURCE.contains("entry.into_storage_parts()"),
        "broad NodeEntry access should now be compatibility assembly over split node lanes"
    );
}

#[test]
fn performance_profiles_are_baseline_gated_not_report_only() {
    assert!(
        PERFORMANCE_SUPPORT_SOURCE.contains("capture_and_certify_perf_samples")
            && PERFORMANCE_SUPPORT_SOURCE.contains("certify_against_baseline")
            && PERFORMANCE_SUPPORT_SOURCE.contains("performance_baseline.json"),
        "ignored performance profiles should certify against a committed baseline artifact"
    );
    assert!(
        PERFORMANCE_BASELINE_SOURCE.contains("\"version\"")
            && PERFORMANCE_BASELINE_SOURCE.contains("\"cases\""),
        "performance baseline artifact should be present in-repo for certification runs"
    );
}
