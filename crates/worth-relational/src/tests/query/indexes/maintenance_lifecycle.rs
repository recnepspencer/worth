use super::*;
use crate::facade::indexes::{
    DerivedIndexMaintenanceBudget, RelatedEntityEndpoint, RelatedEntityOrderingDirection,
    RelatedEntityOrderingField, RelationJoinDefinition, RelationJoinLeg,
    RelationJoinSharedEndpoint,
};

#[test]
fn retained_dangling_endpoint_deletion_matches_full_index_builds() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let parent = create_entity(&runtime, "parent");
    let left = create_entity(&runtime, "left");
    let shared = create_entity(&runtime, "shared");
    let right = create_entity(&runtime, "right");
    create_relation(&runtime, parent, shared, "parent-shared");
    create_relation(&runtime, left, shared, "left-shared");
    let before = create_relation_outcome(&runtime, shared, right, "shared-right");
    let definitions = [
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.ordering".into(),
            kind: DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: KindId(2),
                parent_endpoint: RelatedEntityEndpoint::SourceParent,
                child_kind: KindId(1),
                ordering: vec![RelatedEntityOrderingField::new(
                    aspect_field_locator(aspect_key("name"), field_key("name")),
                    RelatedEntityOrderingDirection::Ascending,
                )],
            },
            branch_scoped: true,
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.join".into(),
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
            branch_scoped: true,
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.relation-field".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: true,
        },
    ];
    let ids = definitions
        .into_iter()
        .map(|definition| runtime.index_authority().register(definition).index_id)
        .collect::<Vec<_>>();
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: ids.clone(),
    };
    let built = runtime
        .index_authority()
        .build_for_commit(request(before.commit.commit_id));
    assert!(built.failed_indexes.is_empty());
    let deleted = delete_entity(&runtime, shared);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(deleted.commit.commit_id),
            &basis,
            Some(&before.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 100_000,
            },
        )
        .unwrap();
    assert_eq!(local.work.cold_record_slots, 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request(deleted.commit.commit_id), &basis);
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(local.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in local.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(incremental.entries, cold.entries);
    }
    let child_changed = update_entity(&runtime, right, "right-changed");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(child_changed.commit.commit_id),
            &basis,
            Some(&deleted.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 100_000,
            },
        )
        .unwrap();
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request(child_changed.commit.commit_id), &basis);
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(local.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in local.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(incremental.entries, cold.entries);
    }
    release_test_commit_snapshot(&runtime, &before);
    release_test_commit_snapshot(&runtime, &deleted);
    release_test_commit_snapshot(&runtime, &child_changed);
}
