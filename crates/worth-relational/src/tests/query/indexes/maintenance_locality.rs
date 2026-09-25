use super::*;
use crate::facade::indexes::{DerivedIndexEntries, DerivedIndexMaintenanceBudget};

#[test]
fn unrelated_field_indexes_do_not_reread_changed_records() {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_ids = ["name", "unrelated-a", "unrelated-b", "unrelated-c"]
        .into_iter()
        .map(|name| {
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: format!("maintenance.{name}"),
                    kind: DerivedIndexKind::EntityField {
                        field_locator: aspect_field_locator(aspect_key(name), field_key(name)),
                    },
                    branch_scoped: true,
                })
                .index_id
        })
        .collect::<Vec<_>>();
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: index_ids.clone(),
    };
    let initial = runtime
        .index_authority()
        .build_for_commit(request(first.commit.commit_id));
    assert!(initial.failed_indexes.is_empty());
    let second = update_entity(&runtime, entity, "after");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id),
            &basis,
            Some(&first.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 0,
            },
        )
        .unwrap();
    assert_eq!(refreshed.work.patch_records, 1);
    assert_eq!(refreshed.work.record_reads, 2);
    assert_eq!(refreshed.work.entry_edits, 2);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn relation_field_deletion_removes_the_old_entry_with_one_read_per_basis() {
    let runtime = runtime_with_index_field_aspects();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let first = create_relation_outcome(&runtime, source, target, "link");
    let relation = changed_relations(&first)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "maintenance.deleted-relation".into(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: true,
    });
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    let initial = runtime
        .index_authority()
        .build_for_commit(request(first.commit.commit_id));
    assert!(initial.failed_indexes.is_empty());
    let second = delete_relation_on_branch(&runtime, relation, BranchId("main".into()));
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id),
            &basis,
            Some(&first.snapshot),
            budget(),
        )
        .unwrap();
    assert_eq!(refreshed.work.record_reads, 2);
    assert_eq!(refreshed.work.entry_edits, 1);
    let DerivedIndexEntries::RelationField(entries) = &refreshed.generations[0].entries else {
        panic!("relation field index")
    };
    assert!(entries.is_empty());
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn mixed_patch_and_cold_field_generations_use_their_own_changes() {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let register = |name: &str, field: &str| {
        runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: format!("maintenance.{name}"),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key(field), field_key(field)),
            },
            branch_scoped: true,
        })
    };
    let name = register("name", "name");
    let request = |commit_id, index_ids| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids,
    };
    let initial = runtime
        .index_authority()
        .build_for_commit(request(first.commit.commit_id, vec![name.index_id]));
    assert!(initial.failed_indexes.is_empty());
    let cold = register("name-cold", "name");
    let second = update_entity(&runtime, entity, "after");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id, vec![name.index_id, cold.index_id]),
            &basis,
            Some(&first.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 1_000,
                maximum_derived_rows: 0,
            },
        )
        .unwrap();
    assert!(refreshed.work.cold_record_slots > 0);
    assert_eq!(refreshed.work.record_reads, 4);
    assert_eq!(refreshed.work.entry_edits, 3);
    let name_generation = refreshed
        .generations
        .iter()
        .find(|generation| generation.index_id == name.index_id)
        .unwrap();
    let DerivedIndexEntries::EntityField(name_entries) = &name_generation.entries else {
        panic!("name field index")
    };
    assert_eq!(name_entries.len(), 1);
    let cold_generation = refreshed
        .generations
        .iter()
        .find(|generation| generation.index_id == cold.index_id)
        .unwrap();
    let DerivedIndexEntries::EntityField(cold_entries) = &cold_generation.entries else {
        panic!("cold field index")
    };
    assert_eq!(cold_entries, name_entries);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 100_000,
        maximum_cold_record_slots: 0,
        maximum_derived_rows: 0,
    }
}
