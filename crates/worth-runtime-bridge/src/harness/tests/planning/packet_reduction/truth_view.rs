#[test]
fn bridge_bulk_packet_set_tracks_truth_view_materialization_packets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
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
    let snapshot_a = crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a");
    assert_eq!(
        planned
            .packet_set()
            .truth_view_packets()
            .iter()
            .map(|packet| packet.source_snapshot())
            .collect::<Vec<_>>(),
        vec![snapshot_a.as_str(), snapshot_a.as_str()]
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
    assert_eq!(planned.packet_set().counters().bulk_packet_count(), 5);
}

#[test]
fn bridge_bulk_reduction_artifact_carries_truth_view_materializations() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let left_source = InMemoryRelationalBridgeSource::default();
    left_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    left_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let left_runtime = build_runtime(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let right_source = InMemoryRelationalBridgeSource::default();
    right_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    right_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let right_runtime = build_runtime(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let left = left_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
                BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                ),
            )]),
            execution,
        )
        .expect("left bulk workload should plan");
    let right = right_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
                BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                ),
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
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").as_str()
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
