//! Fixture adaptation (C2.5): restores f9f15bf8c8 global generation
//! reclamation over the Memory runtime (no retired test medium).
use super::*;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexGenerationId, DerivedIndexId,
    DerivedIndexKind,
};

#[test]
fn repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root() {
    crate::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let runtime = runtime_with_test_schema();
        let first = create_entity_outcome(execution, &runtime, "global-duplicate-first");
        let index = runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "global-duplicate-name".into(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: false,
            })
            .expect("index fixture has identity capacity");
        let built = runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: first.commit.commit_id,
                branch_id: BranchId("main".into()),
                index_ids: vec![index.index_id],
            },
        );
        assert!(built.failed_indexes.is_empty());
        let original = built.generations[0].generation_id;
        let template = runtime.indexes.generation(original).unwrap();
        let mut duplicates = Vec::new();
        // Maintenance reuses an exact published commit. Seed a valid restored
        // inventory to exercise duplicate retention independently of that fast path.
        for _ in 0..16 {
            let mut generation = template.as_ref().clone();
            generation.generation_id = DerivedIndexGenerationId(
                runtime
                    .indexes
                    .next_generation_id()
                    .expect("fixture generation capacity"),
            );
            duplicates.push(generation.generation_id);
            runtime.indexes.restore_generation(generation);
        }
        drop(template);
        let selected_pinned = *duplicates.last().unwrap();
        let next = create_entity_outcome(execution, &runtime, "global-duplicate-next");
        let current = runtime
            .index_access()
            .read_latest_generation(execution, index.index_id, &BranchId("main".into()))
            .unwrap()
            .expect("publication maintains the registered global index generation")
            .generation_id;
        release_test_commit_snapshot(&runtime, &next);
        assert_eq!(
            runtime
                .run_index_generation_reclamation_after_publication(&next)
                .unwrap(),
            Some(16)
        );
        assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
        assert!(runtime.indexes.generation(original).is_none());
        assert!(duplicates[..duplicates.len() - 1]
            .iter()
            .all(|id| runtime.indexes.generation(*id).is_none()));
        assert!(runtime.indexes.generation(selected_pinned).is_some());
        assert!(runtime.indexes.generation(current).is_some());

        let checkpoint = runtime
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        let saved = checkpoint
            .derived_index_checkpoint
            .unwrap()
            .readmit(fixture_lane)
            .unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].generation_id, current);
        release_test_commit_snapshot(&runtime, &first);
        assert_eq!(
            runtime
                .run_index_generation_reclamation_after_publication(&next)
                .unwrap(),
            Some(1)
        );
        assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
        assert!(runtime.indexes.generation(selected_pinned).is_none());
        assert!(runtime.indexes.generation(current).is_some());
    })
}
