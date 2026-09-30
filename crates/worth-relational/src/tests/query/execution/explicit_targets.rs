use super::*;

#[test]
fn planned_query_execution_reduces_explicit_targets_into_canonical_entity_order() {
    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "first");
    let second = create_entity_outcome(&runtime, "second");
    let first_id = changed_entities(&first)[0];
    let second_id = changed_entities(&second)[0];

    let outcome = execute_explicit_query(
        &runtime,
        &second.snapshot,
        "reverse-order",
        vec![RecordRef::Entity(second_id), RecordRef::Entity(first_id)],
    );

    assert_eq!(
        outcome.result.ordering,
        QueryOrderingContract::CanonicalRecordRefOrder
    );
    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![first_id, second_id]
    );
    assert_eq!(outcome.complexity.packet_count, 1);
    assert_eq!(outcome.complexity.fragment_count, 1);
    assert_eq!(outcome.complexity.target_count, 2);
    assert_eq!(outcome.complexity.authoritative_entity_records_emitted, 2);
    assert_eq!(outcome.complexity.authoritative_relation_records_emitted, 0);
}

#[test]
fn planned_query_execution_is_deterministic_for_identical_inputs() {
    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "first");
    let second = create_entity_outcome(&runtime, "second");
    let first_id = changed_entities(&first)[0];
    let second_id = changed_entities(&second)[0];
    let first_outcome = execute_explicit_query(
        &runtime,
        &second.snapshot,
        "stable-execution",
        vec![RecordRef::Entity(second_id), RecordRef::Entity(first_id)],
    );
    let second_outcome = execute_explicit_query(
        &runtime,
        &second.snapshot,
        "stable-execution",
        vec![RecordRef::Entity(second_id), RecordRef::Entity(first_id)],
    );

    assert_eq!(first_outcome.result, second_outcome.result);
    assert_eq!(first_outcome.complexity, second_outcome.complexity);
}

#[test]
fn unleased_query_execution_keeps_multi_packet_reads_serial() {
    let runtime = runtime_with_test_schema();
    let targets = vec![
        RecordRef::Entity(create_entity_in_partition(&runtime, "a-1", PartitionId(7))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "b-1", PartitionId(11))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "a-2", PartitionId(7))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "b-2", PartitionId(11))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "a-3", PartitionId(7))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "b-3", PartitionId(11))),
    ];
    let snapshot = runtime.visibility_authority().snapshot();
    runtime.performance_access().reset_counters();

    let outcome = execute_explicit_query(&runtime, &snapshot, "parallel-query", targets);
    let counters = runtime.performance_access().counters();

    assert_eq!(outcome.complexity.fragment_count, 2);
    assert_eq!(outcome.complexity.touched_partitions, 2);
    assert_eq!(counters.query_packet_count, 2);
    assert_eq!(counters.query_packet_item_count, 6);
    assert_eq!(counters.query_parallel_legal_count, 1);
    assert_eq!(counters.query_parallel_profitable_count, 1);
    assert_eq!(counters.query_staged_parallel_strategy_count, 0);
    assert_eq!(counters.query_serial_strategy_count, 1);
    assert_eq!(counters.query_authoritative_entity_records_emitted, 6);
    assert_eq!(counters.query_authoritative_relation_records_emitted, 0);
}

#[test]
fn planned_query_execution_explicit_targets_do_not_claim_fragment_scratch_reuse() {
    let runtime = runtime_with_test_schema();
    let left = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(&runtime, "right", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();

    runtime.performance_access().reset_counters();
    let outcome = execute_explicit_query(
        &runtime,
        &snapshot,
        "explicit-targets",
        vec![
            crate::facade::transactions::RecordRef::Entity(left),
            crate::facade::transactions::RecordRef::Entity(right),
        ],
    );
    let counters = runtime.performance_access().counters();

    assert_eq!(outcome.complexity.packet_count, 2);
    assert_eq!(counters.query_fragment_scratch_reuse_count, 0);
}
