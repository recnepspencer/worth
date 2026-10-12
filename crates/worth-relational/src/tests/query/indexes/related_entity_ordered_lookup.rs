use super::*;
use crate::facade::indexes::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenialKind,
    BoundedRelatedEntityOrderedLookupRequest, DerivedIndexEntries, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField,
};

#[test]
fn related_ordering_certification_rejects_a_missing_candidate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let parent = changed_entities(&create_entity_outcome(execution, &runtime, "parent"))[0];
    let alpha = changed_entities(&create_entity_outcome(execution, &runtime, "alpha"))[0];
    let beta = changed_entities(&create_entity_outcome(execution, &runtime, "beta"))[0];
    create_relation_outcome(execution, &runtime, parent, alpha, "owns-alpha");
    create_relation_outcome(execution, &runtime, parent, beta, "owns-beta");
    let index = register_related_name_index(&runtime);
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            let DerivedIndexEntries::RelatedEntityOrdering(entries) = &mut generation.entries
            else {
                panic!("related-ordering generation expected");
            };
            entries.get_mut(&parent).unwrap().remove(0);
        });
    let snapshot = runtime.visibility_authority().snapshot();
    let request = || {
        BoundedRelatedEntityOrderedLookupRequest::new(
            snapshot.clone(),
            index.index_id,
            parent,
            KindId(1),
            None,
            2,
        )
        .unwrap()
    };

    let production = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            request(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    assert_eq!(production.child_entity_ids().len(), 1);
    let denial = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            request(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        BoundedRelatedEntityOrderedLookupDenialKind::StorageParityMismatch
    );
}

#[test]
fn related_entity_pages_seek_exact_order_with_identity_ties() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let parent = changed_entities(&create_entity_outcome(execution, &runtime, "parent"))[0];
    let alpha_one = changed_entities(&create_entity_outcome(execution, &runtime, "alpha"))[0];
    let beta = changed_entities(&create_entity_outcome(execution, &runtime, "beta"))[0];
    let alpha_two = changed_entities(&create_entity_outcome(execution, &runtime, "alpha"))[0];
    create_relation_outcome(execution, &runtime, parent, beta, "owns");
    create_relation_outcome(execution, &runtime, parent, alpha_two, "owns");
    create_relation_outcome(execution, &runtime, parent, alpha_one, "owns");
    let index = register_related_name_index(&runtime);
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    let snapshot = runtime.visibility_authority().snapshot();

    let first = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot.clone(),
                index.index_id,
                parent,
                KindId(1),
                None,
                2,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    let mut expected_alphas = vec![alpha_one, alpha_two];
    expected_alphas.sort();
    assert_eq!(first.child_entity_ids(), expected_alphas);
    assert_eq!(first.examined_entry_count(), 2);
    assert_eq!(first.seek_comparison_count(), 0);
    assert!(first.has_more());

    let second = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot,
                index.index_id,
                parent,
                KindId(1),
                first.next_boundary().cloned(),
                2,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert_eq!(second.child_entity_ids(), &[beta]);
    assert!(!second.has_more());
    assert!(second.seek_comparison_count() > 0);
    assert_eq!(
        second.examined_entry_count(),
        second.child_entity_ids().len() + second.seek_comparison_count()
    );
    assert_eq!(second.generation_id(), build.generations[0].generation_id);
}

#[test]
fn related_entity_boundary_is_parent_and_generation_scoped() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let parent = changed_entities(&create_entity_outcome(execution, &runtime, "parent"))[0];
    let foreign_parent = changed_entities(&create_entity_outcome(
        execution,
        &runtime,
        "foreign-parent",
    ))[0];
    let child = changed_entities(&create_entity_outcome(execution, &runtime, "alpha"))[0];
    let later_child = changed_entities(&create_entity_outcome(execution, &runtime, "beta"))[0];
    create_relation_outcome(execution, &runtime, parent, child, "owns");
    create_relation_outcome(execution, &runtime, parent, later_child, "owns");
    let index = register_related_name_index(&runtime);
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    let snapshot = runtime.visibility_authority().snapshot();
    let first = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot.clone(),
                index.index_id,
                parent,
                KindId(1),
                None,
                1,
            )
            .unwrap(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    assert!(first.has_more());
    let boundary = first.next_boundary().cloned().unwrap();
    let denial = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot,
                index.index_id,
                foreign_parent,
                KindId(1),
                Some(boundary),
                1,
            )
            .unwrap(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        crate::facade::indexes::BoundedRelatedEntityOrderedLookupDenialKind::ForeignBoundary
    );
}

#[test]
fn expected_generation_mismatch_denies_before_ordered_page_access() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let parent = changed_entities(&create_entity_outcome(execution, &runtime, "parent"))[0];
    let child = changed_entities(&create_entity_outcome(execution, &runtime, "alpha"))[0];
    create_relation_outcome(execution, &runtime, parent, child, "owns");
    let index = register_related_name_index(&runtime);
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    let actual = build.generations[0].generation_id;
    let foreign = crate::facade::indexes::DerivedIndexGenerationId(actual.0 + 1);
    let snapshot = runtime.visibility_authority().snapshot();
    let denial = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot,
                index.index_id,
                parent,
                KindId(1),
                None,
                1,
            )
            .unwrap()
            .expect_generation(foreign),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        crate::facade::indexes::BoundedRelatedEntityOrderedLookupDenialKind::ExpectedGenerationMismatch
    );
}

#[test]
fn related_ordering_rebuilds_retained_historical_child_values_after_update() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let parent = changed_entities(&create_entity_outcome(execution, &runtime, "parent"))[0];
    let child = changed_entities(&create_entity_outcome(execution, &runtime, "historical"))[0];
    let historical = create_relation_outcome(execution, &runtime, parent, child, "owns-historical");
    let index = register_related_name_index(&runtime);

    update_entity(execution, &runtime, child, "current");
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: historical.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let result = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                historical.snapshot.clone(),
                index.index_id,
                parent,
                KindId(1),
                None,
                1,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();

    assert_eq!(result.child_entity_ids(), &[child]);
    assert!(!result.has_more());
}

fn register_related_name_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "relation.owns.child-name".to_string(),
        kind: DerivedIndexKind::RelatedEntityOrdering {
            relation_kind: KindId(2),
            parent_endpoint: RelatedEntityEndpoint::SourceParent,
            child_kind: KindId(1),
            ordering: vec![RelatedEntityOrderingField::new(
                aspect_field_locator(aspect_key("name"), field_key("name")),
                RelatedEntityOrderingDirection::Ascending,
            )],
        },
        branch_scoped: false,
    })
}
