use crate::facade::history::BranchId;
use crate::facade::lineage::{
    HistoricalResolutionBoundednessBasis, HistoricalResolutionRequest, LineageDivergenceRequest,
    LineageDivergenceTraversalBasis, LineageGraphRequest, LineageGraphTraversalBasis,
};
use crate::facade::replay::{
    RelationalReplayRequest, ReplayAuthorityBasisKind, ReplayExecutionMode,
    ReplayLineageDigestMode, ReplayVerificationMode,
};
use crate::tests::support::*;

#[test]
fn complexity_budget_lineage_historical_resolution_reports_branch_scoped_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .lineage_id;

    for index in 0..5 {
        let label = format!("noise-{index}");
        let _ = create_entity_outcome(execution, &runtime, &label);
    }

    runtime.performance_access().reset_counters();
    let resolution = runtime
        .lineage_access()
        .resolve_historical_lineage(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
        )
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.lineage_historical_resolution_requests, 1);
    assert_eq!(
        counters.lineage_historical_resolution_event_visits,
        resolution.metrics.event_visit_count
    );
    assert_eq!(
        counters.lineage_historical_resolution_traversed_events,
        resolution.metrics.traversed_event_count
    );
    assert_eq!(resolution.metrics.event_visit_count, 0);
}

#[test]
fn complexity_budget_lineage_branch_divergence_reports_breadth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _main = create_entity_outcome(execution, &runtime, "main");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );

    runtime.performance_access().reset_counters();
    let divergence = runtime
        .lineage_access()
        .divergence_between_branches(
            execution,
            LineageDivergenceRequest {
                left_branch: BranchId("main".to_string()),
                right_branch: BranchId("feature".to_string()),
                traversal_basis: LineageDivergenceTraversalBasis::FullBranchGraphComparison,
            },
        )
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.lineage_branch_divergence_requests, 1);
    assert_eq!(
        counters.lineage_branch_divergence_event_scans,
        divergence.metrics.left_event_count + divergence.metrics.right_event_count
    );
    assert_eq!(
        counters.lineage_branch_divergence_node_scans,
        divergence.metrics.left_node_count + divergence.metrics.right_node_count
    );
    assert!(counters.lineage_graph_snapshot_requests >= 2);
    assert_eq!(
        counters.lineage_graph_snapshot_nodes_materialized,
        divergence.metrics.left_node_count + divergence.metrics.right_node_count
    );
    assert!(
        counters.visible_authoritative_entity_records_materialized
            >= counters.lineage_graph_snapshot_nodes_materialized
    );
    assert!(counters.visibility_cache_hits + counters.visibility_cache_miss_reconstructions >= 2);
    assert_eq!(
        counters.lineage_graph_snapshot_visibility_cache_hits
            + counters.lineage_graph_snapshot_visibility_cache_miss_reconstructions,
        counters.lineage_graph_snapshot_requests
    );
    assert!(
        counters.lineage_graph_snapshot_visibility_cache_hits
            + counters.lineage_graph_snapshot_visibility_cache_miss_reconstructions
            >= 2
    );
}

#[test]
fn complexity_budget_lineage_graph_snapshot_reports_full_breadth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "left");
    create_entity_outcome(execution, &runtime, "right");

    runtime.performance_access().reset_counters();
    let graph = runtime
        .lineage_access()
        .graph(
            execution,
            LineageGraphRequest {
                branch_id: BranchId("main".to_string()),
                traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
            },
        )
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.lineage_graph_snapshot_requests, 1);
    assert_eq!(
        counters.lineage_graph_snapshot_nodes_materialized,
        graph.metrics.node_count
    );
    assert_eq!(
        counters.lineage_graph_snapshot_events_materialized,
        graph.metrics.event_count
    );
}

#[test]
fn complexity_budget_replay_lineage_parity_reports_authority_basis_and_digest_width() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "source");
    let second = create_entity_outcome(execution, &runtime, "target");
    let replay_commit_id = second.commit.commit_id;

    runtime.performance_access().reset_counters();
    let durable_replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: replay_commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(
        durable_replay
            .lineage_authority_basis
            .as_ref()
            .map(|basis| basis.kind()),
        Some(ReplayAuthorityBasisKind::DurableLogCanonical)
    );
    assert_eq!(
        durable_replay
            .lineage_authority_basis
            .as_ref()
            .map(|basis| basis.digest_mode()),
        Some(ReplayLineageDigestMode::ExactCanonicalArtifactDigest)
    );
    assert_eq!(counters.replay_lineage_authority_lookup_requests, 1);
    assert_eq!(counters.replay_lineage_log_index_hits, 1);
    assert_eq!(counters.replay_lineage_checkpoint_index_hits, 0);
    assert_eq!(counters.replay_lineage_durable_basis_selections, 1);
    assert_eq!(
        counters.replay_lineage_retained_envelope_basis_selections,
        0
    );
    assert_eq!(counters.replay_lineage_authoritative_basis_rejections, 0);
    assert_eq!(
        counters.replay_lineage_digest_event_width,
        durable_replay
            .lineage_authority_basis
            .as_ref()
            .unwrap()
            .lineage_event_count()
    );
    assert_eq!(
        counters.replay_lineage_digest_decision_width,
        durable_replay
            .lineage_authority_basis
            .as_ref()
            .unwrap()
            .lineage_decision_count()
    );

    assert!(runtime
        .durability_authority()
        .remove_durable_envelope_for_test(replay_commit_id));
    runtime.performance_access().reset_counters();
    let retained_envelope_replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: replay_commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    let counters = runtime.performance_access().counters();
    assert_eq!(
        retained_envelope_replay
            .lineage_authority_basis
            .as_ref()
            .map(|basis| basis.kind()),
        Some(ReplayAuthorityBasisKind::RetainedEnvelopeCanonical)
    );
    assert_eq!(
        retained_envelope_replay
            .lineage_authority_basis
            .as_ref()
            .map(|basis| basis.digest_mode()),
        Some(ReplayLineageDigestMode::ExactCanonicalArtifactDigest)
    );
    assert_eq!(counters.replay_lineage_authority_lookup_requests, 1);
    assert_eq!(counters.replay_lineage_log_index_hits, 0);
    assert_eq!(counters.replay_lineage_checkpoint_index_hits, 0);
    assert_eq!(counters.replay_lineage_durable_basis_selections, 0);
    assert_eq!(
        counters.replay_lineage_retained_envelope_basis_selections,
        1
    );
    assert_eq!(counters.replay_lineage_authoritative_basis_rejections, 0);
}
