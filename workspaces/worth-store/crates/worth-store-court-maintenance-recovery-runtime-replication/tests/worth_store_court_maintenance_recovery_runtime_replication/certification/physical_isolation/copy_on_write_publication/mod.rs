use worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference;
use {
    worth_store_test_support::harness::physical_isolation::publication::admitted_copy_on_write_plan,
    worth_store_test_support::harness::physical_isolation::publication::mismatched_release_receipt,
    worth_store_test_support::harness::physical_isolation::publication::publication_inputs,
    worth_store_test_support::harness::physical_isolation::publication::publication_inputs_for_store,
    worth_store_test_support::harness::physical_isolation::publication::root_publication_validation,
};

#[test]
fn copy_on_write_plan_preserves_exact_root_and_release_correlation() {
    let inputs = publication_inputs();
    let receipt = admitted_copy_on_write_plan(&inputs).complete_plan();

    assert_eq!(
        receipt.epochs().root().old().get(),
        inputs.old_root.epoch().get()
    );
    assert_eq!(
        receipt.epochs().root().new_epoch().get(),
        inputs.new_root.epoch().get()
    );
    assert_eq!(
        receipt.epochs().manifest().old().get(),
        inputs.old_root.manifest_epoch().get()
    );
    assert_eq!(
        receipt.epochs().manifest().new_epoch().get(),
        inputs.new_root.manifest_epoch().get()
    );
    assert_eq!(
        receipt.release_posture(),
        worth_store_physical_isolation::PhysicalPublicationReleasePosture::OldReachabilityRetainedUntilReadRelease
    );
    assert!(receipt.old_reachability().retained_until_release());
    assert_eq!(receipt.counters().intent_validations(), 1);
    assert_eq!(receipt.counters().old_reachability_checks(), 1);
    assert_eq!(receipt.counters().epoch_checks(), 1);
    assert_eq!(receipt.counters().ordering_checks(), 1);
    assert_eq!(receipt.counters().readiness_joins(), 1);

    assert_eq!(
        receipt
            .admit_old_reachability_release(inputs.old_release)
            .unwrap()
            .footprint_basis(),
        inputs.old_reachability.footprint_basis()
    );
    assert!(matches!(
        receipt
            .admit_old_reachability_release(mismatched_release_receipt(777))
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalPublicationDenial::ReclaimBeforeReadPlanRelease { .. }
    ));
}

#[test]
fn publication_plan_exposes_the_declared_successor_root() {
    let inputs = publication_inputs();
    let publication = admitted_copy_on_write_plan(&inputs).complete_plan();

    assert_eq!(
        publication.old_root().epoch().get(),
        inputs.old_root.epoch().get()
    );
    assert_eq!(
        publication.new_root().epoch().get(),
        inputs.new_root.epoch().get()
    );
}

#[test]
fn publication_denies_in_place_overwrite_and_missing_old_reachability() {
    let inputs = publication_inputs();
    let in_place = worth_store_physical_isolation::PhysicalPublicationIntent::in_place_reachable_overwrite_attempt(
        inputs.old_candidate,
        inputs.new_candidate,
        inputs.old_reachability,
    )
    .validate_copy_on_write_inputs()
    .unwrap_err();
    assert_eq!(
        in_place,
        worth_store_physical_isolation::PhysicalPublicationDenial::InPlaceReachableOverwrite
    );

    let missing = worth_store_physical_isolation::PhysicalPublicationIntent::missing_old_reachability_attempt(
        inputs.old_candidate,
        inputs.new_candidate,
    )
    .validate_copy_on_write_inputs()
    .unwrap_err();
    assert_eq!(
        missing,
        worth_store_physical_isolation::PhysicalPublicationDenial::MissingOldReachability
    );
}

#[test]
fn publication_denies_stale_epochs_and_weak_root_swap_ordering() {
    let inputs = publication_inputs();
    let stale =
        worth_store_physical_isolation::PhysicalPublicationIntent::copy_on_write_root_manifest(
            inputs.old_candidate,
            inputs.old_candidate,
            inputs.old_reachability,
        )
        .validate_copy_on_write_inputs()
        .unwrap_err();
    assert_eq!(
        stale,
        worth_store_physical_isolation::PhysicalPublicationDenial::StaleRootPublicationEpoch
    );

    let weak = worth_store_physical_isolation::RootSwapOrderingContract::from_ordering(
        worth_store_physical_isolation::PhysicalOrderingContract::acquire_release_for(
            worth_store_physical_isolation::PhysicalOrderingSite::Validation,
        ),
    )
    .unwrap_err();
    assert!(matches!(
        weak,
        worth_store_physical_isolation::PhysicalPublicationDenial::WeakOrdering(_)
    ));

    let intent =
        worth_store_physical_isolation::PhysicalPublicationIntent::copy_on_write_root_manifest(
            inputs.old_candidate,
            inputs.new_candidate,
            inputs.old_reachability,
        )
        .validate_copy_on_write_inputs()
        .unwrap();
    let stronger = worth_store_physical_isolation::RootSwapOrderingContract::from_ordering(
        worth_store_physical_isolation::PhysicalOrderingContract::sequentially_consistent_for(
            worth_store_physical_isolation::PhysicalOrderingSite::RootSwap,
        ),
    )
    .unwrap();
    let lowered = intent.clone().lower_with_ordering(stronger).unwrap();
    let readiness = worth_store_physical_isolation::PhysicalPublicationReadiness::from_validated_intent(
        &intent,
        worth_store_physical_isolation::NewRootPublicationProof::from_root_validation(inputs.new_validation),
        worth_store_physical_isolation::PublicationLatchReadiness::declared_publish_latches_released_before_blocking_io(),
    );
    let publication = lowered.join_readiness(readiness).unwrap().complete_plan();
    assert_eq!(
        publication.ordering().ordering().strength(),
        worth_store_physical_isolation::PhysicalOrderingStrength::SequentiallyConsistent
    );
}

#[test]
fn identity_reuse_requires_allocator_publication_fence() {
    let inputs = publication_inputs();
    let intent = worth_store_physical_isolation::PhysicalPublicationIntent::copy_on_write_root_manifest_with_identity_reuse(
        inputs.old_candidate,
        inputs.new_candidate,
        inputs.old_reachability,
    )
    .validate_copy_on_write_inputs()
    .unwrap();
    assert_eq!(
        intent.identity_reuse(),
        worth_store_physical_isolation::PhysicalIdentityReuse::Requested
    );
    let lowered = intent
        .clone()
        .lower_with_ordering(
            worth_store_physical_isolation::RootSwapOrderingContract::acquire_release_or_stronger(),
        )
        .unwrap();
    let readiness = worth_store_physical_isolation::PhysicalPublicationReadiness::from_validated_intent(
        &intent,
        worth_store_physical_isolation::NewRootPublicationProof::from_root_validation(inputs.new_validation),
        worth_store_physical_isolation::PublicationLatchReadiness::declared_publish_latches_released_before_blocking_io(),
    );
    let denial = lowered.join_readiness(readiness).unwrap_err();
    assert_eq!(
        denial,
        worth_store_physical_isolation::PhysicalPublicationDenial::IdentityReuseWithoutCrashStableFence
    );

    let fence = worth_store_physical_isolation::AllocatorPublicationFence::from_ordering(
        worth_store_physical_isolation::PhysicalOrderingContract::sequentially_consistent_for(
            worth_store_physical_isolation::PhysicalOrderingSite::AllocatorPublication,
        ),
    )
    .unwrap();
    let old_identity = current_generation_page_reference(701);
    let reused_identity = current_generation_page_reference(702);
    let released = inputs
        .old_reachability
        .admit_release(inputs.old_release)
        .unwrap();
    let reuse = worth_store_physical_isolation::CrashStableFreeReusePosture::admit(
        fence,
        old_identity,
        reused_identity,
        released,
    )
    .unwrap();
    let intent = worth_store_physical_isolation::PhysicalPublicationIntent::copy_on_write_root_manifest_with_identity_reuse(
        inputs.old_candidate,
        inputs.new_candidate,
        inputs.old_reachability,
    )
    .validate_copy_on_write_inputs()
    .unwrap();
    let readiness = worth_store_physical_isolation::PhysicalPublicationReadiness::from_validated_intent(
        &intent,
        worth_store_physical_isolation::NewRootPublicationProof::from_root_validation(inputs.new_validation),
        worth_store_physical_isolation::PublicationLatchReadiness::declared_publish_latches_released_before_blocking_io(),
    )
    .with_free_reuse_posture(reuse)
    .unwrap();
    let receipt = intent
        .lower_with_ordering(
            worth_store_physical_isolation::RootSwapOrderingContract::acquire_release_or_stronger(),
        )
        .unwrap()
        .join_readiness(readiness)
        .unwrap()
        .complete_plan();
    assert_eq!(
        receipt.release_posture(),
        worth_store_physical_isolation::PhysicalPublicationReleasePosture::IdentityReuseProtectedByAllocatorFence
    );
    assert_eq!(receipt.free_reuse(), Some(reuse));
    assert_eq!(
        reuse.fence().ordering().strength(),
        worth_store_physical_isolation::PhysicalOrderingStrength::SequentiallyConsistent
    );
}

#[test]
fn identity_reuse_denies_owner_mismatch_and_stale_generation() {
    let inputs = publication_inputs();
    let released = inputs
        .old_reachability
        .admit_release(inputs.old_release)
        .unwrap();
    let fence = worth_store_physical_isolation::AllocatorPublicationFence::from_ordering(
        worth_store_physical_isolation::PhysicalOrderingContract::acquire_release_for(
            worth_store_physical_isolation::PhysicalOrderingSite::AllocatorPublication,
        ),
    )
    .unwrap();
    let old_identity = current_generation_page_reference(711);
    let stale_identity = current_generation_page_reference(711);
    assert_eq!(
        worth_store_physical_isolation::CrashStableFreeReusePosture::admit(fence, old_identity, stale_identity, released)
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalPublicationDenial::IdentityReuseWithoutGenerationAdvance
    );
    let other_owner = worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference(712);
    assert_eq!(
        worth_store_physical_isolation::CrashStableFreeReusePosture::admit(
            fence,
            old_identity,
            other_owner,
            released
        )
        .unwrap_err(),
        worth_store_physical_isolation::PhysicalPublicationDenial::IdentityReuseOwnerMismatch
    );
}

#[test]
fn readiness_denies_new_root_proof_mismatch_before_publish() {
    let inputs = publication_inputs();
    let mismatched_validation = root_publication_validation(2702, 99);
    let intent =
        worth_store_physical_isolation::PhysicalPublicationIntent::copy_on_write_root_manifest(
            inputs.old_candidate,
            inputs.new_candidate,
            inputs.old_reachability,
        )
        .validate_copy_on_write_inputs()
        .unwrap();
    let readiness = worth_store_physical_isolation::PhysicalPublicationReadiness::from_validated_intent(
        &intent,
        worth_store_physical_isolation::NewRootPublicationProof::from_root_validation(mismatched_validation),
        worth_store_physical_isolation::PublicationLatchReadiness::declared_publish_latches_released_before_blocking_io(),
    );

    let denial = intent
        .lower_with_ordering(
            worth_store_physical_isolation::RootSwapOrderingContract::acquire_release_or_stronger(),
        )
        .unwrap()
        .join_readiness(readiness)
        .unwrap_err();

    assert_eq!(
        denial,
        worth_store_physical_isolation::PhysicalPublicationDenial::NewRootPublicationProofMismatch
    );
}

#[test]
fn root_candidate_denies_validation_for_unrelated_published_root() {
    let inputs = publication_inputs();
    let unrelated_validation = root_publication_validation(inputs.new_root.scope() + 99, 1);

    assert_eq!(
        worth_store_physical_isolation::PublicationRootCandidate::admit(inputs.new_root, unrelated_validation).unwrap_err(),
        worth_store_physical_isolation::PhysicalPublicationDenial::RootPublicationValidationRootMismatch
    );
}

#[test]
fn copy_on_write_validation_rejects_roots_issued_by_different_stores() {
    let current = publication_inputs();
    let foreign_identity =
        worth_store_test_support::harness::layout::foreign_layout_physical_store_identity();
    let foreign = publication_inputs_for_store(&foreign_identity, 713);
    let intent =
        worth_store_physical_isolation::PhysicalPublicationIntent::copy_on_write_root_manifest(
            current.old_candidate,
            foreign.new_candidate,
            current.old_reachability,
        );

    assert_eq!(
        intent.validate_copy_on_write_inputs().unwrap_err(),
        worth_store_physical_isolation::PhysicalPublicationDenial::StoreAuthorityMismatch
    );
}
