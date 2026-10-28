use worth_foundational::{
    boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane},
    foundational_boundary_evidence_branch_divergence_posture_definitions,
    foundational_boundary_evidence_lineage_outcome_kind_definitions,
    foundational_boundary_evidence_lineage_partiality_posture_definitions,
    foundational_boundary_evidence_promotion_posture_definitions, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    FoundationalBoundaryEvidenceBranchDivergencePosture,
    FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageConstructionDenial,
    FoundationalBoundaryEvidenceLineageOutcomeKind,
    FoundationalBoundaryEvidenceLineagePartialityPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceLineageSubjectSet,
    FoundationalBoundaryEvidenceLocality, FoundationalBoundaryEvidencePromotionPosture,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceReceiptKind,
    FoundationalBoundaryEvidenceSourceBasis, FoundationalCommitId, FoundationalCommitParentBasis,
    FoundationalCommitParentageLocator, FoundationalTransitionLocator,
};
use worth_proof::TransitionOutcome;

use super::provenance::{
    authority_path, digest_basis, profile_basis, source_basis, strategy_basis,
};

fn receipt_boundary(seed: u64) -> FoundationalBoundaryEvidenceReceiptBoundary {
    FoundationalBoundaryEvidenceReceiptBoundary::transition(
        FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
            FoundationalCommitId::new(BoundaryHandle::new(seed)),
            FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(
                seed + 10,
            )),
        )),
    )
}

fn subject(seed: u64) -> FoundationalBoundaryEvidenceLineageSubject {
    FoundationalBoundaryEvidenceLineageSubject::new(BoundaryHandle::new(seed))
}

fn historical_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    common_path::provenance()
        .historical(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .canonical_digest_basis(digest_basis())
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
        .expect_success("historical provenance")
}

fn replay_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .replay_derived(source_basis())
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
        .expect_success("replay provenance")
}

fn restored_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .restored_readmitted(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            BoundaryArtifactLocator::new(BoundaryArtifactId::new(77), BoundaryArtifactField::Basis),
        ))
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint)
        .expect_success("restored provenance")
}

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
    let attested = common_path::lineage().continuity(subject(1)).attested_by(
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
        FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity
    );
    assert_eq!(
        attested.executed_receipt().receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Execution
    );
    assert_eq!(
        replay.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity
    );
    assert_eq!(
        replay.provenance().locality(),
        FoundationalBoundaryEvidenceLocality::ReplayDerived
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
    let reconstructed = common_path::lineage()
        .reconstructed_equivalence(subject(2))
        .with_provenance(replay_provenance())
        .expect_success("reconstructed equivalence");

    assert_eq!(
        restored.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::RestoredContinuity
    );
    assert_eq!(
        restored.restoration_receipt().receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Restoration
    );
    assert_eq!(
        reconstructed.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::ReconstructedEquivalence
    );
    assert_eq!(
        reconstructed.provenance().locality(),
        FoundationalBoundaryEvidenceLocality::ReplayDerived
    );
}

#[test]
fn branch_local_replacement_and_promotion_posture_are_explicit() {
    let branch_local = boundary_evidence()
        .lineage()
        .branch_local_replacement(subject(3))
        .with_divergence(FoundationalBoundaryEvidenceBranchDivergencePosture::BranchLocalOnly)
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(3))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .branch_local(source_basis())
                        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
                        .expect_success("branch local provenance"),
                ),
        );

    let promoted = branch_local
        .clone()
        .promote_with(
            FoundationalBoundaryEvidencePromotionPosture::PromotedToGlobalContinuity,
            boundary_evidence()
                .receipt()
                .publication(receipt_boundary(4))
                .with_provenance(historical_provenance()),
        )
        .expect_success("promoted continuity");

    assert_eq!(
        branch_local.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::BranchLocalReplacement
    );
    assert_eq!(
        promoted.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::SingularContinuity
    );
    assert_eq!(
        promoted.promotion_posture(),
        FoundationalBoundaryEvidencePromotionPosture::PromotedToGlobalContinuity
    );
    assert_eq!(
        promoted.divergence_posture(),
        FoundationalBoundaryEvidenceBranchDivergencePosture::BranchLocalOnly
    );
}

#[test]
fn partiality_and_transient_outcomes_are_explicit() {
    let partial = boundary_evidence()
        .lineage()
        .named_gap_partial_continuity(subject(5))
        .with_provenance(historical_provenance());
    let transient = common_path::lineage()
        .transient_within_boundary_closure(subject(6))
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(6))
                .with_provenance(replay_provenance()),
        );

    assert_eq!(
        partial.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::NamedGapPartialContinuity
    );
    assert_eq!(
        partial.partiality_posture(),
        FoundationalBoundaryEvidenceLineagePartialityPosture::NamedGap
    );
    assert_eq!(
        transient.outcome_kind(),
        FoundationalBoundaryEvidenceLineageOutcomeKind::TransientWithinBoundaryClosure
    );
    assert!(transient.executed_receipt().did_execute());
}

#[test]
fn attested_lineage_semantics_are_stable_across_common_and_direct_construction_paths() {
    let common_path_lineage = common_path::lineage().continuity(subject(10)).attested_by(
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
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceLineageConstructionDenial::ReplayDerivedContinuityRequiresReplayDerivedProvenance
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
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceLineageConstructionDenial::RestoredContinuityRequiresRestorationOrCheckpointReceipt
        )
    );
    assert_eq!(
        boundary_evidence()
            .lineage()
            .reconstructed_equivalence(subject(7))
            .with_provenance(historical_provenance()),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceLineageConstructionDenial::ReconstructedEquivalenceRequiresReplayOrRestoredProvenance
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
                        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
                        .expect_success("branch local provenance"),
                ),
        );
    assert_eq!(
        branch_local.promote_with(
            FoundationalBoundaryEvidencePromotionPosture::PromotionDenied,
            boundary_evidence()
                .receipt()
                .publication(receipt_boundary(9))
                .with_provenance(historical_provenance()),
        ),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceLineageConstructionDenial::PromotionDeniedDoesNotProduceGlobalContinuity
        )
    );
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase4_surface() {
    assert_eq!(
        boundary_evidence().lineage_outcome_kind_definitions(),
        lower_lane::lineage::foundational_boundary_evidence_lineage_outcome_kind_definitions()
    );
    assert_eq!(
        boundary_evidence().branch_divergence_posture_definitions(),
        lower_lane::lineage::foundational_boundary_evidence_branch_divergence_posture_definitions()
    );
    assert_eq!(
        boundary_evidence().promotion_posture_definitions(),
        lower_lane::lineage::foundational_boundary_evidence_promotion_posture_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceLineageFrontDoor =
        common_path::lineage();
    let _related = FoundationalBoundaryEvidenceLineageSubjectSet::new(vec![subject(1), subject(2)])
        .expect("related set");
}

trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for TransitionOutcome<T, E> {
    fn expect_success(self, label: &str) -> T {
        match self {
            TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}
