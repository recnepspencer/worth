use super::*;

#[test]
fn logical_packet_identity_is_independent_of_the_callers_worker_count() {
    let runtime = runtime_with_test_schema();
    let targets = vec![
        RecordRef::Entity(create_entity_in_partition(&runtime, "a", PartitionId(7))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "b", PartitionId(11))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "c", PartitionId(13))),
    ];
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = planned_explicit_query(&runtime, &snapshot, "packet-identity", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    assert_eq!(expected.complexity.packet_count, 3);
    for width in [1, 4] {
        let caller = lease(width, MEMORY, CancellationToken::new());
        let actual = runtime
            .read_truth()
            .execute_query_plan_with_lease_report(plan.clone(), &caller);
        assert!(
            actual.is_ok(),
            "logical packet identity must be independent of worker count: {actual:?}"
        );
        let actual = actual.unwrap().unwrap();
        assert_eq!(
            actual.outcome.result, expected.result,
            "logical packet identity must preserve results at every worker count"
        );
        assert_eq!(
            actual.outcome.complexity.packet_count, 3,
            "logical packets must retain their identity at every worker count"
        );
    }
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
