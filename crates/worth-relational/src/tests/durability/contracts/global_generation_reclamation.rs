use super::*;
use crate::indexes::data::DerivedIndexGenerationId;

#[test]
fn repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root() {
    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "global-duplicate-first");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "global-duplicate-name".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let built = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        });
    assert!(built.failed_indexes.is_empty());
    let original = built.generations[0].generation_id;
    let template = runtime.indexes.generation(original).unwrap();
    let mut duplicates = Vec::new();
    // Maintenance reuses an exact published commit. Seed a valid restored
    // inventory to exercise duplicate retention independently of that fast path.
    for _ in 0..16 {
        let mut generation = template.as_ref().clone();
        generation.generation_id = DerivedIndexGenerationId(runtime.indexes.next_generation_id());
        duplicates.push(generation.generation_id);
        runtime.indexes.restore_generation(generation);
    }
    let selected_pinned = *duplicates.last().unwrap();
    let next = create_entity_outcome(&runtime, "global-duplicate-next");
    let next_built = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: next.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        });
    assert!(next_built.failed_indexes.is_empty());
    let current = next_built.generations[0].generation_id;
    release_test_commit_snapshot(&runtime, &next);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(16));
    assert!(runtime.indexes.generation(original).is_none());
    assert!(duplicates[..duplicates.len() - 1]
        .iter()
        .all(|id| runtime.indexes.generation(*id).is_none()));
    assert!(runtime.indexes.generation(selected_pinned).is_some());
    assert!(runtime.indexes.generation(current).is_some());

    let checkpoint = runtime.durability_authority().checkpoint().unwrap();
    let saved = checkpoint
        .derived_index_checkpoint
        .unwrap()
        .readmit()
        .unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].generation_id, current);
    release_test_commit_snapshot(&runtime, &first);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
    assert!(runtime.indexes.generation(selected_pinned).is_none());
    assert!(runtime.indexes.generation(current).is_some());
}
