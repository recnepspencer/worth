#[test]
fn bridge_bulk_packet_set_tracks_continuity_remap_packets() {
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
    let lineage_context = BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                ))
                .with_mapping_context(
                    BridgeMappingContext::default().with_lineage_context(lineage_context.clone()),
                ),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                ))
                .with_mapping_context(
                    BridgeMappingContext::default().with_lineage_context(lineage_context),
                ),
            ]),
            execution,
        )
        .expect("continuity-bearing bulk workload should plan");

    assert_eq!(planned.packet_set().continuity_packets().len(), 2);
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.continuity_member_identity().clone())
            .collect::<std::collections::BTreeSet<_>>(),
        planned
            .canonical_request()
            .continuity_members()
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.originating_route_identity().clone())
            .collect::<Vec<_>>(),
        planned
            .packet_set()
            .routing_packets()
            .iter()
            .map(|packet| packet.route_identity().clone())
            .collect::<Vec<_>>()
    );
    let snapshot_a = crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a");
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.snapshot_identity())
            .collect::<Vec<_>>(),
        vec![snapshot_a.as_str(), snapshot_a.as_str()]
    );
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.prior_slice_count())
            .sum::<usize>(),
        2
    );
}

#[test]
fn bridge_bulk_reduction_artifact_carries_continuity_remaps() {
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
    let lineage_context = BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let request = BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
        BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
            "commit-a",
        )),
    )
    .with_mapping_context(BridgeMappingContext::default().with_lineage_context(lineage_context))]);

    let left = left_runtime
        .plan_bulk_workload(request.clone(), execution)
        .expect("left continuity-bearing workload should plan");
    let right = right_runtime
        .plan_bulk_workload(request, execution)
        .expect("right continuity-bearing workload should plan");

    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()
            .len(),
        1
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps(),
        right
            .execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()[0]
            .continuity_member_identity(),
        left.packet_set().continuity_packets()[0].continuity_member_identity()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()[0]
            .snapshot_identity(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").as_str()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()[0]
            .prior_slice_count(),
        1
    );
}

use super::*;
