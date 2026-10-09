use super::*;
use crate::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind,
    BoundedIndexParityMode,
};

#[test]
fn prepared_field_selection_keeps_its_root_without_reselecting_the_generation() {
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "pinned-name");
    let entity = changed_entities(&created)[0];
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.prepared".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: true,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let before = runtime.index_access().generation_selection_counters();
    let prepared = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator)
        .unwrap();
    let foreign = runtime_with_index_field_aspects();
    let denied = foreign
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator)
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
        "foreign view fails before consulting the other runtime's index catalog"
    );
    assert_eq!(
        runtime
            .index_access()
            .generation_selection_counters()
            .generation_payload_reads
            - before.generation_payload_reads,
        1
    );
    drop(view);
    runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .unwrap();
    update_entity(&runtime, entity, "new-name");
    let before = runtime.index_access().generation_selection_counters();
    let handles = runtime.visibility.snapshot_handle_registry_cost_counters();
    for (name, expected) in [
        ("pinned-name", vec![entity]),
        ("new-name", vec![]),
        ("absent", vec![]),
    ] {
        let selected = runtime
            .index_access()
            .execute_prepared_entity_field_lookup(
                &prepared,
                &string_aspect_value(name),
                1024,
                BoundedIndexParityMode::Production,
                || Ok::<_, ()>(()),
            )
            .unwrap();
        assert_eq!(selected.candidate_entity_ids(), expected);
        assert_eq!(selected.generation_id(), build.generations[0].generation_id);
        assert!(!selected.overflowed());
    }
    assert_eq!(
        runtime.index_access().generation_selection_counters(),
        before
    );
    assert_eq!(
        runtime.visibility.snapshot_handle_registry_cost_counters(),
        handles
    );
    let certified = runtime
        .index_access()
        .execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("pinned-name"),
            1024,
            BoundedIndexParityMode::Certification,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(certified.candidate_entity_ids(), &[entity]);

    let mut contacted = false;
    let denied = foreign
        .index_access()
        .execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("pinned-name"),
            1024,
            BoundedIndexParityMode::Production,
            || {
                contacted = true;
                Ok::<_, ()>(())
            },
        )
        .unwrap_err();
    assert!(matches!(denied, Stop::ExactBasisRequired));
    assert!(!contacted, "foreign affinity fails before caller work");
    let stopped = runtime
        .index_access()
        .execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("pinned-name"),
            1024,
            BoundedIndexParityMode::Production,
            || Err("cancelled"),
        )
        .unwrap_err();
    assert!(matches!(stopped, Stop::Admission("cancelled")));
}
