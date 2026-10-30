use worth_store_test_support::harness::physical_isolation::epoch_scope as support;
use worth_store_test_support::harness::recovery::checkpoint_basis as checkpoint_basis_fixture;
use worth_store_test_support::harness::recovery::checkpoint_durability as checkpoint_durability_fixture;

use support::{
    current_generation_extent_reference, current_generation_page_reference,
    current_generation_segment_reference, current_root_from_authority,
    generation_counted_page_reference, physical_authority_from_complete_closeout,
    physical_authority_from_operation_digest_closeout,
};
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot,
    PhysicalReferenceAuthority, PhysicalSegmentId,
};
use worth_store_physical_isolation::{
    physical_epoch_vector_for_current_root, reject_checkpoint_root_as_current_read_authority,
    reject_manifest_locator_root_as_current_read_authority,
    reject_recovery_root_as_current_read_authority, required_physical_isolation_ordering_contracts,
    CheckpointPublicationIdentity, CheckpointPublicationRoot, CurrentPhysicalRoot,
    EpochComparisonScope, EpochRetryDecision, GenerationCountedPhysicalReference,
    ManifestLocatorRoot, PhysicalEpochDriftKind, PhysicalEpochVector, PhysicalOrderingContract,
    PhysicalOrderingContractDenial, PhysicalOrderingSite, PhysicalOrderingStrength, RecoveryRoot,
    RootKindMismatchDenial,
};

#[test]
fn current_root_epoch_vectors_are_scoped_to_read_plan_admission() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let epoch_vector = physical_epoch_vector_for_current_root(current_root).unwrap();

    let plan_epoch_freshness = physical_epoch_vector_for_current_root(current_root)
        .unwrap()
        .compare_against(epoch_vector);
    assert_eq!(plan_epoch_freshness.decision(), EpochRetryDecision::Current);
    assert_eq!(
        current_root.ordering(),
        PhysicalOrderingContract::root_swap_acquire_release()
    );
}

#[test]
fn epoch_vectors_compare_only_inside_declared_scope() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let expected = physical_epoch_vector_for_current_root(current_root).unwrap();
    let outside_scope = PhysicalEpochVector::for_scope(EpochComparisonScope::root_readmission(
        current_root.scope(),
    ))
    .with_root(current_root.epoch())
    .with_manifest(current_root.manifest_epoch())
    .seal()
    .unwrap();

    let freshness = expected.compare_against(outside_scope);

    assert_eq!(freshness.decision(), EpochRetryDecision::RebindRequired);
    assert_eq!(
        freshness.drift(),
        Some(PhysicalEpochDriftKind::ScopeMismatch)
    );
}

#[test]
fn drifted_root_manifest_extent_page_and_future_chunk_deny_before_read_plan() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let expected = physical_epoch_vector_for_current_root(current_root).unwrap();
    let different_authority = physical_authority_from_operation_digest_closeout("op-31");
    let drifted_root = current_root_from_authority(&different_authority);
    let observed_root_drift = PhysicalEpochVector::for_scope(expected.scope())
        .with_root(drifted_root.epoch())
        .with_manifest(current_root.manifest_epoch())
        .seal()
        .unwrap();

    assert_eq!(
        expected.compare_against(observed_root_drift).drift(),
        Some(PhysicalEpochDriftKind::RootEpoch)
    );
    assert_eq!(
        expected
            .compare_against(observed_root_drift)
            .into_stale_read_plan_denial()
            .unwrap()
            .drift(),
        PhysicalEpochDriftKind::RootEpoch
    );

    let observed_manifest_drift = PhysicalEpochVector::for_scope(expected.scope())
        .with_root(current_root.epoch())
        .with_manifest(drifted_root.manifest_epoch())
        .seal()
        .unwrap();
    assert_eq!(
        expected.compare_against(observed_manifest_drift).drift(),
        Some(PhysicalEpochDriftKind::ManifestEpoch)
    );
    assert_eq!(
        expected
            .compare_against(observed_manifest_drift)
            .into_stale_read_plan_denial()
            .unwrap()
            .drift(),
        PhysicalEpochDriftKind::ManifestEpoch
    );

    let observed_extent_drift = PhysicalEpochVector::for_scope(expected.scope())
        .with_root(current_root.epoch())
        .with_manifest(current_root.manifest_epoch())
        .with_extent(
            current_root
                .admit_extent_publication_epoch(current_generation_extent_reference(73))
                .unwrap()
                .epoch(),
        )
        .seal()
        .unwrap();
    assert_eq!(
        expected.compare_against(observed_extent_drift).drift(),
        Some(PhysicalEpochDriftKind::ExtentEpoch)
    );
    assert_eq!(
        expected
            .compare_against(observed_extent_drift)
            .into_stale_read_plan_denial()
            .unwrap()
            .drift(),
        PhysicalEpochDriftKind::ExtentEpoch
    );

    let observed_page_drift = PhysicalEpochVector::for_scope(expected.scope())
        .with_root(current_root.epoch())
        .with_manifest(current_root.manifest_epoch())
        .with_page(
            current_root
                .admit_page_publication_epoch(current_generation_page_reference(91))
                .unwrap()
                .epoch(),
        )
        .seal()
        .unwrap();
    assert_eq!(
        expected.compare_against(observed_page_drift).drift(),
        Some(PhysicalEpochDriftKind::PageEpoch)
    );
    assert_eq!(
        expected
            .compare_against(observed_page_drift)
            .into_stale_read_plan_denial()
            .unwrap()
            .drift(),
        PhysicalEpochDriftKind::PageEpoch
    );

    let observed_chunk_drift = PhysicalEpochVector::for_scope(expected.scope())
        .with_root(current_root.epoch())
        .with_manifest(current_root.manifest_epoch())
        .with_chunk(
            current_root
                .future_chunk_publication_epoch_placeholder()
                .epoch(),
        )
        .seal()
        .unwrap();
    assert_eq!(
        expected.compare_against(observed_chunk_drift).drift(),
        Some(PhysicalEpochDriftKind::ChunkEpoch)
    );
    assert_eq!(
        expected
            .compare_against(observed_chunk_drift)
            .into_stale_read_plan_denial()
            .unwrap()
            .drift(),
        PhysicalEpochDriftKind::ChunkEpoch
    );

    let chunk_denial = GenerationCountedPhysicalReference::reject_future_chunk_lifecycle_claim();
    assert!(matches!(
        chunk_denial,
        worth_store_physical_isolation::GenerationCountedReferenceDenial::FutureChunkLifecycleNotOwnedByS5
    ));
}

#[test]
fn root_kinds_are_distinct_authorities() {
    let authority = physical_authority_from_complete_closeout();
    let ordering = PhysicalOrderingContract::root_swap_acquire_release();
    let root_basis = authority.root_epoch_basis();
    let validation =
        checkpoint_durability_fixture::validate(checkpoint_basis_fixture::manifest(10, 20, 12));
    let checkpoint = CheckpointPublicationRoot::from_checkpoint_publication(
        root_basis.checkpoint_publication_root_basis(),
        ordering,
        CheckpointPublicationIdentity::from_checkpoint_id(validation.checkpoint_id()),
    )
    .unwrap();
    let recovery = RecoveryRoot::from_recovery_basis(root_basis.recovery_root_basis());
    let locator =
        ManifestLocatorRoot::from_manifest_locator_basis(root_basis.manifest_locator_root_basis());

    assert_eq!(
        reject_checkpoint_root_as_current_read_authority(checkpoint),
        RootKindMismatchDenial::CheckpointPublicationRootCannotAdmitCurrentReadPlan
    );
    assert_eq!(
        reject_recovery_root_as_current_read_authority(recovery),
        RootKindMismatchDenial::RecoveryRootRequiresEntryReadmission
    );
    assert_eq!(
        reject_manifest_locator_root_as_current_read_authority(locator),
        RootKindMismatchDenial::ManifestLocatorRootCannotAdmitCurrentReadPlan
    );
}

#[test]
fn generation_counted_references_reject_aba_reuse() {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(7).unwrap();
    let page = PhysicalPageId::from_raw(11).unwrap();
    let slot = PhysicalRecordSlot::from_raw(3).unwrap();
    let admitted_generation = PhysicalGeneration::from_raw(1).unwrap();
    let reused_generation = PhysicalGeneration::from_raw(2).unwrap();
    let cell = generations
        .slot_cell(segment, page, slot)
        .with_slot_generation(admitted_generation);
    let admitted_reference = references.admit_page_slot(cell);
    let counted = GenerationCountedPhysicalReference::from_admitted_reference(admitted_reference);

    let mismatch = counted
        .require_current_generation(reused_generation)
        .unwrap_err();

    assert_eq!(mismatch.admitted_generation(), admitted_generation);
    assert_eq!(mismatch.observed_generation(), reused_generation);

    let current_reference = counted
        .require_current_generation(admitted_generation)
        .unwrap();
    assert_eq!(current_reference.generation(), admitted_generation);
}

#[test]
fn stale_generation_reuse_cannot_reach_publication_epoch_admission() {
    let admitted_generation = PhysicalGeneration::from_raw(41).unwrap();
    let reused_generation = PhysicalGeneration::from_raw(42).unwrap();
    let counted = generation_counted_page_reference(admitted_generation.get());

    let mismatch = counted
        .require_current_generation(reused_generation)
        .unwrap_err();

    assert_eq!(mismatch.admitted_generation(), admitted_generation);
    assert_eq!(mismatch.observed_generation(), reused_generation);
}

#[test]
fn publication_epochs_are_issued_by_current_physical_root_not_generation_references() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let segment_reference = current_generation_segment_reference(13);
    let extent_reference = current_generation_extent_reference(17);
    let page_reference = current_generation_page_reference(19);

    assert_eq!(segment_reference.generation().get(), 13);
    assert_eq!(extent_reference.generation().get(), 17);
    assert_eq!(page_reference.generation().get(), 19);
    assert_ne!(
        current_root
            .admit_segment_publication_epoch(segment_reference)
            .unwrap()
            .epoch()
            .get(),
        13
    );
    assert_ne!(
        current_root
            .admit_extent_publication_epoch(extent_reference)
            .unwrap()
            .epoch()
            .get(),
        17
    );
    assert_ne!(
        current_root
            .admit_page_publication_epoch(page_reference)
            .unwrap()
            .epoch()
            .get(),
        19
    );
}

#[test]
fn ordering_contract_rejects_relaxed_and_ambient_ordering() {
    assert!(PhysicalOrderingContract::reject_relaxed().is_err());
    assert!(PhysicalOrderingContract::reject_ambient().is_err());
    let contracts = required_physical_isolation_ordering_contracts();
    assert_eq!(contracts.len(), 6);
    for site in [
        PhysicalOrderingSite::RootSwap,
        PhysicalOrderingSite::HazardPublication,
        PhysicalOrderingSite::ReaderEpochPublication,
        PhysicalOrderingSite::GenerationAdvancement,
        PhysicalOrderingSite::AllocatorPublication,
        PhysicalOrderingSite::Validation,
    ] {
        let matching: Vec<_> = contracts
            .iter()
            .filter(|contract| contract.site() == site)
            .collect();
        assert_eq!(matching.len(), 1);
        assert_eq!(
            matching[0].strength(),
            PhysicalOrderingStrength::AcquireRelease
        );
    }
}

#[test]
fn wrong_ordering_site_cannot_admit_current_root() {
    let authority = physical_authority_from_complete_closeout();
    let denial = CurrentPhysicalRoot::from_physical_isolation_entry(
        authority.root_epoch_basis().current_root_basis(),
        PhysicalOrderingContract::acquire_release_for(PhysicalOrderingSite::HazardPublication),
    )
    .unwrap_err();

    assert_eq!(
        denial,
        PhysicalOrderingContractDenial::WrongOrderingSite {
            expected: PhysicalOrderingSite::RootSwap,
            actual: PhysicalOrderingSite::HazardPublication,
        }
    );

    let stronger_root = CurrentPhysicalRoot::from_physical_isolation_entry(
        authority.root_epoch_basis().current_root_basis(),
        PhysicalOrderingContract::sequentially_consistent_for(PhysicalOrderingSite::RootSwap),
    )
    .unwrap();
    assert_eq!(
        stronger_root.ordering().strength(),
        PhysicalOrderingStrength::SequentiallyConsistent
    );
}
