#[allow(unused_imports)] use crate::certification::ordinary_context::boundary_evidence::lineage::*;
use {worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path, worth_foundational::boundary_evidence_api::lower_lane, worth_foundational::foundational_boundary_evidence_branch_divergence_posture_definitions, worth_foundational::foundational_boundary_evidence_lineage_outcome_kind_definitions, worth_foundational::foundational_boundary_evidence_lineage_partiality_posture_definitions, worth_foundational::foundational_boundary_evidence_promotion_posture_definitions, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle, worth_foundational::FoundationalBoundaryEvidenceBranchDivergencePosture, worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture, worth_foundational::FoundationalBoundaryEvidenceLineageConstructionDenial, worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind, worth_foundational::FoundationalBoundaryEvidenceLineagePartialityPosture, worth_foundational::FoundationalBoundaryEvidenceLineageSubject, worth_foundational::FoundationalBoundaryEvidenceLineageSubjectSet, worth_foundational::FoundationalBoundaryEvidenceLocality, worth_foundational::FoundationalBoundaryEvidencePromotionPosture, worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary, worth_foundational::FoundationalBoundaryEvidenceReceiptKind, worth_foundational::FoundationalBoundaryEvidenceSourceBasis, worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalTransitionLocator};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::boundary_evidence::provenance::authority_path, crate::certification::ordinary_context::boundary_evidence::provenance::digest_basis, crate::certification::ordinary_context::boundary_evidence::provenance::profile_basis, crate::certification::ordinary_context::boundary_evidence::provenance::source_basis, crate::certification::ordinary_context::boundary_evidence::provenance::strategy_basis};

#[test]
fn lineage_definitions_are_blind_consumer_interpretable() {
    let kinds = foundational_boundary_evidence_lineage_outcome_kind_definitions();
    let divergence = foundational_boundary_evidence_branch_divergence_posture_definitions();
    let promotion = foundational_boundary_evidence_promotion_posture_definitions();
    let partiality = foundational_boundary_evidence_lineage_partiality_posture_definitions();

    assert_eq!(kinds.len(), 13);
    assert_eq!(
        divergence
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec!["branch_local_only", "superseded_before_promotion"]
    );
    assert_eq!(
        promotion
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec!["promoted_to_global_continuity", "promotion_denied"]
    );
    assert_eq!(
        partiality
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec!["named_gap", "withheld_redacted", "denied"]
    );
    assert!(kinds
        .iter()
        .all(|definition| !definition.must_not_mean().trim().is_empty()));
}

#[test]
fn attested_and_replay_derived_continuity_stay_distinct() {
    let attested = worth_foundational::boundary_evidence_api::common_path::lineage().continuity(subject(1)).attested_by(
        boundary_evidence()
            .receipt()
            .execution(receipt_boundary(1))
            .with_provenance(historical_provenance()),
    );
    let replay = boundary_evidence()
        .lineage()
        .replay_derived_continuity(subject(1))
        .with_provenance(replay_provenance())
        .expect_success("replay lineage");

    assert_eq!(
        attested.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity
    );
    assert_eq!(
        attested.executed_receipt().receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Execution
    );
    assert_eq!(
        replay.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity
    );
    assert_eq!(
        replay.provenance().locality(),
        worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived
    );
}

#[test]
fn restored_continuity_and_reconstructed_equivalence_stay_distinct() {
    let restored = boundary_evidence()
        .lineage()
        .restored_continuity(subject(2))
        .attested_by(
            boundary_evidence()
                .receipt()
                .restoration(receipt_boundary(2))
                .with_provenance(restored_provenance()),
        )
        .expect_success("restored continuity");
    let reconstructed = worth_foundational::boundary_evidence_api::common_path::lineage()
        .reconstructed_equivalence(subject(2))
        .with_provenance(replay_provenance())
        .expect_success("reconstructed equivalence");

    assert_eq!(
        restored.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::RestoredContinuity
    );
    assert_eq!(
        restored.restoration_receipt().receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Restoration
    );
    assert_eq!(
        reconstructed.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::ReconstructedEquivalence
    );
    assert_eq!(
        reconstructed.provenance().locality(),
        worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived
    );
}

#[test]
fn branch_local_replacement_and_promotion_posture_are_explicit() {
    let branch_local = boundary_evidence()
        .lineage()
        .branch_local_replacement(subject(3))
        .with_divergence(worth_foundational::FoundationalBoundaryEvidenceBranchDivergencePosture::BranchLocalOnly)
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(3))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .branch_local(source_basis())
                        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
                        .expect_success("branch local provenance"),
                ),
        );

    let promoted = branch_local
        .clone()
        .promote_with(
            worth_foundational::FoundationalBoundaryEvidencePromotionPosture::PromotedToGlobalContinuity,
            boundary_evidence()
                .receipt()
                .publication(receipt_boundary(4))
                .with_provenance(historical_provenance()),
        )
        .expect_success("promoted continuity");

    assert_eq!(
        branch_local.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::BranchLocalReplacement
    );
    assert_eq!(
        promoted.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity
    );
    assert_eq!(
        promoted.promotion_posture(),
        worth_foundational::FoundationalBoundaryEvidencePromotionPosture::PromotedToGlobalContinuity
    );
    assert_eq!(
        promoted.divergence_posture(),
        worth_foundational::FoundationalBoundaryEvidenceBranchDivergencePosture::BranchLocalOnly
    );
}

#[test]
fn partiality_and_transient_outcomes_are_explicit() {
    let partial = boundary_evidence()
        .lineage()
        .named_gap_partial_continuity(subject(5))
        .with_provenance(historical_provenance());
    let transient = worth_foundational::boundary_evidence_api::common_path::lineage()
        .transient_within_boundary_closure(subject(6))
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(6))
                .with_provenance(replay_provenance()),
        );

    assert_eq!(
        partial.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::NamedGapPartialContinuity
    );
    assert_eq!(
        partial.partiality_posture(),
        worth_foundational::FoundationalBoundaryEvidenceLineagePartialityPosture::NamedGap
    );
    assert_eq!(
        transient.outcome_kind(),
        worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind::TransientWithinBoundaryClosure
    );
    assert!(transient.executed_receipt().did_execute());
}

#[test]
fn attested_lineage_semantics_are_stable_across_common_and_direct_construction_paths() {
    let common_path_lineage = worth_foundational::boundary_evidence_api::common_path::lineage().continuity(subject(10)).attested_by(
        boundary_evidence()
            .receipt()
            .execution(receipt_boundary(10))
            .with_provenance(historical_provenance()),
    );
    let direct_path_lineage = boundary_evidence()
        .lineage()
        .continuity(subject(10))
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(10))
                .with_provenance(historical_provenance()),
        );

    assert_eq!(common_path_lineage, direct_path_lineage);
}

#[test]
fn lineage_construction_denials_are_explicit_and_fail_closed() {
    assert_eq!(
        boundary_evidence()
            .lineage()
            .replay_derived_continuity(subject(7))
            .with_provenance(historical_provenance()),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceLineageConstructionDenial::ReplayDerivedContinuityRequiresReplayDerivedProvenance
        )
    );
    assert_eq!(
        boundary_evidence()
            .lineage()
            .restored_continuity(subject(7))
            .attested_by(
                boundary_evidence()
                    .receipt()
                    .execution(receipt_boundary(7))
                    .with_provenance(historical_provenance()),
            ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceLineageConstructionDenial::RestoredContinuityRequiresRestorationOrCheckpointReceipt
        )
    );
    assert_eq!(
        boundary_evidence()
            .lineage()
            .reconstructed_equivalence(subject(7))
            .with_provenance(historical_provenance()),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceLineageConstructionDenial::ReconstructedEquivalenceRequiresReplayOrRestoredProvenance
        )
    );
    let branch_local = boundary_evidence()
        .lineage()
        .branch_local_replacement(subject(8))
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(8))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .branch_local(source_basis())
                        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
                        .expect_success("branch local provenance"),
                ),
        );
    assert_eq!(
        branch_local.promote_with(
            worth_foundational::FoundationalBoundaryEvidencePromotionPosture::PromotionDenied,
            boundary_evidence()
                .receipt()
                .publication(receipt_boundary(9))
                .with_provenance(historical_provenance()),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceLineageConstructionDenial::PromotionDeniedDoesNotProduceGlobalContinuity
        )
    );
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase4_surface() {
    assert_eq!(
        boundary_evidence().lineage_outcome_kind_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::lineage::foundational_boundary_evidence_lineage_outcome_kind_definitions()
    );
    assert_eq!(
        boundary_evidence().branch_divergence_posture_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::lineage::foundational_boundary_evidence_branch_divergence_posture_definitions()
    );
    assert_eq!(
        boundary_evidence().promotion_posture_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::lineage::foundational_boundary_evidence_promotion_posture_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceLineageFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::lineage();
    let _related = worth_foundational::FoundationalBoundaryEvidenceLineageSubjectSet::new(vec![subject(1), subject(2)])
        .expect("related set");
}
