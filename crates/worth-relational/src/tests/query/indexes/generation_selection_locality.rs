use super::*;
use crate::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};

#[test]
fn bounded_generation_selection_does_not_enumerate_retained_builds() {
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "selected");
    let entity = changed_entities(&created)[0];
    let field = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.generation-locality".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: field.clone(),
        },
        branch_scoped: true,
    });
    let receipt = runtime
        .history()
        .immutable_commit_receipt(created.commit.commit_id)
        .unwrap();
    let snapshot = runtime.visibility_authority().snapshot();
    // Repeated cold rebuilds are real native publications. The measured action
    // is the unchanged bounded consumer, not index construction or graph history.
    for retained in 1..=1_000 {
        let build = runtime
            .index_authority()
            .build_for_commit(DerivedIndexBuildRequest {
                source_commit_id: receipt.commit_id,
                branch_id: receipt.branch_id.clone(),
                index_ids: vec![index.index_id],
            });
        assert!(build.failed_indexes.is_empty());
        if ![100, 1_000].contains(&retained) {
            continue;
        }
        let before = runtime.index_access().generation_selection_counters();
        let selected = runtime
            .index_access()
            .published_generation_for_commit(index.index_id, &receipt)
            .unwrap();
        let outcome = runtime
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldLookupRequest::new(
                    snapshot.clone(),
                    index.index_id,
                    KindId(1),
                    field.clone(),
                    string_aspect_value("selected"),
                    1,
                )
                .unwrap(),
                BoundedIndexParityMode::Production,
            )
            .unwrap();
        let after = runtime.index_access().generation_selection_counters();
        assert_eq!(outcome.generation_id(), build.generations[0].generation_id);
        assert_eq!(selected.generation_id, outcome.generation_id());
        assert_eq!(outcome.candidate_entity_ids(), &[entity]);
        assert_eq!(
            after.generation_payload_reads - before.generation_payload_reads,
            2
        );
        assert_eq!(
            after.history_inventory_entries - before.history_inventory_entries,
            0
        );
    }
}
