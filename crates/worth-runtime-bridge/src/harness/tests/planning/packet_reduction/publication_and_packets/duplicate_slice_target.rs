use super::{
    build_runtime, committed_patch, registration, snapshot, BridgeBulkWorkloadRequest,
    BridgeBulkWorkloadSegment, BridgeInvalidationReductionFamily, BridgeParallelAdmissionReason,
    BridgeRouteRequest, InMemoryRelationalBridgeSource, RecordingSignalBridgeSink,
};

#[test]
fn bridge_bulk_packet_reduction_collapses_duplicate_slice_targets() {
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
        .expect("duplicate workload should plan");

    assert_eq!(planned.packet_set().routing_packets().len(), 2);
    assert_eq!(planned.packet_set().truth_view_packets().len(), 1);
    assert_eq!(planned.packet_set().reduction_packets().len(), 1);
    assert_eq!(
        planned.packet_set().reduction_packets()[0].reduction_family(),
        BridgeInvalidationReductionFamily::Publication
    );
    assert_eq!(
        planned.packet_set().reduction_packets()[0].reduced_subscription_slice_identity(),
        planned.packet_set().routing_packets()[0].subscription_slice_identity()
    );
    assert_eq!(planned.packet_set().counters().bulk_packet_count(), 4);
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::SharedTruthViewMaterializationTarget
    );
    assert_eq!(
        planned
            .execution_plan()
            .locality_footprint()
            .publication_scope_count(),
        1
    );
}
