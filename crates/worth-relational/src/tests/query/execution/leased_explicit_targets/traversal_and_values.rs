use super::*;

#[test]
fn leased_traversal_stops_inside_high_fanout() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let seed = create_entity_in_partition(&runtime, "seed", PartitionId(7));
    for ordinal in 0..24 {
        let neighbor =
            create_entity_in_partition(&runtime, &format!("neighbor-{ordinal}"), PartitionId(11));
        create_relation_in_partition(
            &runtime,
            seed,
            neighbor,
            &format!("edge-{ordinal}"),
            PartitionId(13),
        );
    }
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-traversal-ceiling".to_owned(),
                context_id: context,
                scope: QueryScope::OutgoingNeighborhood {
                    seeds: Arc::from([seed]),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_303),
                target_count_hint: 1,
            },
        )
        .unwrap();
    let unleased = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .unwrap();
    assert_eq!(unleased.result.relations.len(), 24);

    let constrained = lease_with_work(1, 1 << 20, 10, CancellationToken::new());
    let actual = runtime
        .read_truth()
        .execute_query_plan(
            plan,
            &worth_execution::ExecutionRequest::leased(&constrained),
        )
        .map(|read| read.map(|read| read.outcome));
    assert!(
        matches!(
            &actual,
            Err(QueryReadExecutionStop::PreparationStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            }) | Err(QueryReadExecutionStop::PacketStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            })
        ),
        "{actual:?}"
    );
}

#[test]
fn leased_any_of_scan_matches_serial_with_reversed_values() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(&runtime, "alpha", PartitionId(7));
    create_entity_in_partition(&runtime, "omega", PartitionId(7));
    create_entity_in_partition(&runtime, "other", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-any-of-order".to_owned(),
                context_id: context,
                scope: QueryScope::EntityFieldAnyOf {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    values: Arc::from([string_aspect_value("omega"), string_aspect_value("alpha")]),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([PartitionId(7)]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_305),
                target_count_hint: 2,
            },
        )
        .unwrap();
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .unwrap();
    let actual = runtime
        .read_truth()
        .execute_query_plan(
            plan.clone(),
            &worth_execution::ExecutionRequest::leased(&lease(
                1,
                1 << 20,
                CancellationToken::new(),
            )),
        )
        .map(|read| read.map(|read| read.outcome))
        .unwrap()
        .unwrap();
    assert_eq!(actual.result, expected.result);
    assert_eq!(actual.result.entities.len(), 2);
}
