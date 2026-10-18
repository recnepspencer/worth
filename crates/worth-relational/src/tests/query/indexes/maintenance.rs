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
        maximum_work_units: Some(100_000),
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: Some(100_000),
    }
}

#[test]
fn patch_local_field_maintenance_matches_independent_rebuild_and_preserves_prior_reader() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
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
        .build_for_commit(execution, request(first.commit.commit_id));
    assert!(initial.failed_indexes.is_empty());
    let retained = runtime
        .indexes
        .generation(initial.generations[0].generation_id)
        .unwrap();
    let second = update_entity(execution, &runtime, entity, "after");
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
            execution,
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    assert_eq!(refreshed.work.patch_records, 1);
    assert_eq!(refreshed.work.record_reads, 2);
    assert_eq!(refreshed.work.entry_edits, 2);
    let rebuilt = runtime.index_authority().build_for_basis(
        execution,
        request(second.commit.commit_id),
        &basis,
    );
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "first");
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
        .build_for_commit(execution, request(first.commit.commit_id));
    let second = update_entity(execution, &runtime, entity, "second");
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
                maximum_work_units: Some(0),
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: Some(0),
            },
            execution,
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
            execution,
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use crate::facade::indexes::{
        RelatedEntityEndpoint, RelatedEntityOrderingDirection, RelatedEntityOrderingField,
        RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint,
    };
    let runtime = runtime_with_index_field_aspects();
    let parent = create_entity(execution, &runtime, "parent");
    let alpha = create_entity(execution, &runtime, "alpha");
    let beta = create_entity(execution, &runtime, "beta");
    let left = create_entity(execution, &runtime, "left");
    let shared = create_entity(execution, &runtime, "shared");
    let right = create_entity(execution, &runtime, "right");
    create_relation(execution, &runtime, parent, alpha, "owns-alpha");
    create_relation(execution, &runtime, parent, beta, "owns-beta");
    let left_link = create_relation(execution, &runtime, left, shared, "left-link");
    let last = create_relation_outcome(execution, &runtime, shared, right, "right-link");
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
        .build_for_commit(execution, request(last.commit.commit_id))
        .failed_indexes
        .is_empty());

    let changed = update_entity(execution, &runtime, alpha, "zeta");
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
            execution,
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    assert!(refreshed.work.adjacency_work_units > 0);
    let rebuilt = runtime.index_authority().build_for_basis(
        execution,
        request(changed.commit.commit_id),
        &basis,
    );
    assert_eq!(refreshed.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in refreshed.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(
            incremental.entries, cold.entries,
            "index {:?}",
            incremental.index_id
        );
    }

    let deleted =
        delete_relation_on_branch(execution, &runtime, left_link, BranchId("main".into()));
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
            execution,
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    let rebuilt = runtime.index_authority().build_for_basis(
        execution,
        request(deleted.commit.commit_id),
        &basis,
    );
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

#[path = "maintenance/retained_reconstruction.rs"]
mod retained_reconstruction;
