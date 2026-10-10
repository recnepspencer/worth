use super::*;

#[test]
fn planned_query_execution_supports_outgoing_neighborhood_with_canonical_traversal_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&runtime, "b", PartitionId(11));
    let c = create_entity_in_partition(&runtime, "c", PartitionId(13));
    let first_relation = create_relation_in_partition(&runtime, a, b, "ab", PartitionId(7));
    let second_relation = create_relation_in_partition(&runtime, a, c, "ac", PartitionId(13));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "outgoing-neighborhood".to_string(),
        context_id: context,
        scope: QueryScope::OutgoingNeighborhood {
            seeds: Arc::from([a]),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(902),
        target_count_hint: 1,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(
            plan,
            execution,
            crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("query execution outcome");

    assert_eq!(outcome.complexity.packet_count, 1);
    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![a, b, c]
    );
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

#[test]
fn planned_query_execution_supports_incoming_neighborhood_with_canonical_traversal_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&runtime, "b", PartitionId(11));
    let c = create_entity_in_partition(&runtime, "c", PartitionId(13));
    let first_relation = create_relation_in_partition(&runtime, a, c, "ac", PartitionId(7));
    let second_relation = create_relation_in_partition(&runtime, b, c, "bc", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "incoming-neighborhood".to_string(),
        context_id: context,
        scope: QueryScope::IncomingNeighborhood {
            seeds: Arc::from([c]),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(905),
        target_count_hint: 1,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(
            plan,
            execution,
            crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("query execution outcome");

    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![c, a, b]
    );
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

#[test]
fn planned_query_execution_supports_connectivity_traversal_with_depth_bound() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&runtime, "b", PartitionId(11));
    let c = create_entity_in_partition(&runtime, "c", PartitionId(13));
    let first_relation = create_relation_in_partition(&runtime, a, b, "ab", PartitionId(7));
    let _second_relation = create_relation_in_partition(&runtime, b, c, "bc", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "connectivity-traversal".to_string(),
        context_id: context,
        scope: QueryScope::ConnectivityTraversal {
            seeds: Arc::from([a]),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
            max_depth: Some(1),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(903),
        target_count_hint: 1,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(
            plan,
            execution,
            crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("query execution outcome");

    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![a, b]
    );
    assert_eq!(
        outcome
            .result
            .relations
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        vec![first_relation]
    );
    assert_eq!(outcome.complexity.authoritative_entity_records_emitted, 2);
    assert_eq!(outcome.complexity.authoritative_relation_records_emitted, 1);
}

#[test]
fn planned_query_execution_normalizes_traversal_seed_order_deterministically() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&runtime, "b", PartitionId(11));
    let c = create_entity_in_partition(&runtime, "c", PartitionId(13));
    let _first_relation = create_relation_in_partition(&runtime, a, c, "ac", PartitionId(7));
    let _second_relation = create_relation_in_partition(&runtime, b, c, "bc", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");

    let descending_packet = PlannedQueryPacket {
        label: "incoming-neighborhood-desc".to_string(),
        context_id: context.clone(),
        scope: QueryScope::IncomingNeighborhood {
            seeds: Arc::from([c, a, c]),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(906),
        target_count_hint: 3,
    };
    let ascending_packet = PlannedQueryPacket {
        label: "incoming-neighborhood-asc".to_string(),
        context_id: context,
        scope: QueryScope::IncomingNeighborhood {
            seeds: Arc::from([a, c]),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(907),
        target_count_hint: 2,
    };

    let descending = runtime
        .read_truth()
        .execute_query_plan(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, descending_packet)
                .expect("descending plan"),
            execution,
            crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("descending outcome");
    let ascending = runtime
        .read_truth()
        .execute_query_plan(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, ascending_packet)
                .expect("ascending plan"),
            execution,
            crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("ascending outcome");

    assert_eq!(descending.result.entities, ascending.result.entities);
    assert_eq!(descending.result.relations, ascending.result.relations);
    assert_eq!(
        descending.result.reduction_digest,
        ascending.result.reduction_digest
    );
}
