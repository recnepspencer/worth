pub(crate) use {worth_store_test_support::harness::physical_isolation::epoch_scope as support};

pub(crate) use {worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority, worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout};
pub(crate) use {worth_foundational::FoundationalBoundaryArtifactRole, worth_foundational::FoundationalDiagnosticArtifactKind, worth_foundational::FoundationalDiagnosticBreachClass};
pub(crate) use {worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalFutureChunkId, worth_store_physical_format::PhysicalFutureChunkReference, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalReferenceAuthority, worth_store_physical_format::PhysicalSegmentId};
pub(crate) use {worth_store_physical_isolation::admit_seed_stable_read_plan, worth_store_physical_isolation::ChunkMigrationReadInterlockPlan, worth_store_physical_isolation::FutureBlobMigrationNonClaim, worth_store_physical_isolation::FutureChunkStabilityBasis, worth_store_physical_isolation::GenerationCountedPhysicalReference, worth_store_physical_isolation::MovablePhysicalRef, worth_store_physical_isolation::PhysicalChunkStabilityPlaceholder, worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics, worth_store_physical_isolation::ProtectedPhysicalReferenceSet, worth_store_physical_isolation::PublishedReaderHazard, worth_store_physical_isolation::ReadPlanAdmissionScratchArena, worth_store_physical_isolation::StablePhysicalReadPlan, worth_store_physical_isolation::TierMovementAdmissionLabel, worth_store_physical_isolation::TierMovementReadInterlockPlan, worth_store_physical_isolation::TierMovementStabilityDenial, worth_store_physical_isolation::TierMovementStabilityVerdict, worth_store_physical_isolation::TraversalAdmissionGuard, worth_store_physical_isolation::UnprotectedReadIntent, worth_store_physical_isolation::UnsupportedTierMovementClaim, worth_store_physical_isolation::UnsupportedTierMovementRequest};

pub(crate) fn ordinary_plan() -> StablePhysicalReadPlan {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let references = worth_store_physical_isolation::ProtectedPhysicalReferenceSet::from_current_generation_refs_with_scratch(
        [
            current_generation_segment_reference(13),
            current_generation_extent_reference(17),
            current_generation_page_reference(19),
        ],
        worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(4),
    )
    .unwrap();
    let observed = references.clone();
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(root, references, 8192)
        .with_release_semantics(worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all());
    let hazard = worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent).unwrap();
    let observation =
        worth_store_physical_isolation::PostProtectionPhysicalReadObservation::from_authority_after_hazard_publication(
            &authority, &hazard, root, observed,
        )
        .unwrap();
    let validated = hazard
        .observe_authority_after_publication(&authority, observation)
        .unwrap()
        .validate()
        .unwrap();
    let receipt = worth_store_physical_isolation::TraversalAdmissionGuard::from_validated_root(validated)
        .admit(worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(4))
        .unwrap();
    admit_seed_stable_read_plan(receipt.into_cursor().finish()).unwrap()
}

pub(crate) fn movable_extent(
    barrier: worth_store_physical_isolation::PhysicalReadReachabilityBarrier,
    generation: u64,
) -> MovablePhysicalRef {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let extent = current_generation_extent_reference(generation);
    let epoch = root.admit_extent_publication_epoch(extent).unwrap().epoch();
    worth_store_physical_isolation::MovablePhysicalRef::extent(extent, epoch, barrier).unwrap()
}

pub(crate) fn future_chunk_placeholder(
    barrier: worth_store_physical_isolation::PhysicalReadReachabilityBarrier,
) -> PhysicalChunkStabilityPlaceholder {
    let reference = future_chunk_reference(7, 3);
    future_chunk_placeholder_with_reference(reference, barrier)
}

pub(crate) fn future_chunk_placeholder_with_reference(
    reference: PhysicalFutureChunkReference,
    barrier: worth_store_physical_isolation::PhysicalReadReachabilityBarrier,
) -> PhysicalChunkStabilityPlaceholder {
    let epoch = future_chunk_epoch();
    let basis = worth_store_physical_isolation::FutureChunkStabilityBasis::from_stability_receipt(reference, epoch, barrier);
    worth_store_physical_isolation::PhysicalChunkStabilityPlaceholder::admit_with_epoch(reference, epoch, basis).unwrap()
}

pub(crate) fn future_chunk_reference(id: u64, generation: u64) -> PhysicalFutureChunkReference {
    worth_store_physical_format::PhysicalFutureChunkReference::stability_placeholder(
        worth_store_physical_format::PhysicalFutureChunkId::from_raw(id).unwrap(),
        worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap(),
    )
}

pub(crate) fn assert_equivalent_future_chunk_stability(
    chunk_plan: ChunkMigrationReadInterlockPlan,
    placeholder: PhysicalChunkStabilityPlaceholder,
) {
    let recipe = chunk_plan.resolved_stability_recipe();
    let basis = recipe.strong_basis().value();
    assert_eq!(
        recipe.payload().placeholder().reference(),
        placeholder.reference()
    );
    assert_eq!(
        recipe.payload().placeholder().reference().generation(),
        placeholder.reference().generation()
    );
    assert_eq!(basis.reference(), placeholder.reference());
    assert_eq!(
        basis.reference().generation(),
        placeholder.reference().generation()
    );
    assert_eq!(basis.epoch(), placeholder.epoch());
    assert_eq!(basis.reachability(), placeholder.reachability());
}

pub(crate) fn assert_non_claim_report(
    report: worth_store_physical_isolation::FutureBlobMigrationNonClaimReport,
) {
    assert_eq!(
        report.blob_lifecycle(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S7OwnsBlobLifecycle
    );
    assert_eq!(
        report.blob_retention(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S7OwnsBlobRetention
    );
    assert_eq!(
        report.blob_dedupe(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S7OwnsBlobDedupe
    );
    assert_eq!(
        report.resumable_writes(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S7OwnsResumableWrites
    );
    assert_eq!(
        report.cold_tier_qos(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S6OwnsColdTierQos
    );
}

pub(crate) fn future_chunk_epoch() -> worth_store_physical_isolation::ChunkEpoch {
    let authority = physical_authority_from_complete_closeout();
    current_root_from_authority(&authority)
        .future_chunk_publication_epoch_placeholder()
        .epoch()
}

pub(crate) fn generation_counted_extent_reference(generation: u64) -> GenerationCountedPhysicalReference {
    let generations = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = worth_store_physical_format::PhysicalReferenceAuthority::for_canonical_physical_format();
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(29).unwrap();
    let extent = worth_store_physical_format::PhysicalExtentId::from_raw(31).unwrap();
    let cell = generations
        .extent_cell(segment, extent)
        .with_extent_generation(worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap());
    worth_store_physical_isolation::GenerationCountedPhysicalReference::from_admitted_reference(references.admit_extent(cell))
}