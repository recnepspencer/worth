use super::*;

#[test]
fn derived_index_contract_staged_parallel_generation_matches_serial_reference() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

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

    let serial = serial_runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: serial_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids,
        },
    );
    let staged = staged_runtime
        .index_authority()
        .build_for_commit_with_lease(
            &worth_execution::ExecutionRequest::leased(&test_execution_lease()),
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
