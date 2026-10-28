use super::*;

#[test]
fn derived_index_contract_relation_field_equals_reports_corrupt_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let source = create_entity_outcome(execution, &runtime, "source");
    let target = create_entity_outcome(execution, &runtime, "target");
    let relation = create_relation_outcome(
        execution,
        &runtime,
        changed_entities(&source)[0],
        changed_entities(&target)[0],
        "edge",
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(1),
            name: "relation.label.lookup".to_string(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity");
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: relation.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            generation.status = crate::facade::indexes::DerivedIndexPublicationStatus::BuildFailed;
        });

    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "relation-label-equals".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::RelationFieldEquals {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            value: string_aspect_value("edge"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1012),
        target_count_hint: 0,
    };
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("query plan"),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("query outcome");

    assert_eq!(
        outcome.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::CorruptIndexEntries,
        }
    );
}
