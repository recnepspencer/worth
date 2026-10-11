use crate::tests::support::*;

#[test]
fn complexity_budget_query_packetization_reports_parallel_shape_for_cross_partition_reads() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left_a = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    let left_b = create_entity_in_partition(execution, &runtime, "left-b", PartitionId(7));
    let right = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();

    runtime.performance_access().reset_counters();
    let lease = test_execution_lease();
    let _ = runtime
        .read_truth()
        .execute_query_plan(
            planned_explicit_query(
                &runtime,
                &snapshot,
                "cross-partition",
                vec![
                    crate::facade::transactions::RecordRef::Entity(left_a),
                    crate::facade::transactions::RecordRef::Entity(left_b),
                    crate::facade::transactions::RecordRef::Entity(right),
                ],
            ),
            &worth_execution::ExecutionRequest::leased(&lease),
        )
        .map(|read| read.map(|read| read.outcome))
        .expect("leased query execution")
        .expect("query outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.query_packet_count, 2);
    assert_eq!(counters.query_packet_item_count, 3);
    assert_eq!(counters.query_packet_peak_width_total, 2);
    assert_eq!(counters.query_scope_unit_count, 2);
    assert_eq!(counters.query_parallel_legal_count, 1);
    assert_eq!(counters.query_parallel_profitable_count, 1);
    assert_eq!(counters.query_staged_parallel_strategy_count, 1);
}

#[test]
fn complexity_budget_query_packetization_reports_narrow_shape_with_a_lease() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "single");
    for partition in 20..84 {
        let _ = create_entity_in_partition(
            execution,
            &runtime,
            &format!("unrelated-{partition}"),
            PartitionId(partition),
        );
    }
    let snapshot = runtime.visibility_authority().snapshot();

    runtime.performance_access().reset_counters();
    let lease = test_execution_lease();
    let _ = runtime
        .read_truth()
        .execute_query_plan(
            planned_explicit_query(
                &runtime,
                &snapshot,
                "single-target",
                vec![crate::facade::transactions::RecordRef::Entity(entity)],
            ),
            &worth_execution::ExecutionRequest::leased(&lease),
        )
        .map(|read| read.map(|read| read.outcome))
        .expect("leased query execution")
        .expect("query outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.query_packet_count, 1);
    assert_eq!(counters.query_packet_item_count, 1);
    assert_eq!(counters.query_packet_peak_width_total, 1);
    assert_eq!(counters.query_scope_unit_count, 1);
    assert_eq!(counters.query_parallel_legal_count, 1);
    assert_eq!(counters.query_parallel_profitable_count, 0);
    assert_eq!(counters.query_staged_parallel_strategy_count, 1);
    assert_eq!(counters.visibility_exact_state_materializations, 0);
    assert_eq!(counters.visibility_entity_slot_scans, 0);
    assert_eq!(counters.visibility_relation_slot_scans, 0);
}
