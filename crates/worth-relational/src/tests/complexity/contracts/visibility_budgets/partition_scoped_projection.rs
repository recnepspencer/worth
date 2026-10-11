use crate::tests::support::*;

#[test]
fn complexity_budget_partition_scoped_historical_entity_scans_are_partition_bounded() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _left_a = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    let _left_b = create_entity_in_partition(execution, &runtime, "left-b", PartitionId(7));
    let historical_version = runtime.history().latest_commit().unwrap().version_id;
    let _right_a = create_entity_in_partition(execution, &runtime, "right-a", PartitionId(11));
    let _right_b = create_entity_in_partition(execution, &runtime, "right-b", PartitionId(11));

    runtime.performance_access().reset_counters();
    let records = runtime
        .read_truth()
        .project_historical_version(historical_version)
        .authoritative_entity_records_in(execution, PartitionId(7), KindId(1))
        .expect("read request remains live");
    let counters = runtime.performance_access().counters();

    assert_eq!(records.len(), 2);
    assert_eq!(counters.visibility_entity_slot_scans, 2);
}

#[test]
fn complexity_budget_partition_scoped_historical_relation_scans_are_partition_bounded() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left_source =
        create_entity_in_partition(execution, &runtime, "left-source", PartitionId(7));
    let left_target =
        create_entity_in_partition(execution, &runtime, "left-target", PartitionId(7));
    let right_source =
        create_entity_in_partition(execution, &runtime, "right-source", PartitionId(11));
    let right_target =
        create_entity_in_partition(execution, &runtime, "right-target", PartitionId(11));
    let _left_relation = create_relation_in_partition(
        execution,
        &runtime,
        left_source,
        left_target,
        "left-r0",
        PartitionId(7),
    );
    let historical_version = runtime.history().latest_commit().unwrap().version_id;
    let _right_relation = create_relation_in_partition(
        execution,
        &runtime,
        right_source,
        right_target,
        "right-r0",
        PartitionId(11),
    );

    runtime.performance_access().reset_counters();
    let records = runtime
        .read_truth()
        .project_historical_version(historical_version)
        .authoritative_relation_records_in(execution, PartitionId(7), KindId(2))
        .expect("read request remains live");
    let counters = runtime.performance_access().counters();

    assert_eq!(records.len(), 1);
    assert_eq!(counters.visibility_relation_slot_scans, 1);
}
