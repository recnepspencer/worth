use super::{
    build_runtime, committed_patch, registration, snapshot, BridgeBulkWorkloadRequest,
    BridgeBulkWorkloadSegment, BridgeRouteRequest, InMemoryRelationalBridgeSource,
    RecordingSignalBridgeSink,
};

#[test]
fn bridge_bulk_reduction_collapses_duplicate_publications_deterministically() {
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
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                )),
            ]),
            execution,
        )
        .expect("duplicate bulk workload should plan");
    let reduced_artifact = planned.execution_plan().reduced_artifact();

    assert_eq!(reduced_artifact.reduction_input_count(), 2);
    assert_eq!(reduced_artifact.reduction_output_count(), 2);
    assert_eq!(reduced_artifact.counters().bulk_reduction_input_count(), 2);
    assert_eq!(reduced_artifact.reduced_truth_views().len(), 1);
    assert_eq!(reduced_artifact.reduced_publications().len(), 1);
    assert_eq!(
        reduced_artifact.reduced_publications()[0]
            .reduced_route_identities()
            .len(),
        2
    );
    assert_eq!(
        reduced_artifact.reduced_publications()[0].reduced_route_identities(),
        planned
            .packet_set()
            .routing_packets()
            .iter()
            .map(|packet| packet.route_identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        reduced_artifact.reduced_publications()[0].invalidation_target_count(),
        2
    );
}
