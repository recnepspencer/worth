use super::*;

#[test]
fn derived_index_contract_relation_field_equals_persisted_recovery_preserves_parity() {
    let runtime = persisted_runtime_with_index_field_aspects();
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
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: relation.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        });
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1013),
        target_count_hint: 0,
    };
    let original = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet.clone())
                .expect("original plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("original outcome");

    let (_recovery, recovered) =
        checkpoint_and_recover_with(&runtime, persisted_runtime_with_index_field_aspects);
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let recovered_context = recovered
        .read_truth()
        .query_plan_context(&recovered_snapshot)
        .expect("recovered context");
    let mut recovered_packet = packet;
    recovered_packet.context_id = recovered_context;
    let recovered_outcome = recovered
        .index_access()
        .execute_query_plan_with_index_parity(
            recovered
                .read_truth()
                .plan_query_packet(&recovered_snapshot, recovered_packet)
                .expect("recovered plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("recovered outcome");

    assert_eq!(original.access_path, recovered_outcome.access_path);
    assert_eq!(
        original.execution.result,
        recovered_outcome.execution.result
    );
    assert_eq!(
        original.execution.result.reduction_digest,
        recovered_outcome.execution.result.reduction_digest
    );
}

#[test]
fn derived_index_contract_persisted_recovery_preserves_entity_field_equals_parity() {
    let runtime = persisted_runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.lookup".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());

    let snapshot = alpha.snapshot.clone();
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1008),
        target_count_hint: 0,
    };
    let original = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet.clone())
                .expect("original plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("original outcome");

    let (_recovery, recovered) =
        checkpoint_and_recover_with(&runtime, persisted_runtime_with_index_field_aspects);
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let recovered_context = recovered
        .read_truth()
        .query_plan_context(&recovered_snapshot)
        .expect("recovered context");
    let mut recovered_packet = packet;
    recovered_packet.context_id = recovered_context;
    let recovered_outcome = recovered
        .index_access()
        .execute_query_plan_with_index_parity(
            recovered
                .read_truth()
                .plan_query_packet(&recovered_snapshot, recovered_packet)
                .expect("recovered plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("recovered outcome");

    assert_eq!(original.access_path, recovered_outcome.access_path);
    assert_eq!(
        original.execution.result,
        recovered_outcome.execution.result
    );
    assert_eq!(
        original.execution.result.reduction_digest,
        recovered_outcome.execution.result.reduction_digest
    );
}

#[test]
fn derived_index_contract_entity_field_equals_is_stable_with_leased_index_build() {
    fn build_runtime(
        leased: bool,
    ) -> (
        crate::facade::runtime::RelationalRuntime,
        crate::facade::snapshots::SnapshotHandle,
    ) {
        let runtime = runtime_with_test_schema();
        let alpha = create_entity_outcome(&runtime, "alpha");
        let index = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.name.lookup".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        });
        let request = DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        };
        let build = if leased {
            runtime
                .index_authority()
                .build_for_commit_with_lease(request, &test_execution_lease())
        } else {
            runtime.index_authority().build_for_commit(request)
        };
        assert!(build.failed_indexes.is_empty());
        (runtime, alpha.snapshot.clone())
    }

    let (serial_runtime, serial_snapshot) = build_runtime(false);
    let (staged_runtime, staged_snapshot) = build_runtime(true);

    let serial_context = serial_runtime
        .read_truth()
        .query_plan_context(&serial_snapshot)
        .expect("serial context");
    let staged_context = staged_runtime
        .read_truth()
        .query_plan_context(&staged_snapshot)
        .expect("staged context");

    let serial_outcome = serial_runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            serial_runtime
                .read_truth()
                .plan_query_packet(
                    &serial_snapshot,
                    crate::facade::query::PlannedQueryPacket {
                        label: "entity-name-equals".to_string(),
                        context_id: serial_context,
                        scope: crate::facade::query::QueryScope::EntityFieldEquals {
                            field_locator: aspect_field_locator(
                                aspect_key("name"),
                                field_key("name"),
                            ),
                            value: string_aspect_value("alpha"),
                            partition_scope: None,
                        },
                        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
                        ordering:
                            crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
                        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
                        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
                        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
                        plan_key: crate::facade::query::DeterministicQueryPlanKey(1009),
                        target_count_hint: 0,
                    },
                )
                .expect("serial plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("serial outcome");

    let staged_outcome = staged_runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            staged_runtime
                .read_truth()
                .plan_query_packet(
                    &staged_snapshot,
                    crate::facade::query::PlannedQueryPacket {
                        label: "entity-name-equals".to_string(),
                        context_id: staged_context,
                        scope: crate::facade::query::QueryScope::EntityFieldEquals {
                            field_locator: aspect_field_locator(
                                aspect_key("name"),
                                field_key("name"),
                            ),
                            value: string_aspect_value("alpha"),
                            partition_scope: None,
                        },
                        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
                        ordering:
                            crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
                        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
                        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
                        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
                        plan_key: crate::facade::query::DeterministicQueryPlanKey(1010),
                        target_count_hint: 0,
                    },
                )
                .expect("staged plan"),
            IndexParityMode::CertificationParity,
        )
        .expect("staged outcome");

    assert_eq!(serial_outcome.access_path, staged_outcome.access_path);
    assert_eq!(
        serial_outcome.execution.result,
        staged_outcome.execution.result
    );
    assert_eq!(
        serial_outcome.execution.result.reduction_digest,
        staged_outcome.execution.result.reduction_digest
    );
}

#[test]
fn derived_index_contract_staged_parallel_generation_matches_serial_reference() {
    fn build_runtime() -> (
        crate::facade::runtime::RelationalRuntime,
        crate::facade::history::CommitId,
        Vec<crate::facade::indexes::DerivedIndexId>,
    ) {
        let runtime = runtime_with_test_schema();
        let commit = create_entity_outcome(&runtime, "main-a");
        let name_index = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.name".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        });
        let missing_index = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.missing".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("missing"), field_key("missing")),
            },
            branch_scoped: false,
        });

        (
            runtime,
            commit.commit.commit_id,
            vec![name_index.index_id, missing_index.index_id],
        )
    }

    let (serial_runtime, serial_commit_id, index_ids) = build_runtime();
    let (staged_runtime, staged_commit_id, staged_index_ids) = build_runtime();

    serial_runtime.performance_access().reset_counters();
    staged_runtime.performance_access().reset_counters();

    let serial = serial_runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: serial_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids,
        });
    let staged = staged_runtime
        .index_authority()
        .build_for_commit_with_lease(
            DerivedIndexBuildRequest {
                source_commit_id: staged_commit_id,
                branch_id: BranchId("main".to_string()),
                index_ids: staged_index_ids,
            },
            &test_execution_lease(),
        );

    let staged_counters = staged_runtime.performance_access().counters();

    assert_eq!(staged, serial);
    assert_eq!(staged_counters.preparation_packet_count, 2);
    assert_eq!(staged_counters.preparation_parallel_legal_count, 1);
    assert_eq!(staged_counters.preparation_parallel_profitable_count, 1);
    assert_eq!(
        staged_counters.preparation_staged_parallel_strategy_count,
        1
    );
    assert_eq!(staged_counters.preparation_serial_strategy_count, 0);
}
