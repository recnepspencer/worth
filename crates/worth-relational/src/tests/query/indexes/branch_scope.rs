use super::*;

#[test]
fn derived_index_contract_success_branch_scoped_build_keeps_storage_read() {
    let runtime = runtime_with_index_field_aspects();
    let main_outcome = create_entity_outcome(&runtime, "main-a");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_outcome =
        create_entity_outcome_on_branch(&runtime, "feature-a", BranchId("feature".to_string()));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let feature_build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: feature_outcome.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            index_ids: vec![index.index_id],
        });
    let context = runtime
        .read_truth()
        .query_plan_context(&main_outcome.snapshot)
        .expect("query plan context");
    let mut planned = crate::facade::query::PlannedQueryPacket::explicit_targets(
        "entities",
        context,
        vec![RecordRef::Entity(changed_entities(&main_outcome)[0])],
    );
    planned.access_contract = QueryAccessContract::DerivedIndexWithStorageParity;
    let plan = runtime
        .read_truth()
        .plan_query_packet(&main_outcome.snapshot, planned)
        .expect("query plan");
    let storage_read = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::ProductionAdmissibility)
        .unwrap();

    assert!(feature_build.failed_indexes.is_empty());
    assert_eq!(
        runtime
            .index_access()
            .latest_generation(index.index_id, &BranchId("feature".to_string()))
            .unwrap()
            .source_branch_id,
        BranchId("feature".to_string())
    );
    assert_eq!(storage_read.execution.result.entities.len(), 1);
    assert_eq!(
        storage_read.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::MissingGeneration,
        }
    );
}

#[test]
fn derived_index_contract_unscoped_generation_is_rejected_for_unsupported_scope() {
    let runtime = runtime_with_index_field_aspects();
    let main_outcome = create_entity_outcome(&runtime, "main-a");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_outcome =
        create_entity_outcome_on_branch(&runtime, "feature-a", BranchId("feature".to_string()));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.global".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: feature_outcome.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            index_ids: vec![index.index_id],
        });
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let mut planned = crate::facade::query::PlannedQueryPacket::explicit_targets(
        "entities",
        context,
        vec![RecordRef::Entity(changed_entities(&main_outcome)[0])],
    );
    planned.access_contract = QueryAccessContract::DerivedIndexWithStorageParity;
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, planned)
        .expect("query plan");
    let storage_read = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::ProductionAdmissibility)
        .unwrap();

    assert!(build.failed_indexes.is_empty());
    assert_eq!(
        runtime
            .index_access()
            .latest_generation(index.index_id, &BranchId("main".to_string()))
            .unwrap()
            .source_branch_id,
        BranchId("feature".to_string())
    );
    assert_eq!(
        storage_read.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::UnsupportedScope,
        }
    );
}

#[test]
fn derived_index_contract_relation_field_equals_branch_scoped_generation_reports_unsupported_branch(
) {
    let runtime = runtime_with_index_field_aspects();
    let main_source = create_entity_outcome(&runtime, "main-source");
    let main_target = create_entity_outcome(&runtime, "main-target");
    let main_relation = create_relation_outcome(
        &runtime,
        changed_entities(&main_source)[0],
        changed_entities(&main_target)[0],
        "edge",
    );
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_source = create_entity_outcome_on_branch(
        &runtime,
        "feature-source",
        BranchId("feature".to_string()),
    );
    let feature_target = create_entity_outcome_on_branch(
        &runtime,
        "feature-target",
        BranchId("feature".to_string()),
    );
    let mut feature_txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    feature_txn
        .push_batch(
            WorkerIntentBatch::new("feature-relation").push(MutationIntent::Create(
                CreateIntent::Relation(crate::transactions::data::RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("edge"),
                    source: crate::transactions::data::EntityReference::Existing(
                        changed_entities(&feature_source)[0],
                    ),
                    target: crate::transactions::data::EntityReference::Existing(
                        changed_entities(&feature_target)[0],
                    ),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let feature_relation = feature_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("feature relation");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(1),
        name: "relation.label.branch".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: true,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: feature_relation.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());

    let context = runtime
        .read_truth()
        .query_plan_context(&main_relation.snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "relation-branch-mismatch".to_string(),
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1014),
        target_count_hint: 0,
    };
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&main_relation.snapshot, packet)
                .expect("query plan"),
            IndexParityMode::ProductionAdmissibility,
        )
        .expect("query outcome");

    assert_eq!(
        outcome.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::UnsupportedBranch,
        }
    );
}

#[test]
fn derived_index_contract_branch_scoped_generation_reports_unsupported_branch() {
    let runtime = runtime_with_index_field_aspects();
    let main_outcome = create_entity_outcome(&runtime, "main-a");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_outcome =
        create_entity_outcome_on_branch(&runtime, "feature-a", BranchId("feature".to_string()));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.branch".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: feature_outcome.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());

    let context = runtime
        .read_truth()
        .query_plan_context(&main_outcome.snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "branch-mismatch".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("main-a"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1002),
        target_count_hint: 0,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&main_outcome.snapshot, packet)
        .expect("query plan");
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::ProductionAdmissibility)
        .expect("storage-read outcome");

    assert_eq!(
        outcome.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::UnsupportedBranch,
        }
    );
}

#[test]
fn derived_index_contract_prefers_older_supported_generation_over_newer_unsupported_one() {
    let runtime = runtime_with_index_field_aspects();
    let main_alpha = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.lookup".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let main_build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: main_alpha.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        });
    assert!(main_build.failed_indexes.is_empty());

    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_alpha =
        create_entity_outcome_on_branch(&runtime, "alpha", BranchId("feature".to_string()));
    let feature_build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: feature_alpha.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            index_ids: vec![index.index_id],
        });
    assert!(feature_build.failed_indexes.is_empty());
    assert!(feature_build.generations[0].generation_id > main_build.generations[0].generation_id);

    let snapshot = main_alpha.snapshot.clone();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-equals-main".to_string(),
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1005),
        target_count_hint: 0,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("query plan");
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::ProductionAdmissibility)
        .expect("query outcome");

    assert_eq!(
        outcome.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: main_build.generations[0].generation_id,
        }
    );
}
