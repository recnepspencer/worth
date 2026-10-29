use worth_foundational::{
    boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane},
    foundational_boundary_evidence_support_basis_disclosure_definitions,
    foundational_boundary_evidence_support_recovery_posture_definitions,
    foundational_boundary_evidence_support_residual_debt_kind_definitions,
    foundational_boundary_evidence_support_truth_kind_definitions, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceLineageSubject,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryEvidenceSupportBasisDisclosure,
    FoundationalBoundaryEvidenceSupportConstructionDenial,
    FoundationalBoundaryEvidenceSupportRecoveryPosture,
    FoundationalBoundaryEvidenceSupportResidualDebtKind,
    FoundationalBoundaryEvidenceSupportResidualDebtSet,
    FoundationalBoundaryEvidenceSupportTruthKind, FoundationalCommitId,
    FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalTransitionLocator,
};
use worth_proof::TransitionOutcome;

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

fn source_basis() -> FoundationalBoundaryEvidenceSourceBasis {
    FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(90),
        BoundaryArtifactField::Basis,
    ))
}

fn replay_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .replay_derived(source_basis())
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
        .expect_success("replay provenance")
}

fn historical_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .historical(source_basis())
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
        .expect_success("historical provenance")
}

fn residual_debt() -> FoundationalBoundaryEvidenceSupportResidualDebtSet {
    FoundationalBoundaryEvidenceSupportResidualDebtSet::new(vec![
        FoundationalBoundaryEvidenceSupportResidualDebtKind::RebuildRequired,
        FoundationalBoundaryEvidenceSupportResidualDebtKind::ReducedBasisLimitsParity,
    ])
    .expect("residual debt set")
}

#[test]
fn support_definitions_are_blind_consumer_interpretable() {
    assert_eq!(
        foundational_boundary_evidence_support_truth_kind_definitions().len(),
        7
    );
    assert_eq!(
        foundational_boundary_evidence_support_recovery_posture_definitions().len(),
        4
    );
    assert_eq!(
        foundational_boundary_evidence_support_basis_disclosure_definitions().len(),
        4
    );
    assert_eq!(
        foundational_boundary_evidence_support_residual_debt_kind_definitions().len(),
        4
    );
}

#[test]
fn published_support_requires_support_publication_and_exposes_provenance() {
    let support = common_path::support()
        .published_evidence()
        .with_basis_disclosure(
            FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis,
        )
        .with_recovery_posture(
            FoundationalBoundaryEvidenceSupportRecoveryPosture::ReplayReconstructed,
        )
        .attested_by(
            boundary_evidence()
                .receipt()
                .support_publication(receipt_boundary(1))
                .with_provenance(replay_provenance()),
        )
        .expect_success("published support");

    assert_eq!(
        support.support_truth_kind(),
        FoundationalBoundaryEvidenceSupportTruthKind::EvidenceBundle
    );
    assert_eq!(
        support.provenance().locality(),
        worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived
    );
}

#[test]
fn support_closeout_preserves_completed_boundary_truth_without_execution() {
    let closeout = boundary_evidence()
        .support()
        .degraded_recovery_report()
        .with_basis_disclosure(FoundationalBoundaryEvidenceSupportBasisDisclosure::StaleBasis)
        .closed_out_by(
            boundary_evidence()
                .receipt()
                .blocked_closeout(receipt_boundary(2))
                .with_provenance(historical_provenance()),
        )
        .expect_success("support closeout");

    assert_eq!(
        closeout.support_truth_kind(),
        FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
    );
    assert!(!closeout.closeout_receipt().did_execute());
}

#[test]
fn rebuild_and_quarantine_support_require_residual_debt() {
    assert_eq!(
        boundary_evidence()
            .support()
            .published_evidence()
            .with_basis_disclosure(FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis)
            .with_recovery_posture(FoundationalBoundaryEvidenceSupportRecoveryPosture::RebuildRequired)
            .attested_by(
                boundary_evidence()
                    .receipt()
                    .support_publication(receipt_boundary(3))
                    .with_provenance(historical_provenance()),
            ),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceSupportConstructionDenial::RebuildRequiredSupportRequiresResidualDebt
        )
    );
    assert_eq!(
        boundary_evidence()
            .support()
            .degraded_recovery_report()
            .with_basis_disclosure(FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis)
            .with_recovery_posture(FoundationalBoundaryEvidenceSupportRecoveryPosture::Quarantined)
            .closed_out_by(
                boundary_evidence()
                    .receipt()
                    .denied_closeout(receipt_boundary(4))
                    .with_provenance(historical_provenance()),
            ),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceSupportConstructionDenial::QuarantinedSupportRequiresResidualDebt
        )
    );
}

#[test]
fn transient_lifecycle_support_is_support_grade_and_subject_explicit() {
    let transient = boundary_evidence()
        .support()
        .transient_lifecycle(FoundationalBoundaryEvidenceLineageSubject::new(
            BoundaryHandle::new(7),
        ))
        .with_basis_disclosure(FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis)
        .with_residual_debt(residual_debt())
        .opened_and_closed_within(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(5))
                .with_provenance(replay_provenance()),
        );

    assert_eq!(
        transient.support_truth_kind(),
        FoundationalBoundaryEvidenceSupportTruthKind::TransientLifecycleEvidence
    );
    assert_eq!(transient.subject().handle(), BoundaryHandle::new(7));
}

#[test]
fn published_support_semantics_are_stable_across_common_and_direct_construction_paths() {
    let common_path_support = common_path::support()
        .published_evidence()
        .with_basis_disclosure(
            FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis,
        )
        .with_recovery_posture(
            FoundationalBoundaryEvidenceSupportRecoveryPosture::ReplayReconstructed,
        )
        .attested_by(
            boundary_evidence()
                .receipt()
                .support_publication(receipt_boundary(11))
                .with_provenance(replay_provenance()),
        )
        .expect_success("common path support");
    let direct_path_support = boundary_evidence()
        .support()
        .published_evidence()
        .with_basis_disclosure(
            FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis,
        )
        .with_recovery_posture(
            FoundationalBoundaryEvidenceSupportRecoveryPosture::ReplayReconstructed,
        )
        .attested_by(
            boundary_evidence()
                .receipt()
                .support_publication(receipt_boundary(11))
                .with_provenance(replay_provenance()),
        )
        .expect_success("direct path support");

    assert_eq!(common_path_support, direct_path_support);
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase5_surface() {
    assert_eq!(
        boundary_evidence().support_truth_kind_definitions(),
        lower_lane::support::foundational_boundary_evidence_support_truth_kind_definitions()
    );
    assert_eq!(
        boundary_evidence().support_recovery_posture_definitions(),
        lower_lane::support::foundational_boundary_evidence_support_recovery_posture_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceSupportFrontDoor =
        common_path::support();
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
