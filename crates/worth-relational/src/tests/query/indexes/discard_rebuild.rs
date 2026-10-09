use super::*;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::facade::identity::EntityId;
use crate::facade::indexes::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenialKind,
    BoundedRelatedEntityOrderedLookupRequest, DerivedIndexDiscardDenial,
    DerivedIndexDiscardRequest, DerivedIndexEntries, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField,
};

mod checkpoint;

struct OrderingFixture {
    runtime: RelationalRuntime,
    index: DerivedIndexDefinition,
    parent: EntityId,
    expected: Vec<EntityId>,
}

impl OrderingFixture {
    fn new() -> Self {
        let runtime = persisted_runtime_with_index_field_aspects();
        let parent = changed_entities(&create_entity_outcome(&runtime, "parent"))[0];
        let alpha_one = changed_entities(&create_entity_outcome(&runtime, "alpha"))[0];
        let beta = changed_entities(&create_entity_outcome(&runtime, "beta"))[0];
        let alpha_two = changed_entities(&create_entity_outcome(&runtime, "alpha"))[0];
        for child in [beta, alpha_two, alpha_one] {
            create_relation_outcome(&runtime, parent, child, "owns");
        }
        let index = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "discard.owns.child-name".into(),
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
        });
        let mut expected = vec![alpha_one, alpha_two];
        expected.sort();
        expected.push(beta);
        Self {
            runtime,
            index,
            parent,
            expected,
        }
    }

    fn basis(&self) -> AdmittedRelationalBranchBasis {
        self.runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1
    }

    fn request(&self, basis: &AdmittedRelationalBranchBasis) -> DerivedIndexBuildRequest {
        let observation = basis.observation();
        DerivedIndexBuildRequest {
            source_commit_id: observation.commit_id().unwrap(),
            branch_id: observation.identity().branch_id().clone(),
            index_ids: vec![self.index.index_id],
        }
    }

    fn assert_cold(&self, snapshot: &crate::snapshots::data::SnapshotHandle) {
        let denial = self
            .runtime
            .index_access()
            .execute_bounded_related_entity_ordered_lookup(
                BoundedRelatedEntityOrderedLookupRequest::new(
                    snapshot.clone(),
                    self.index.index_id,
                    self.parent,
                    KindId(1),
                    None,
                    3,
                )
                .unwrap(),
                BoundedIndexParityMode::Certification,
            )
            .unwrap_err();
        assert_eq!(
            denial.kind(),
            BoundedRelatedEntityOrderedLookupDenialKind::ExactGenerationUnavailable
        );
    }
}

#[test]
fn discard_all_bases_keeps_readers_and_rebuilds_complete_ordered_content() {
    let fixture = OrderingFixture::new();
    let runtime = &fixture.runtime;
    let historical = fixture.basis();
    let historical_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&historical.observation())
        .unwrap();
    let old = runtime
        .index_authority()
        .build_for_basis(fixture.request(&historical), &historical);
    assert!(old.failed_indexes.is_empty());
    let retained = runtime
        .index_access()
        .published_generation_for_observation(fixture.index.index_id, &historical.observation())
        .unwrap();
    let weak = Arc::downgrade(&retained);
    let sibling = create_branch_from_main(runtime, "discard-sibling");
    let (_, sibling_basis) = runtime
        .observe_branch(&runtime.branch_identity(&sibling).unwrap())
        .unwrap();
    let sibling_build = runtime
        .index_authority()
        .build_for_basis(fixture.request(&sibling_basis), &sibling_basis);
    assert!(sibling_build.failed_indexes.is_empty());
    create_entity_outcome(runtime, "unrelated-new-record");
    let current = fixture.basis();
    let current_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&current.observation())
        .unwrap();
    let built = runtime
        .index_authority()
        .build_for_basis(fixture.request(&current), &current);
    assert!(built.failed_indexes.is_empty());
    let unrelated = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "discard.unrelated.name".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let mut unrelated_request = fixture.request(&current);
    unrelated_request.index_ids = vec![unrelated.index_id];
    let unrelated_build = runtime
        .index_authority()
        .build_for_basis(unrelated_request, &current);
    let untouched = runtime
        .index_access()
        .latest_generation(unrelated.index_id, &BranchId("main".into()))
        .unwrap();
    assert_eq!(unrelated_build.generations.len(), 1);
    let selected_count = runtime
        .index_access()
        .generations_snapshot()
        .iter()
        .filter(|generation| generation.index_id == fixture.index.index_id)
        .count();
    assert!(
        selected_count >= 3,
        "current, historical and sibling generations exist"
    );

    let before = runtime.current_version_id();
    let discarded = runtime
        .index_authority()
        .discard_generations(DerivedIndexDiscardRequest::all_bases(
            fixture.index.index_id,
        ))
        .unwrap();
    assert_eq!(discarded.index_id(), fixture.index.index_id);
    assert_eq!(discarded.removed_generation_count(), selected_count);
    assert!(runtime
        .index_access()
        .generations_snapshot()
        .iter()
        .all(|generation| generation.index_id != fixture.index.index_id));
    assert_eq!(runtime.current_version_id(), before);
    assert_eq!(
        runtime.index_access().matching_definition(&fixture.index),
        Some(fixture.index.clone())
    );
    for basis in [&historical, &sibling_basis, &current] {
        assert!(runtime
            .index_access()
            .published_generation_for_observation(fixture.index.index_id, &basis.observation())
            .is_none());
    }
    for branch in [BranchId("main".into()), sibling] {
        assert!(runtime
            .index_access()
            .latest_generation(fixture.index.index_id, &branch)
            .is_none());
    }
    fixture.assert_cold(&historical_snapshot);
    fixture.assert_cold(&current_snapshot);
    let DerivedIndexEntries::RelatedEntityOrdering(entries) = &retained.entries else {
        panic!("ordering")
    };
    assert_eq!(entries.get(&fixture.parent).unwrap().len(), 3);
    assert!(weak.upgrade().is_some());
    assert!(Arc::ptr_eq(
        &untouched,
        &runtime
            .index_access()
            .latest_generation(unrelated.index_id, &BranchId("main".into()))
            .unwrap()
    ));

    let lease = test_execution_lease();
    let cancellation = worth_execution::CancellationToken::new();
    let stopped = lease.controlled_child(cancellation.clone(), None);
    cancellation.cancel();
    let refused = runtime.index_authority().build_for_basis_with_lease(
        fixture.request(&current),
        &current,
        &stopped,
    );
    assert!(refused.generations.is_empty());
    assert_eq!(refused.failed_indexes, vec![fixture.index.index_id]);
    assert!(refused.basis_denial.is_none());
    assert_eq!(
        refused.execution_denial.unwrap().kind,
        crate::facade::indexes::DerivedIndexExecutionDenialKind::Cancelled
    );
    fixture.assert_cold(&current_snapshot);
    assert_eq!(runtime.current_version_id(), before);
    let rebuilt = runtime.index_authority().build_for_basis_with_lease(
        fixture.request(&current),
        &current,
        &lease,
    );
    assert!(rebuilt.failed_indexes.is_empty());
    assert!(rebuilt.execution_denial.is_none());
    assert_eq!(rebuilt.generations[0].entries, built.generations[0].entries);
    assert!(rebuilt.generations[0].generation_id > built.generations[0].generation_id);
    let page = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                current_snapshot.clone(),
                fixture.index.index_id,
                fixture.parent,
                KindId(1),
                None,
                3,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
        )
        .unwrap();
    assert_eq!(page.child_entity_ids(), fixture.expected);
    assert!(!page.has_more());
    let old_generation = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                current_snapshot,
                fixture.index.index_id,
                fixture.parent,
                KindId(1),
                None,
                3,
            )
            .unwrap()
            .expect_generation(built.generations[0].generation_id),
            BoundedIndexParityMode::Production,
        )
        .unwrap_err();
    assert_eq!(
        old_generation.kind(),
        BoundedRelatedEntityOrderedLookupDenialKind::ExpectedGenerationMismatch
    );
    drop(retained);
    assert!(
        weak.upgrade().is_none(),
        "catalog no longer retains the generation Arc; entry backing may still be shared"
    );
}

#[test]
fn discard_requires_installed_definition_and_empty_discard_is_idempotent() {
    let fixture = OrderingFixture::new();
    let authority = fixture.runtime.index_authority();
    let missing = DerivedIndexId(fixture.index.index_id.0 + 1);
    assert_eq!(
        authority.discard_generations(DerivedIndexDiscardRequest::all_bases(missing)),
        Err(DerivedIndexDiscardDenial::IndexNotInstalled { index_id: missing })
    );
    for _ in 0..2 {
        assert_eq!(
            authority
                .discard_generations(DerivedIndexDiscardRequest::all_bases(
                    fixture.index.index_id
                ))
                .unwrap()
                .removed_generation_count(),
            0
        );
    }
}
