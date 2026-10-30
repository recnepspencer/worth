#[allow(unused_imports)]
use crate::tier_movement_future_chunk_stability::*;
use worth_store_test_support::harness::physical_isolation::epoch_scope as support;

use {
    worth_foundational::FoundationalBoundaryArtifactRole,
    worth_foundational::FoundationalDiagnosticArtifactKind,
    worth_foundational::FoundationalDiagnosticBreachClass,
};
use {
    worth_store_physical_format::PhysicalExtentId,
    worth_store_physical_format::PhysicalFutureChunkId,
    worth_store_physical_format::PhysicalFutureChunkReference,
    worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalReferenceAuthority,
    worth_store_physical_format::PhysicalSegmentId,
};
use {
    worth_store_physical_isolation::admit_seed_stable_read_plan,
    worth_store_physical_isolation::ChunkMigrationReadInterlockPlan,
    worth_store_physical_isolation::FutureBlobMigrationNonClaim,
    worth_store_physical_isolation::FutureChunkStabilityBasis,
    worth_store_physical_isolation::GenerationCountedPhysicalReference,
    worth_store_physical_isolation::MovablePhysicalRef,
    worth_store_physical_isolation::PhysicalChunkStabilityPlaceholder,
    worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics,
    worth_store_physical_isolation::ProtectedPhysicalReferenceSet,
    worth_store_physical_isolation::PublishedReaderHazard,
    worth_store_physical_isolation::ReadPlanAdmissionScratchArena,
    worth_store_physical_isolation::StablePhysicalReadPlan,
    worth_store_physical_isolation::TierMovementAdmissionLabel,
    worth_store_physical_isolation::TierMovementReadInterlockPlan,
    worth_store_physical_isolation::TierMovementStabilityDenial,
    worth_store_physical_isolation::TierMovementStabilityVerdict,
    worth_store_physical_isolation::TraversalAdmissionGuard,
    worth_store_physical_isolation::UnprotectedReadIntent,
    worth_store_physical_isolation::UnsupportedTierMovementClaim,
    worth_store_physical_isolation::UnsupportedTierMovementRequest,
};
use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
};

#[test]
fn tier_and_future_chunk_plans_preserve_stability_only_facts() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let plan = ordinary_plan();
    let barrier = plan.reachability_barrier();
    let extent = current_generation_extent_reference(17);
    let extent_epoch = root.admit_extent_publication_epoch(extent).unwrap().epoch();
    let movable_extent =
        worth_store_physical_isolation::MovablePhysicalRef::extent(extent, extent_epoch, barrier)
            .unwrap();

    let first =
        worth_store_physical_isolation::TierMovementReadInterlockPlan::admit(movable_extent)
            .unwrap();
    let second =
        worth_store_physical_isolation::TierMovementReadInterlockPlan::admit(movable_extent)
            .unwrap();

    assert_eq!(first.reference(), second.reference());
    assert_eq!(
        first.verdict(),
        worth_store_physical_isolation::TierMovementStabilityVerdict::StabilityOnly
    );
    assert_eq!(
        first.require_cold_tier_qos(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::ColdTierQosRemainsS6Scope)
    );
    assert_eq!(
        first.non_claims().blob_lifecycle(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S7OwnsBlobLifecycle
    );

    let placeholder = future_chunk_placeholder(barrier);
    let equivalent_placeholder =
        future_chunk_placeholder_with_reference(placeholder.reference(), barrier);
    let chunk_plan =
        worth_store_physical_isolation::ChunkMigrationReadInterlockPlan::admit(placeholder)
            .unwrap();
    let equivalent_chunk_plan =
        worth_store_physical_isolation::ChunkMigrationReadInterlockPlan::admit(
            equivalent_placeholder,
        )
        .unwrap();

    assert_eq!(
        chunk_plan.non_claim().blob_retention(),
        worth_store_physical_isolation::FutureBlobMigrationNonClaim::S7OwnsBlobRetention
    );
    assert_equivalent_future_chunk_stability(chunk_plan, placeholder);
    assert_equivalent_future_chunk_stability(equivalent_chunk_plan, equivalent_placeholder);
    assert_eq!(
        chunk_plan.placeholder(),
        equivalent_chunk_plan.placeholder()
    );
}

#[test]
fn invalid_migration_inputs_deny_before_stability_admission() {
    let barrier = ordinary_plan().reachability_barrier();
    let chunk = future_chunk_reference(7, 3);
    let missing_epoch_basis =
        worth_store_physical_isolation::FutureChunkStabilityBasis::from_stability_receipt(
            chunk,
            future_chunk_epoch(),
            barrier,
        );
    let missing_epoch = worth_store_physical_isolation::PhysicalChunkStabilityPlaceholder::admit(
        chunk,
        None,
        missing_epoch_basis,
    );
    assert_eq!(
        missing_epoch.unwrap_err(),
        worth_store_physical_isolation::TierMovementStabilityDenial::MissingChunkEpoch
    );
    let copied_basis =
        worth_store_physical_isolation::FutureChunkStabilityBasis::from_stability_receipt(
            future_chunk_reference(8, 3),
            future_chunk_epoch(),
            barrier,
        );
    let mismatched_basis =
        worth_store_physical_isolation::PhysicalChunkStabilityPlaceholder::admit_with_epoch(
            chunk,
            future_chunk_epoch(),
            copied_basis,
        );
    assert_eq!(
        mismatched_basis.unwrap_err(),
        worth_store_physical_isolation::TierMovementStabilityDenial::PlaceholderBasisMismatch
    );

    let stale_extent =
        worth_store_physical_isolation::MovablePhysicalRef::extent_from_generation_counted(
            generation_counted_extent_reference(11),
            worth_store_physical_format::PhysicalGeneration::from_raw(12).unwrap(),
            current_root_from_authority(&physical_authority_from_complete_closeout())
                .admit_extent_publication_epoch(current_generation_extent_reference(11))
                .unwrap()
                .epoch(),
            barrier,
        );
    assert!(matches!(
        stale_extent,
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::StaleGeneration(_))
    ));

    let extent = movable_extent(barrier, 17);
    let chunk_ref =
        worth_store_physical_isolation::MovablePhysicalRef::future_chunk_from_placeholder(
            future_chunk_placeholder_with_reference(chunk, barrier),
        );
    let copied = worth_store_physical_isolation::TierMovementReadInterlockPlan::admit_with_label(
        chunk_ref,
        worth_store_physical_isolation::TierMovementAdmissionLabel::copied_from(extent),
    );
    assert_eq!(
        copied.unwrap_err(),
        worth_store_physical_isolation::TierMovementStabilityDenial::CopiedMigrationLabel
    );
    for reference_kind in [
        worth_store_physical_isolation::MovablePhysicalRefKind::Extent,
        worth_store_physical_isolation::MovablePhysicalRefKind::FutureChunk,
    ] {
        for claim in [
            worth_store_physical_isolation::UnsupportedTierMovementClaim::ColdTierQos,
            worth_store_physical_isolation::UnsupportedTierMovementClaim::HardwareMediaPlacement,
            worth_store_physical_isolation::UnsupportedTierMovementClaim::BlobLifecycleMigration,
        ] {
            let unsupported = worth_store_physical_isolation::UnsupportedTierMovementRequest::new(
                reference_kind,
                claim,
            );
            assert_eq!(unsupported.reference_kind(), reference_kind);
            assert_eq!(unsupported.claim(), claim);
            assert_eq!(
                worth_store_physical_isolation::TierMovementReadInterlockPlan::reject_unsupported_tier_movement(unsupported)
                    .unwrap_err(),
                worth_store_physical_isolation::TierMovementStabilityDenial::UnsupportedTierMovement
            );
        }
    }
}

#[test]
fn blob_lifecycle_and_io_qos_authority_requests_remain_explicit_non_claims() {
    let chunk_plan = worth_store_physical_isolation::ChunkMigrationReadInterlockPlan::admit(
        future_chunk_placeholder(ordinary_plan().reachability_barrier()),
    )
    .unwrap();
    let evidence = chunk_plan.foundational_non_claim_evidence();

    assert_eq!(
        evidence.support().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::SupportOnly
    );
    assert_eq!(
        evidence.planned().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::PlannedWork
    );
    assert_eq!(
        evidence.artifact_kind(),
        worth_foundational::FoundationalDiagnosticArtifactKind::SupportReport
    );
    assert_eq!(
        evidence.breach_class(),
        worth_foundational::FoundationalDiagnosticBreachClass::CoverageOmission
    );
    assert_eq!(
        evidence.deny_blob_authority_promotion(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::FoundationalSurfaceCannotPromoteToBlobAuthority)
    );
    assert_eq!(
        evidence.deny_cold_tier_qos_promotion(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::ColdTierQosRemainsS6Scope)
    );
    assert_eq!(
        chunk_plan.deny_proof_assumption_blob_authority_promotion(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::ProofAssumptionCannotPromoteToBlobAuthority)
    );
    assert_eq!(
        chunk_plan.deny_proof_assumption_cold_tier_qos_promotion(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::ProofAssumptionCannotPromoteToColdTierQos)
    );
    assert_non_claim_report(chunk_plan.non_claim());
    assert_eq!(
        chunk_plan.require_blob_lifecycle_authority(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::BlobLifecycleRemainsS7Scope)
    );
    assert_eq!(
        chunk_plan.require_blob_retention_authority(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::BlobRetentionRemainsS7Scope)
    );
    assert_eq!(
        chunk_plan.require_blob_dedupe_authority(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::BlobDedupeRemainsS7Scope)
    );
    assert_eq!(
        chunk_plan.require_resumable_write_authority(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::ResumableWritesRemainS7Scope)
    );
    assert_eq!(
        chunk_plan.require_cold_tier_qos(),
        Err(worth_store_physical_isolation::TierMovementStabilityDenial::ColdTierQosRemainsS6Scope)
    );
}

#[test]
fn ordinary_read_plans_do_not_carry_future_chunk_fields() {
    let plan = ordinary_plan();

    assert!(plan.epoch_vector().chunk_epoch().is_none());
    assert_eq!(plan.footprint().protected().references().len(), 3);
    let placeholder = future_chunk_placeholder_with_reference(
        future_chunk_reference(9, 4),
        plan.reachability_barrier(),
    );
    assert!(matches!(
        worth_store_physical_isolation::MovablePhysicalRef::future_chunk_from_placeholder(
            placeholder
        )
        .kind(),
        worth_store_physical_isolation::MovablePhysicalRefKind::FutureChunk
    ));
}
