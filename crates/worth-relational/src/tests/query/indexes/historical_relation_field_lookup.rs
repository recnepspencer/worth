use super::*;

#[test]
fn relation_field_index_executes_retained_history_after_relation_deletion() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let source = changed_entities(&create_entity_outcome(&runtime, "source"))[0];
    let target = changed_entities(&create_entity_outcome(&runtime, "target"))[0];
    let historical = create_relation_outcome(&runtime, source, target, "historical");
    let relation_id = changed_relations(&historical)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "relation.label.historical".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: false,
    });

    delete_relation_on_branch(&runtime, relation_id, BranchId("main".to_string()));
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: historical.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let context = runtime
        .read_truth()
        .query_plan_context(&historical.snapshot)
        .unwrap();
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "historical-relation-label".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::RelationFieldEquals {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            value: string_aspect_value("historical"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1_013),
        target_count_hint: 1,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&historical.snapshot, packet)
        .unwrap();
    let storage = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .unwrap();
    let indexed = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::CertificationParity, execution)
        .expect("request remains live")
        .unwrap();

    assert_eq!(indexed.execution.result, storage.result);
    assert_eq!(indexed.execution.result.relations.len(), 1);
    assert_eq!(
        indexed.execution.result.relations[0].relation_id,
        relation_id
    );
    assert_eq!(
        indexed.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: build.generations[0].generation_id,
        }
    );
}
