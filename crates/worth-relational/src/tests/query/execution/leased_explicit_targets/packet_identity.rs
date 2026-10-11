use super::*;

#[test]
fn kernel_stop_keeps_the_packet_identity_at_every_worker_width() {
    let runtime = runtime_with_test_schema();
    let targets = vec![
        RecordRef::Entity(create_entity_in_partition(&runtime, "a", PartitionId(7))),
        RecordRef::Entity(create_entity_in_partition(
            &runtime,
            &"b".repeat(32_000),
            PartitionId(11),
        )),
        RecordRef::Entity(create_entity_in_partition(&runtime, "c", PartitionId(13))),
    ];
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = planned_explicit_query(&runtime, &snapshot, "packet-stop-identity", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    assert_eq!(expected.complexity.packet_count, 3);
    assert_eq!(expected.result.entities.len(), 3);
    // Packet zero fits; packet one exceeds its own result ceiling. The
    // reported stop must name packet one, even when there is only one worker.
    for width in [1, 2, 4] {
        let caller = lease(width, 128 << 10, CancellationToken::new());
        let stopped = runtime
            .read_truth()
            .execute_query_plan_with_lease(plan.clone(), &caller);
        assert!(matches!(stopped,
            Err(QueryReadExecutionStop::PacketStopped {
                boundary: Some(boundary),
                reason: MapStop::Failure {
                    identity,
                    cause: MapKernelFailure::ResultCapacityExceeded,
                }, ..
            }) if boundary == PartitionIdentity::new(2) && identity == boundary
        ), "a stopped packet must keep its identity below the packet count: width {width}, {stopped:?}");
    }
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
