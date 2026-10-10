use super::*;
use worth_execution::ExecutionAllocationPolicy;

#[test]
fn discarded_ordering_stays_cold_after_native_checkpoint_restore_then_rebuilds() {
    let fixture = OrderingFixture::new();
    let runtime = &fixture.runtime;
    let basis = fixture.basis();
    let built = runtime
        .index_authority()
        .build_for_basis(fixture.request(&basis), &basis);
    assert!(built.failed_indexes.is_empty());
    let committed = runtime.history().latest_commit().unwrap();
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(committed.commit_id)
        .unwrap();
    assert!(
        envelope.derived_index_artifacts().is_empty(),
        "current commits keep rebuildable payloads in the native catalog"
    );
    let generation = runtime
        .index_access()
        .published_generation_for_observation(fixture.index.index_id, &basis.observation())
        .unwrap();
    let discarded = runtime
        .index_authority()
        .discard_generations(DerivedIndexDiscardRequest::all_bases(
            fixture.index.index_id,
        ))
        .unwrap();
    assert!(discarded.removed_generation_count() > 0);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let mut restored = persisted_runtime_with_index_field_aspects();
    let recovered = restored
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .unwrap();
    assert_eq!(recovered.latest_commit, Some(committed));
    assert_eq!(
        restored.index_access().matching_definition(&fixture.index),
        Some(fixture.index.clone())
    );
    assert!(restored
        .index_access()
        .latest_generation(fixture.index.index_id, &BranchId("main".into()))
        .is_none());
    let (_, restored_basis) = restored
        .observe_branch(&restored.main_branch_identity())
        .unwrap();
    let snapshot = restored
        .snapshots()
        .snapshot_for_observation(&restored_basis.observation())
        .unwrap();
    let cold = restored
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot.clone(),
                fixture.index.index_id,
                fixture.parent,
                KindId(1),
                None,
                3,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
        )
        .unwrap_err();
    assert_eq!(
        cold.kind(),
        BoundedRelatedEntityOrderedLookupDenialKind::ExactGenerationUnavailable
    );
    let rebuilt = restored.index_authority().build_for_basis_with_lease(
        fixture.request(&restored_basis),
        &restored_basis,
        &test_execution_lease(),
    );
    assert!(rebuilt.failed_indexes.is_empty());
    assert!(rebuilt.execution_denial.is_none());
    assert_eq!(rebuilt.generations[0].entries, generation.entries);
    let page = restored
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            BoundedRelatedEntityOrderedLookupRequest::new(
                snapshot,
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
    let later = create_entity_outcome(&restored, "editable-after-cold-restore");
    assert_eq!(
        restored
            .history()
            .branch_head(&BranchId("main".into()))
            .as_ref(),
        Some(&later.commit)
    );
}
