use super::*;

#[test]
fn derived_index_contract_entity_field_equals_executes_through_real_index_path_with_storage_parity()
{
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let beta = create_entity_outcome(&runtime, "beta");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.lookup".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: beta.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-equals".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1001),
        target_count_hint: 0,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("query plan");
    let storage = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("storage outcome");
    let indexed = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::CertificationParity, execution)
        .expect("request remains live")
        .expect("indexed outcome");

    assert_eq!(
        indexed.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: build.generations[0].generation_id,
        }
    );
    assert_eq!(indexed.execution.result, storage.result);
    assert_eq!(indexed.execution.result.entities.len(), 1);
    assert_eq!(
        indexed.execution.result.entities[0].entity_id,
        changed_entities(&alpha)[0]
    );
}

#[test]
fn derived_index_contract_entity_field_any_of_executes_through_real_index_path_with_storage_parity()
{
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let beta = create_entity_outcome(&runtime, "beta");
    let _gamma = create_entity_outcome(&runtime, "gamma");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(10),
        name: "entity.name.any-of".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let latest_commit_id = runtime
        .history()
        .latest_commit()
        .expect("latest commit")
        .commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: latest_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-any-of".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::EntityFieldAnyOf {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            values: Arc::from([string_aspect_value("beta"), string_aspect_value("alpha")]),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(2010),
        target_count_hint: 2,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("query plan");
    let storage = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("storage outcome");
    let indexed = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::CertificationParity, execution)
        .expect("request remains live")
        .expect("indexed outcome");

    assert_eq!(
        indexed.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: build.generations[0].generation_id,
        }
    );
    assert_eq!(indexed.execution.result, storage.result);
    assert_eq!(
        indexed
            .execution
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![changed_entities(&alpha)[0], changed_entities(&beta)[0]]
    );
}

#[test]
fn derived_index_contract_matching_definition_without_generation_reports_missing_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let outcome = create_entity_outcome(&runtime, "alpha");
    let _index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.lookup".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });

    let context = runtime
        .read_truth()
        .query_plan_context(&outcome.snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-equals".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1006),
        target_count_hint: 0,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&outcome.snapshot, packet)
        .expect("query plan");
    let result = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            plan,
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("query result");

    assert_eq!(
        result.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::MissingGeneration,
        }
    );
}

#[test]
fn derived_index_contract_explicit_corrupt_generation_reports_corrupt_entries() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.lookup".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
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

    let context = runtime
        .read_truth()
        .query_plan_context(&alpha.snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-equals".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1007),
        target_count_hint: 0,
    };
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&alpha.snapshot, packet)
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
