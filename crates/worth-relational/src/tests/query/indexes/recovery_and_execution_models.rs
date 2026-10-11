use super::*;

#[test]
fn derived_index_contract_relation_field_equals_persisted_recovery_preserves_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_index_field_aspects();
    let source = create_entity_outcome(execution, &runtime, "source");
    let target = create_entity_outcome(execution, &runtime, "target");
    let relation = create_relation_outcome(
        execution,
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
            execution,
        )
        .expect("request remains live")
        .expect("original outcome");

    let (_recovery, recovered) = checkpoint_and_recover_with(
        execution,
        &runtime,
        persisted_runtime_with_index_field_aspects,
    );
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
            execution,
        )
        .expect("request remains live")
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(execution, &runtime, "alpha");
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
            execution,
        )
        .expect("request remains live")
        .expect("original outcome");

    let (_recovery, recovered) = checkpoint_and_recover_with(
        execution,
        &runtime,
        persisted_runtime_with_index_field_aspects,
    );
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
            execution,
        )
        .expect("request remains live")
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    fn build_runtime(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        leased: bool,
    ) -> (
        crate::facade::runtime::RelationalRuntime,
        crate::facade::snapshots::SnapshotHandle,
    ) {
        let runtime = runtime_with_test_schema();
        let alpha = create_entity_outcome(execution, &runtime, "alpha");
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
            runtime.index_authority().build_for_commit(
                &worth_execution::ExecutionRequest::leased(&test_execution_lease()),
                request,
            )
        } else {
            runtime
                .index_authority()
                .build_for_commit(execution, request)
        };
        assert!(build.failed_indexes.is_empty());
        (runtime, alpha.snapshot.clone())
    }

    let (serial_runtime, serial_snapshot) = build_runtime(execution, false);
    let (staged_runtime, staged_snapshot) = build_runtime(execution, true);

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
            execution,
        )
        .expect("request remains live")
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
            execution,
        )
        .expect("request remains live")
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

#[path = "recovery_and_execution_models/staged_generation.rs"]
mod staged_generation;
