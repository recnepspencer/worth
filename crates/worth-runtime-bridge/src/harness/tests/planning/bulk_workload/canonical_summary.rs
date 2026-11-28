use crate::harness::fixtures::NamedTruth;
#[test]
fn bridge_bulk_canonical_workload_request_carries_canonical_member_sets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, observation_b) = truth.commit_name(execution, "bob");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
            ]),
            execution,
        )
        .expect("bulk workload should plan");

    assert_eq!(
        planned.canonical_request().workload_identity(),
        planned.workload_identity()
    );
    assert_eq!(planned.canonical_request().route_members().len(), 2);
    assert_eq!(
        planned.canonical_request().route_members(),
        planned
            .planned_routes()
            .iter()
            .map(|route| route.route_identity().clone())
            .collect::<Vec<_>>()
    );
    let mut expected_subscription_slice_members = planned
        .planned_routes()
        .iter()
        .map(|route| {
            route
                .lowering_summary()
                .subscription_slice_identity()
                .clone()
        })
        .collect::<Vec<_>>();
    expected_subscription_slice_members.sort();
    expected_subscription_slice_members.dedup();
    assert_eq!(
        planned.canonical_request().subscription_slice_members(),
        expected_subscription_slice_members.as_slice()
    );
    assert_eq!(planned.canonical_request().truth_view_members().len(), 2);
    assert!(planned
        .canonical_request()
        .truth_view_members()
        .iter()
        .all(|member| member
            .as_str()
            .starts_with("bulk-truth-view-member:sha256:")));
    assert!(planned
        .canonical_request()
        .truth_view_members()
        .iter()
        .all(|member| !member.as_str().contains("snapshot-a")
            && !member.as_str().contains("commit-a")
            && !member.as_str().contains("main")));
    assert_eq!(
        planned.canonical_request().commit_members(),
        &[commit_a.clone(), commit_b.clone(),]
    );
    assert_eq!(
        planned.canonical_request().snapshot_members(),
        &[
            observation_a.snapshot_identity().clone(),
            observation_b.snapshot_identity().clone(),
        ]
    );
    assert_eq!(
        planned.canonical_request().branch_members(),
        &[crate::truth_identity_fixtures::truth_branch_fixture("main")]
    );
    assert_eq!(
        planned
            .canonical_request()
            .workload_segment_identities()
            .len(),
        2
    );
    assert!(planned
        .canonical_request()
        .workload_segment_identities()
        .iter()
        .all(|identity| identity
            .as_str()
            .starts_with("bulk-workload-segment:sha256:")));
    assert!(planned
        .canonical_request()
        .workload_segment_identities()
        .iter()
        .all(|identity| !identity.as_str().contains("segment|commit=")
            && !identity.as_str().contains("commit-a")
            && !identity.as_str().contains("commit-b")));
}

#[test]
fn bridge_bulk_normalized_summary_derives_shared_workload_facts_once() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, _observation_b) = truth.commit_name(execution, "bob");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
            ]),
            execution,
        )
        .expect("bulk workload should plan");

    assert_eq!(
        planned.normalized_summary().workload_identity(),
        planned.workload_identity()
    );
    assert_eq!(planned.normalized_summary().route_count(), 2);
    assert_eq!(planned.normalized_summary().subscription_slice_count(), 2);
    assert_eq!(planned.normalized_summary().snapshot_read_count(), 2);
    assert_eq!(planned.normalized_summary().truth_view_member_count(), 2);
    assert_eq!(planned.normalized_summary().continuity_member_count(), 0);
    assert_eq!(planned.normalized_summary().branch_scope_count(), 1);
    assert_eq!(planned.normalized_summary().snapshot_scope_count(), 2);
}

#[test]
fn bridge_bulk_execution_plan_selects_serial_for_single_route_workload() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
                BridgeRouteRequest::for_commit(commit_a.clone()),
            )]),
            execution,
        )
        .expect("single-route bulk workload should plan");

    assert_eq!(
        planned.execution_plan().selected_mode(),
        BridgePreparationMode::Serial
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::SerialRequired
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::BelowMinWorkloadWidth
    );
    assert_eq!(
        planned.execution_plan().legality_decision().class(),
        BridgeParallelLegalityClass::SerialOnly
    );
    assert_eq!(
        planned.execution_plan().legality_decision().reason(),
        BridgeParallelLegalityReason::BelowMinWorkloadWidth
    );
    assert_eq!(
        planned.execution_plan().profitability_decision().class(),
        BridgeParallelProfitabilityClass::NotApplicable
    );
    assert_eq!(
        planned.execution_plan().profitability_decision().reason(),
        BridgeParallelProfitabilityReason::SerialOnlyWorkload
    );
    assert_eq!(
        planned
            .execution_plan()
            .legality_proof()
            .admitted_partitions()
            .partitions()
            .len(),
        0
    );
    assert_eq!(
        planned
            .execution_plan()
            .reduced_artifact()
            .reduction_input_count(),
        3
    );
    assert_eq!(
        planned
            .execution_plan()
            .reduced_artifact()
            .reduction_output_count(),
        3
    );
    assert!(planned
        .execution_plan()
        .legality_proof()
        .disjoint_packet_regions()
        .regions()
        .is_empty());
    assert_eq!(planned.packet_set().routing_packets().len(), 1);
    assert_eq!(planned.packet_set().truth_view_packets().len(), 1);
    assert_eq!(planned.packet_set().reduction_packets().len(), 1);
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_serial_required_count(),
        1
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_profitable_count(),
        0
    );
    assert!(planned.execution_plan().planning_failures().is_empty());
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::BelowMinWorkloadWidth
    );
}

use super::*;
