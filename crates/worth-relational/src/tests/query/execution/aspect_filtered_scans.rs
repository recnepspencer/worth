use super::*;

#[test]
fn planned_query_execution_supports_aspect_filtered_entity_scans_through_reducer_path() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity_in_partition(&runtime, "left-a", PartitionId(7));
    let right = create_entity_in_partition(&runtime, "right-a", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "aspect-filtered-entities".to_string(),
        context_id: context,
        scope: QueryScope::AspectFilteredEntities {
            kind_id: Some(KindId(1)),
            aspect_filter: ProjectionAspectFilter::whole_aspects(
                ProjectionAspectFilterMode::All,
                [aspect_key("name")],
            ),
            partition_scope: Some(Arc::from([PartitionId(7), PartitionId(11)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7), PartitionId(11)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(901),
        target_count_hint: 0,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .expect("query execution outcome");

    assert_eq!(outcome.complexity.packet_count, 2);
    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![left, right]
    );
}

#[test]
fn planned_query_execution_aspect_filter_requires_projected_authoritative_presence() {
    let mut fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    fixture.entity_aspects.push(entity_field_aspect(
        aspect_key("status"),
        field_key("status"),
    ));
    let runtime = fixture.build_runtime();
    create_entity_in_partition(&runtime, "declared-without-status", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "actual-authoritative-aspect-filter".to_string(),
        context_id: context,
        scope: QueryScope::AspectFilteredEntities {
            kind_id: Some(KindId(1)),
            aspect_filter: ProjectionAspectFilter::whole_aspects(
                ProjectionAspectFilterMode::All,
                [aspect_key("status")],
            ),
            partition_scope: Some(Arc::from([PartitionId(7)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(903),
        target_count_hint: 0,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .expect("query execution outcome");

    assert_eq!(outcome.result.entities.len(), 0);
    assert_eq!(outcome.complexity.authoritative_entity_records_emitted, 0);
}

#[test]
fn planned_query_execution_aspect_filter_supports_field_mask_presence() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let matched = create_entity_with_summary_title(&runtime, "matched", "visible-title");
    create_entity_in_partition(&runtime, "missing-summary", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "field-mask-aspect-filter".to_string(),
        context_id: context,
        scope: QueryScope::AspectFilteredEntities {
            kind_id: Some(KindId(1)),
            aspect_filter: ProjectionAspectFilter::new(
                ProjectionAspectFilterMode::All,
                ProjectionAspectScope::fields(aspect_key("summary"), [field_key("title")]),
            ),
            partition_scope: Some(Arc::from([PartitionId::main(), PartitionId(7)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId::main(), PartitionId(7)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(905),
        target_count_hint: 0,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .expect("query execution outcome");

    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![matched]
    );
}

#[test]
fn planned_query_execution_supports_aspect_filtered_relation_scans_through_reducer_path() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(&runtime, "right", PartitionId(11));
    let third = create_entity_in_partition(&runtime, "third", PartitionId(11));
    let first_relation = create_relation_in_partition(&runtime, left, right, "r1", PartitionId(7));
    let second_relation =
        create_relation_in_partition(&runtime, right, third, "r2", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "aspect-filtered-relations".to_string(),
        context_id: context,
        scope: QueryScope::AspectFilteredRelations {
            kind_id: Some(KindId(2)),
            aspect_filter: ProjectionAspectFilter::whole_aspects(
                ProjectionAspectFilterMode::All,
                [aspect_key("label")],
            ),
            partition_scope: Some(Arc::from([PartitionId(7), PartitionId(11)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7), PartitionId(11)]),
        },
        ordering: QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(904),
        target_count_hint: 0,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .expect("query execution outcome");

    assert_eq!(outcome.complexity.packet_count, 2);
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

fn create_entity_with_summary_title(
    runtime: &RelationalRuntime,
    client_key: &str,
    summary_title: &str,
) -> crate::facade::identity::EntityId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("summary-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                fields: AspectFieldPatch::new(std::collections::BTreeMap::from([
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("name"),
                            field_key("name"),
                        ),
                        string_aspect_value(client_key),
                    ),
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("title"),
                        ),
                        string_aspect_value(summary_title),
                    ),
                ])),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    changed_entities(
        &txn.commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap(),
    )[0]
}
