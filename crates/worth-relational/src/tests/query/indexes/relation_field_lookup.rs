use super::*;

#[test]
fn derived_index_contract_relation_field_equals_executes_through_real_index_path_with_storage_parity(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let source = create_entity_outcome(&runtime, "source");
    let source_id = changed_entities(&source)[0];
    let target = create_entity_outcome(&runtime, "target");
    let target_id = changed_entities(&target)[0];
    let relation = create_relation_outcome(&runtime, source_id, target_id, "edge");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(1),
        name: "relation.label.lookup".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: relation.commit.commit_id,
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1011),
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
    assert_eq!(indexed.execution.result.relations.len(), 1);
    assert_eq!(
        indexed.execution.result.relations[0].relation_id,
        changed_relations(&relation)[0]
    );
}

#[test]
fn derived_index_contract_relation_field_equals_reports_corrupt_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let source = create_entity_outcome(&runtime, "source");
    let target = create_entity_outcome(&runtime, "target");
    let relation = create_relation_outcome(
        &runtime,
        changed_entities(&source)[0],
        changed_entities(&target)[0],
        "edge",
    );
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(1),
        name: "relation.label.lookup".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: false,
    });
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

    let snapshot = runtime.visibility_authority().snapshot();
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

#[test]
fn derived_index_contract_relation_field_any_of_executes_through_real_index_path_with_storage_parity(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let source = create_entity_outcome(&runtime, "source");
    let target = create_entity_outcome(&runtime, "target");
    let third = create_entity_outcome(&runtime, "third");
    let edge = create_relation_outcome(
        &runtime,
        changed_entities(&source)[0],
        changed_entities(&target)[0],
        "edge",
    );
    let arc = create_relation_outcome(
        &runtime,
        changed_entities(&target)[0],
        changed_entities(&third)[0],
        "arc",
    );
    let _other = create_relation_outcome(
        &runtime,
        changed_entities(&third)[0],
        changed_entities(&source)[0],
        "other",
    );
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(11),
        name: "relation.label.any-of".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
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
        label: "relation-label-any-of".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::RelationFieldAnyOf {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            values: Arc::from([string_aspect_value("arc"), string_aspect_value("edge")]),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(2011),
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
            .relations
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        vec![changed_relations(&edge)[0], changed_relations(&arc)[0]]
    );
}

#[test]
fn derived_index_contract_relation_field_equals_partition_scope_keeps_bounded_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let left_source = create_entity_in_partition(&runtime, "left-source", PartitionId(7));
    let left_target = create_entity_in_partition(&runtime, "left-target", PartitionId(7));
    let right_source = create_entity_in_partition(&runtime, "right-source", PartitionId(11));
    let right_target = create_entity_in_partition(&runtime, "right-target", PartitionId(11));
    let left_relation =
        create_relation_in_partition(&runtime, left_source, left_target, "edge", PartitionId(7));
    let _right_relation = create_relation_in_partition(
        &runtime,
        right_source,
        right_target,
        "edge",
        PartitionId(11),
    );
    let commit_id = runtime
        .history()
        .latest_commit()
        .expect("latest commit")
        .commit_id;
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(1),
        name: "relation.label.partitioned".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: commit_id,
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
        label: "relation-label-left-only".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::RelationFieldEquals {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            value: string_aspect_value("edge"),
            partition_scope: Some(std::sync::Arc::from([PartitionId(7)])),
        },
        locality: crate::facade::query::QueryLocalityClass::PartitionBounded {
            partitions: std::sync::Arc::from([PartitionId(7)]),
        },
        ordering: crate::facade::query::QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1015),
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

    assert_eq!(indexed.execution.result, storage.result);
    assert_eq!(indexed.execution.result.relations.len(), 1);
    assert_eq!(
        indexed.execution.result.relations[0].relation_id,
        left_relation
    );
    assert_eq!(
        indexed.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: build.generations[0].generation_id,
        }
    );
}
