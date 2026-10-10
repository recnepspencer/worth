use super::*;

#[test]
fn retained_commit_reconstruction_uses_its_own_root_and_finite_cold_budget() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "historical");
    let entity = changed_entities(&first)[0];
    let second = update_entity(&runtime, entity, "current");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "maintenance.retained".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let request = DerivedIndexBuildRequest {
        source_commit_id: first.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    let denied = runtime
        .index_authority()
        .reconstruct_for_commit(execution, request.clone(), budget())
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    let reconstructed = runtime
        .index_authority()
        .reconstruct_for_commit(
            execution,
            request.clone(),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 1_000,
                maximum_derived_rows: 1_000,
            },
        )
        .unwrap();
    assert!(reconstructed.work.cold_record_slots > 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_commit(execution, request);
    assert_eq!(
        reconstructed.generations[0].entries,
        rebuilt.generations[0].entries
    );
    let current_request = DerivedIndexBuildRequest {
        source_commit_id: second.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let cold_current = runtime
        .index_authority()
        .refresh_for_basis(
            current_request,
            &basis,
            None,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 1_000,
                maximum_derived_rows: 1_000,
            },
            execution,
        )
        .unwrap();
    assert_eq!(cold_current.work.entry_edits, 1);
    let third = update_entity(&runtime, entity, "later");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id: third.commit.commit_id,
                branch_id: BranchId("main".into()),
                index_ids: vec![index.index_id],
            },
            &basis,
            Some(&second.snapshot),
            budget(),
            execution,
        )
        .unwrap();
    assert_eq!(local.work.cold_record_slots, 0);
    let DerivedIndexEntries::EntityField(entries) = &local.generations[0].entries else {
        panic!("field index");
    };
    assert_eq!(
        entries.get(&field_comparison_key("later")).unwrap().get(0),
        Some(&entity)
    );
    assert!(entries.get(&field_comparison_key("current")).is_none());
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
    release_test_commit_snapshot(&runtime, &third);
}
