use super::*;
use crate::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};

#[test]
fn complete_equality_selection_uses_the_callers_budget_above_sixty_four() {
    let runtime = runtime_with_index_field_aspects();
    let mut entities = Vec::new();
    for ordinal in 0..65 {
        let created = create_entity_outcome(&runtime, &format!("selected-{ordinal}"));
        let entity = changed_entities(&created)[0];
        update_entity(&runtime, entity, "shared");
        entities.push(entity);
    }
    entities.sort();
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(76),
        name: "entity.name.caller-selection-budget".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: false,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: runtime.history().latest_commit().unwrap().commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    let snapshot = runtime.visibility_authority().snapshot();
    for limit in [64, 65] {
        let request = BoundedEntityFieldLookupRequest::new(
            snapshot.clone(),
            index.index_id,
            KindId(1),
            locator.clone(),
            string_aspect_value("shared"),
            limit,
        )
        .unwrap();
        let observed = runtime
            .index_access()
            .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Certification)
            .unwrap();
        assert_eq!(observed.candidate_entity_ids(), &entities[..limit]);
        assert_eq!(observed.examined_entry_count(), limit);
        assert_eq!(observed.overflowed(), limit == 64);
    }
}
