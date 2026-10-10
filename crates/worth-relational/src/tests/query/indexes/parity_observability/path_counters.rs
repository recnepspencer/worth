use super::*;

#[test]
fn derived_index_contract_index_counters_track_attempts_paths_and_rejections() {
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
    runtime.performance_access().reset_counters();

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let success_packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-equals".to_string(),
        context_id: context.clone(),
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1003),
        target_count_hint: 0,
    };
    runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, success_packet)
                .expect("success plan"),
            IndexParityMode::CertificationParity,
            execution,
        )
        .expect("request remains live")
        .expect("success outcome");

    let rejection_packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-equals-rejected".to_string(),
        context_id: context,
        scope: crate::facade::query::QueryScope::EntityKindScan {
            kind_id: KindId(1),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: crate::facade::query::DeterministicQueryPlanKey(1004),
        target_count_hint: 0,
    };
    runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, rejection_packet)
                .expect("rejection plan"),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("rejection outcome");

    let counters = runtime.performance_access().counters();
    assert_eq!(counters.query_index_attempt_count, 2);
    assert_eq!(counters.query_index_path_count, 1);
    assert_eq!(counters.query_index_rejection_count, 1);
    assert_eq!(counters.query_index_parity_verification_count, 1);
}
