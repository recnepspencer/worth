use super::*;

#[test]
fn logical_packet_identity_is_independent_of_the_callers_worker_count() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let targets = vec![
        RecordRef::Entity(create_entity_in_partition(
            execution,
            &runtime,
            "a",
            PartitionId(7),
        )),
        RecordRef::Entity(create_entity_in_partition(
            execution,
            &runtime,
            "b",
            PartitionId(11),
        )),
        RecordRef::Entity(create_entity_in_partition(
            execution,
            &runtime,
            "c",
            PartitionId(13),
        )),
    ];
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = planned_explicit_query(&runtime, &snapshot, "packet-identity", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .unwrap()
        .unwrap()
        .outcome;
    assert_eq!(expected.complexity.packet_count, 3);
    for width in [1, 4] {
        let caller = lease(width, MEMORY, CancellationToken::new());
        let actual = runtime.read_truth().execute_query_plan(
            plan.clone(),
            &worth_execution::ExecutionRequest::leased(&caller),
        );
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
