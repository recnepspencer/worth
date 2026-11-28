use crate::harness::fixtures::NamedTruth;
#[test]
fn bridge_bulk_packet_set_tracks_truth_view_materialization_packets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, observation_b) = truth.commit_name(execution, "bob");
    let shared_snapshot = observation_b.snapshot_identity().clone();
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        commit_a.clone(),
                        shared_snapshot.clone(),
                    ),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        commit_b.clone(),
                        shared_snapshot.clone(),
                    ),
                )),
            ]),
            execution,
        )
        .expect("bulk workload should plan");

    assert_eq!(planned.packet_set().truth_view_packets().len(), 2);
    assert_eq!(
        planned
            .packet_set()
            .truth_view_packets()
            .iter()
            .map(|packet| packet.truth_view_member_identity().clone())
            .collect::<Vec<_>>(),
        planned.canonical_request().truth_view_members()
    );
    let snapshot_a = observation_b.snapshot_identity().clone();
    assert_eq!(
        planned
            .packet_set()
            .truth_view_packets()
            .iter()
            .map(|packet| packet.source_snapshot())
            .collect::<Vec<_>>(),
        vec![snapshot_a.canonical_label(), snapshot_a.canonical_label()]
    );
    assert_eq!(
        planned
            .packet_set()
            .truth_view_packets()
            .iter()
            .map(|packet| packet.snapshot_read_count())
            .sum::<usize>(),
        2
    );
    assert_eq!(planned.packet_set().counters().bulk_packet_count(), 7);
}

#[test]
fn bridge_bulk_reduction_artifact_carries_truth_view_materializations() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, observation_a) = truth.commit_name(execution, "alice");
    let left_source = truth.source.clone();

    let left_runtime = build_runtime(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let right_source = truth.source.clone();

    let right_runtime = build_runtime(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let left = left_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
                BridgeRouteRequest::for_commit(commit_a.clone()),
            )]),
            execution,
        )
        .expect("left bulk workload should plan");
    let right = right_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
                BridgeRouteRequest::for_commit(commit_a.clone()),
            )]),
            execution,
        )
        .expect("right bulk workload should plan");

    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_truth_views()
            .len(),
        1
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_truth_views(),
        right
            .execution_plan()
            .reduced_artifact()
            .reduced_truth_views()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_truth_views()[0]
            .truth_view_member_identity(),
        left.packet_set().truth_view_packets()[0].truth_view_member_identity()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_truth_views()[0]
            .source_snapshot(),
        observation_a.snapshot_identity().clone().canonical_label()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_truth_views()[0]
            .snapshot_read_count(),
        1
    );
}

use super::*;
