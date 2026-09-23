use super::*;
use crate::facade::indexes::{
    DerivedIndexEntries, DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenialKind,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;

fn field_comparison_key(value: &str) -> AuthoritativeFieldComparisonKey {
    AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value(value))
}

fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 100_000,
        maximum_cold_record_slots: 0,
        maximum_derived_rows: 100_000,
    }
}

#[test]
fn patch_local_field_maintenance_matches_independent_rebuild_and_preserves_prior_reader() {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "maintenance.name".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
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
    let retained = runtime
        .indexes
        .generation(initial.generations[0].generation_id)
        .unwrap();
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
            budget(),
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    assert_eq!(refreshed.work.patch_records, 1);
    assert_eq!(refreshed.work.record_reads, 2);
    assert_eq!(refreshed.work.entry_edits, 2);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request(second.commit.commit_id), &basis);
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(
        refreshed.generations[0].entries,
        rebuilt.generations[0].entries
    );
    let DerivedIndexEntries::EntityField(old) = &retained.entries else {
        panic!("field index");
    };
    assert_eq!(
        old.get(&field_comparison_key("before")).unwrap().get(0),
        Some(&entity)
    );
    let DerivedIndexEntries::EntityField(new) = &refreshed.generations[0].entries else {
        panic!("field index");
    };
    assert!(new.get(&field_comparison_key("before")).is_none());
    assert_eq!(
        new.get(&field_comparison_key("after")).unwrap().get(0),
        Some(&entity)
    );
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn exhausted_refresh_and_wrong_before_root_publish_nothing() {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "first");
    let entity = changed_entities(&first)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "maintenance.denial".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    runtime
        .index_authority()
        .build_for_commit(request(first.commit.commit_id));
    let second = update_entity(&runtime, entity, "second");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let denied = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id),
            &basis,
            Some(&first.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 0,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 0,
            },
        )
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
    );
    assert!(runtime
        .indexes
        .published_generation_for_commit(
            index.index_id,
            Some(&BranchId("main".into())),
            second.commit.commit_id,
            second.commit.version_id
        )
        .is_none());
    let denied = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id),
            &basis,
            Some(&second.snapshot),
            budget(),
        )
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::BeforeRootMismatch
    );
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn ordering_join_and_relation_field_updates_match_rebuild_after_local_changes() {
    use crate::facade::indexes::{
        RelatedEntityEndpoint, RelatedEntityOrderingDirection, RelatedEntityOrderingField,
        RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint,
    };
    let runtime = runtime_with_index_field_aspects();
    let parent = create_entity(&runtime, "parent");
    let alpha = create_entity(&runtime, "alpha");
    let beta = create_entity(&runtime, "beta");
    let left = create_entity(&runtime, "left");
    let shared = create_entity(&runtime, "shared");
    let right = create_entity(&runtime, "right");
    create_relation(&runtime, parent, alpha, "owns-alpha");
    create_relation(&runtime, parent, beta, "owns-beta");
    let left_link = create_relation(&runtime, left, shared, "left-link");
    let last = create_relation_outcome(&runtime, shared, right, "right-link");
    let definitions = [
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "maintenance.ordering".into(),
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
            name: "maintenance.join".into(),
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
            branch_scoped: true,
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "maintenance.relation-field".into(),
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
    assert!(runtime
        .index_authority()
        .build_for_commit(request(last.commit.commit_id))
        .failed_indexes
        .is_empty());

    let changed = update_entity(&runtime, alpha, "zeta");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(changed.commit.commit_id),
            &basis,
            Some(&last.snapshot),
            budget(),
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    assert!(refreshed.work.adjacency_work_units > 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request(changed.commit.commit_id), &basis);
    assert_eq!(refreshed.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in refreshed.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(
            incremental.entries, cold.entries,
            "index {:?}",
            incremental.index_id
        );
    }

    let deleted = delete_relation_on_branch(&runtime, left_link, BranchId("main".into()));
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(deleted.commit.commit_id),
            &basis,
            Some(&changed.snapshot),
            budget(),
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request(deleted.commit.commit_id), &basis);
    assert_eq!(refreshed.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in refreshed.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(
            incremental.entries, cold.entries,
            "index {:?}",
            incremental.index_id
        );
    }
    release_test_commit_snapshot(&runtime, &last);
    release_test_commit_snapshot(&runtime, &changed);
    release_test_commit_snapshot(&runtime, &deleted);
}

#[test]
fn retained_commit_reconstruction_uses_its_own_root_and_finite_cold_budget() {
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
        .reconstruct_for_commit(request.clone(), budget())
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    let reconstructed = runtime
        .index_authority()
        .reconstruct_for_commit(
            request.clone(),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 1_000,
                maximum_derived_rows: 1_000,
            },
        )
        .unwrap();
    assert!(reconstructed.work.cold_record_slots > 0);
    let rebuilt = runtime.index_authority().build_for_commit(request);
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
