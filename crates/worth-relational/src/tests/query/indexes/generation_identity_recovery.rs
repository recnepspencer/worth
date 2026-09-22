use super::*;
use crate::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};

#[test]
fn restored_catalog_allocates_fresh_identities_and_preserves_old_observations() {
    let runtime = persisted_runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "before-recovery");
    let entity = changed_entities(&created)[0];
    let field = aspect_field_locator(aspect_key("name"), field_key("name"));
    let mut definition = DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.before-recovery".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: field.clone(),
        },
        branch_scoped: true,
    };
    // More than one definition/generation exposes reset-to-zero and reset-to-one.
    let first = runtime.index_authority().register(definition.clone());
    definition.name = "entity.name.second".into();
    let second = runtime.index_authority().register(definition.clone());
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: created.commit.branch_id.clone(),
            index_ids: vec![first.index_id, second.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    let (_, recovered) =
        checkpoint_and_recover_with(&runtime, persisted_runtime_with_index_field_aspects);
    let old_snapshot = recovered.visibility_authority().snapshot();
    definition.name = "entity.name.after-recovery".into();
    let third = recovered.index_authority().register(definition.clone());
    let fourth = recovered.index_authority().register(definition);
    assert!(third.index_id > second.index_id);
    assert!(fourth.index_id > third.index_id);
    let committed = update_entity(&recovered, entity, "after-recovery");
    let after = recovered
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: committed.commit.branch_id.clone(),
            index_ids: vec![
                first.index_id,
                second.index_id,
                third.index_id,
                fourth.index_id,
            ],
        });
    assert!(after.failed_indexes.is_empty());
    let previous_max = build
        .generations
        .iter()
        .map(|generation| generation.generation_id)
        .max()
        .unwrap();
    assert!(after
        .generations
        .iter()
        .all(|generation| generation.generation_id > previous_max));
    for (snapshot, value, expected) in [
        (
            old_snapshot,
            "before-recovery",
            build.generations[0].generation_id,
        ),
        (
            committed.snapshot.clone(),
            "after-recovery",
            after.generations[0].generation_id,
        ),
    ] {
        let before = recovered.index_access().generation_selection_counters();
        let observed = recovered
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldLookupRequest::new(
                    snapshot,
                    first.index_id,
                    KindId(1),
                    field.clone(),
                    string_aspect_value(value),
                    1,
                )
                .unwrap(),
                BoundedIndexParityMode::Certification,
            )
            .unwrap();
        assert_eq!(observed.generation_id(), expected);
        assert_eq!(observed.candidate_entity_ids(), &[entity]);
        let counters = recovered.index_access().generation_selection_counters();
        assert_eq!(
            counters.generation_payload_reads - before.generation_payload_reads,
            1
        );
        assert_eq!(
            counters.history_inventory_entries - before.history_inventory_entries,
            0
        );
    }
}
