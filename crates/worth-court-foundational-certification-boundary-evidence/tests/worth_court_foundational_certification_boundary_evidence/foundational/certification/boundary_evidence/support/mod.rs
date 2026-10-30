#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_evidence::support::*;
use {
    worth_foundational::boundary_evidence,
    worth_foundational::foundational_boundary_evidence_support_basis_disclosure_definitions,
    worth_foundational::foundational_boundary_evidence_support_recovery_posture_definitions,
    worth_foundational::foundational_boundary_evidence_support_residual_debt_kind_definitions,
    worth_foundational::foundational_boundary_evidence_support_truth_kind_definitions,
};

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
    let support = worth_foundational::boundary_evidence_api::common_path::support()
        .published_evidence()
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis,
        )
        .with_recovery_posture(
            worth_foundational::FoundationalBoundaryEvidenceSupportRecoveryPosture::ReplayReconstructed,
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
        worth_foundational::FoundationalBoundaryEvidenceSupportTruthKind::EvidenceBundle
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
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::StaleBasis,
        )
        .closed_out_by(
            boundary_evidence()
                .receipt()
                .blocked_closeout(receipt_boundary(2))
                .with_provenance(historical_provenance()),
        )
        .expect_success("support closeout");

    assert_eq!(
        closeout.support_truth_kind(),
        worth_foundational::FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
    );
    assert!(!closeout.closeout_receipt().did_execute());
}

#[test]
fn rebuild_and_quarantine_support_require_residual_debt() {
    assert_eq!(
        boundary_evidence()
            .support()
            .published_evidence()
            .with_basis_disclosure(worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis)
            .with_recovery_posture(worth_foundational::FoundationalBoundaryEvidenceSupportRecoveryPosture::RebuildRequired)
            .attested_by(
                boundary_evidence()
                    .receipt()
                    .support_publication(receipt_boundary(3))
                    .with_provenance(historical_provenance()),
            ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceSupportConstructionDenial::RebuildRequiredSupportRequiresResidualDebt
        )
    );
    assert_eq!(
        boundary_evidence()
            .support()
            .degraded_recovery_report()
            .with_basis_disclosure(worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis)
            .with_recovery_posture(worth_foundational::FoundationalBoundaryEvidenceSupportRecoveryPosture::Quarantined)
            .closed_out_by(
                boundary_evidence()
                    .receipt()
                    .denied_closeout(receipt_boundary(4))
                    .with_provenance(historical_provenance()),
            ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceSupportConstructionDenial::QuarantinedSupportRequiresResidualDebt
        )
    );
}

#[test]
fn transient_lifecycle_support_is_support_grade_and_subject_explicit() {
    let transient = boundary_evidence()
        .support()
        .transient_lifecycle(
            worth_foundational::FoundationalBoundaryEvidenceLineageSubject::new(
                worth_foundational::BoundaryHandle::new(7),
            ),
        )
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis,
        )
        .with_residual_debt(residual_debt())
        .opened_and_closed_within(
            boundary_evidence()
                .receipt()
                .execution(receipt_boundary(5))
                .with_provenance(replay_provenance()),
        );

    assert_eq!(
        transient.support_truth_kind(),
        worth_foundational::FoundationalBoundaryEvidenceSupportTruthKind::TransientLifecycleEvidence
    );
    assert_eq!(
        transient.subject().handle(),
        worth_foundational::BoundaryHandle::new(7)
    );
}

#[test]
fn published_support_semantics_are_stable_across_common_and_direct_construction_paths() {
    let common_path_support = worth_foundational::boundary_evidence_api::common_path::support()
        .published_evidence()
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis,
        )
        .with_recovery_posture(
            worth_foundational::FoundationalBoundaryEvidenceSupportRecoveryPosture::ReplayReconstructed,
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
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedAndStaleBasis,
        )
        .with_recovery_posture(
            worth_foundational::FoundationalBoundaryEvidenceSupportRecoveryPosture::ReplayReconstructed,
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
fn common_path_and_lower_lane_expose_the_same_surface() {
    assert_eq!(
        boundary_evidence().support_truth_kind_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::support::foundational_boundary_evidence_support_truth_kind_definitions()
    );
    assert_eq!(
        boundary_evidence().support_recovery_posture_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::support::foundational_boundary_evidence_support_recovery_posture_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceSupportFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::support();
}
