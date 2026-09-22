use super::*;
use crate::facade::indexes::{DerivedIndexEntries, DerivedIndexMaintenanceBudget};

fn cold_budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 100_000,
        maximum_cold_record_slots: 1_000,
        maximum_derived_rows: 1_000,
    }
}

#[test]
fn reused_slot_cold_and_retained_generations_match_rebuild_then_accept_local_edit() {
    let runtime = runtime_with_declared_aspect_schema_profile(
        RelationalRuntimeProfile::AiWorkflow,
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let original = create_entity_outcome(&runtime, "original");
    let old_id = changed_entities(&original)[0];
    release_test_commit_snapshot(&runtime, &original);
    let deleted = delete_entity(&runtime, old_id);
    release_test_commit_snapshot(&runtime, &deleted);
    runtime.retention().run_pass();
    let replacement = create_entity_outcome(&runtime, "replacement");
    let new_id = changed_entities(&replacement)[0];
    assert_eq!(old_id.local_slot, new_id.local_slot);
    assert!(new_id.generation.0 > old_id.generation.0);

    let register = |name| {
        runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: true,
            })
            .index_id
    };
    let cold_id = register("reuse.cold".into());
    let retained_id = register("reuse.retained".into());
    let request = |index_id, commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index_id],
    };
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let cold = runtime
        .index_authority()
        .refresh_for_basis(
            request(cold_id, replacement.commit.commit_id),
            &basis,
            None,
            cold_budget(),
        )
        .unwrap();
    let rebuilt_cold = runtime
        .index_authority()
        .build_for_basis(request(cold_id, replacement.commit.commit_id), &basis);
    assert_eq!(
        cold.generations[0].entries,
        rebuilt_cold.generations[0].entries
    );
    let DerivedIndexEntries::EntityField(entries) = &cold.generations[0].entries else {
        panic!("entity field index");
    };
    assert_eq!(entries.values().next().unwrap().get(0), Some(&new_id));

    let changed = update_entity(&runtime, new_id, "later");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(cold_id, changed.commit.commit_id),
            &basis,
            Some(&replacement.snapshot),
            cold_budget(),
        )
        .unwrap();
    assert_eq!(local.work.cold_record_slots, 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request(cold_id, changed.commit.commit_id), &basis);
    assert_eq!(local.generations[0].entries, rebuilt.generations[0].entries);
    let retained = runtime
        .index_authority()
        .reconstruct_for_commit(
            request(retained_id, replacement.commit.commit_id),
            cold_budget(),
        )
        .unwrap();
    let rebuilt_retained = runtime
        .index_authority()
        .build_for_commit(request(retained_id, replacement.commit.commit_id));
    assert_eq!(
        retained.generations[0].entries,
        rebuilt_retained.generations[0].entries
    );
    release_test_commit_snapshot(&runtime, &replacement);
    release_test_commit_snapshot(&runtime, &changed);
}
