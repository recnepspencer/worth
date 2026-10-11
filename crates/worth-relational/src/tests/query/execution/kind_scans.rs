use super::*;

#[test]
fn planned_query_execution_supports_entity_kind_scans_through_reducer_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    let _other_left = create_entity_in_partition(execution, &runtime, "left-b", PartitionId(7));
    let right = create_entity_in_partition(execution, &runtime, "right-a", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "entity-kind-scan".to_string(),
        context_id: context,
        scope: QueryScope::EntityKindScan {
            kind_id: KindId(1),
            partition_scope: Some(Arc::from([PartitionId(7), PartitionId(11)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7), PartitionId(11)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(501),
        target_count_hint: 0,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("query execution outcome");

    assert_eq!(outcome.complexity.packet_count, 2);
    assert_eq!(outcome.complexity.fragment_count, 2);
    assert_eq!(outcome.complexity.authoritative_entity_records_emitted, 3);
    assert_eq!(outcome.result.entities.len(), 3);
    assert_eq!(outcome.result.entities[0].entity_id, left);
    assert_eq!(outcome.result.entities[2].entity_id, right);
}

#[test]
fn planned_query_execution_reports_non_zero_packet_items_for_kind_scans() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let _left = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    let _right = create_entity_in_partition(execution, &runtime, "right-a", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "entity-kind-scan-accounting".to_string(),
        context_id: context,
        scope: QueryScope::EntityKindScan {
            kind_id: KindId(1),
            partition_scope: Some(Arc::from([PartitionId(7), PartitionId(11)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7), PartitionId(11)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(502),
        target_count_hint: 0,
    };

    runtime.performance_access().reset_counters();
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let _ = runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("query execution outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.query_packet_count, 2);
    assert_eq!(counters.query_packet_item_count, 2);
}

#[test]
fn planned_query_execution_supports_relation_kind_scans_through_reducer_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity_in_partition(execution, &runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));
    let third = create_entity_in_partition(execution, &runtime, "third", PartitionId(11));
    let first_relation =
        create_relation_in_partition(execution, &runtime, left, right, "r1", PartitionId(7));
    let second_relation =
        create_relation_in_partition(execution, &runtime, right, third, "r2", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "relation-kind-scan".to_string(),
        context_id: context,
        scope: QueryScope::RelationKindScan {
            kind_id: KindId(2),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(777),
        target_count_hint: 0,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("query execution outcome");

    assert_eq!(outcome.complexity.packet_count, 2);
    assert_eq!(outcome.complexity.authoritative_relation_records_emitted, 2);
    assert_eq!(
        outcome
            .result
            .relations
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        vec![first_relation, second_relation]
    );
}
