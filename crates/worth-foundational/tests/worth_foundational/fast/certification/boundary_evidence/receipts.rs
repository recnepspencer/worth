use worth_foundational::{
    boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane},
    foundational_boundary_evidence_closeout_disposition_definitions,
    foundational_boundary_evidence_receipt_kind_definitions, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    FoundationalBoundaryEvidenceCloseoutDisposition, FoundationalBoundaryEvidenceExecutionPosture,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceReceiptKind,
    FoundationalBoundaryEvidenceSourceBasis, FoundationalCommitId, FoundationalCommitParentBasis,
    FoundationalCommitParentageLocator, FoundationalTransitionLocator,
};

use super::provenance::{
    authority_path, digest_basis, profile_basis, source_basis, strategy_basis,
};

fn receipt_boundary() -> FoundationalBoundaryEvidenceReceiptBoundary {
    FoundationalBoundaryEvidenceReceiptBoundary::transition(
        FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
            FoundationalCommitId::new(BoundaryHandle::new(91)),
            FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(17)),
        )),
    )
}

fn historical_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    common_path::provenance()
        .historical(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .canonical_digest_basis(digest_basis())
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
        .expect_success("historical provenance")
}

#[test]
fn receipt_kind_definitions_are_blind_consumer_interpretable() {
    let kinds = foundational_boundary_evidence_receipt_kind_definitions();
    let closeout = foundational_boundary_evidence_closeout_disposition_definitions();

    assert_eq!(
        kinds
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec![
            "admission",
            "planning",
            "execution",
            "publication",
            "restoration",
            "support_publication",
            "checkpoint_resume",
            "closeout",
        ]
    );
    assert_eq!(
        closeout
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec!["blocked", "denied"]
    );
    assert!(kinds
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
}

#[test]
fn planning_and_completed_receipts_stay_family_distinct() {
    let provenance = common_path::provenance()
        .historical(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .canonical_digest_basis(digest_basis())
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
        .expect_success("provenance");

    let planning = common_path::receipt()
        .planning(receipt_boundary())
        .with_provenance(provenance.clone());
    let execution = common_path::receipt()
        .execution(receipt_boundary())
        .with_provenance(provenance);

    assert_eq!(
        planning.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Planning
    );
    assert_eq!(
        planning.execution_posture(),
        FoundationalBoundaryEvidenceExecutionPosture::Planned
    );
    assert_eq!(
        execution.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Execution
    );
    assert_eq!(
        execution.execution_posture(),
        FoundationalBoundaryEvidenceExecutionPosture::Executed
    );
    assert!(execution.did_execute());
}

#[test]
fn blocked_and_denied_closeout_receipts_preserve_completed_boundary_truth_without_execution() {
    let provenance = boundary_evidence()
        .provenance()
        .replay_derived(source_basis())
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
        .expect_success("provenance");

    let blocked = boundary_evidence()
        .receipt()
        .blocked_closeout(receipt_boundary())
        .with_provenance(provenance.clone());
    let denied = boundary_evidence()
        .receipt()
        .denied_closeout(receipt_boundary())
        .with_provenance(provenance);

    assert_eq!(
        blocked.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Closeout
    );
    assert_eq!(
        blocked.closeout_disposition(),
        Some(FoundationalBoundaryEvidenceCloseoutDisposition::Blocked)
    );
    assert_eq!(
        blocked.execution_posture(),
        FoundationalBoundaryEvidenceExecutionPosture::NotExecuted
    );
    assert!(!blocked.did_execute());
    assert_eq!(
        denied.closeout_disposition(),
        Some(FoundationalBoundaryEvidenceCloseoutDisposition::Denied)
    );
    assert_eq!(
        denied.execution_posture(),
        FoundationalBoundaryEvidenceExecutionPosture::NotExecuted
    );
    assert!(!denied.did_execute());
    assert_eq!(blocked.completed_boundary(), denied.completed_boundary());
}

#[test]
fn completed_receipt_families_are_explicit_and_stable() {
    let provenance = boundary_evidence()
        .provenance()
        .snapshot_bound(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            BoundaryArtifactLocator::new(BoundaryArtifactId::new(12), BoundaryArtifactField::Basis),
        ))
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained,
        )
        .expect_success("provenance");

    let admission = boundary_evidence()
        .receipt()
        .admission(receipt_boundary())
        .with_provenance(provenance.clone());
    let publication = boundary_evidence()
        .receipt()
        .publication(receipt_boundary())
        .with_provenance(provenance.clone());
    let restoration = boundary_evidence()
        .receipt()
        .restoration(receipt_boundary())
        .with_provenance(provenance.clone());
    let support_publication = boundary_evidence()
        .receipt()
        .support_publication(receipt_boundary())
        .with_provenance(provenance.clone());
    let checkpoint_resume = boundary_evidence()
        .receipt()
        .checkpoint_resume(receipt_boundary())
        .with_provenance(provenance);

    assert_eq!(
        admission.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Admission
    );
    assert_eq!(
        publication.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Publication
    );
    assert_eq!(
        restoration.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Restoration
    );
    assert_eq!(
        support_publication.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::SupportPublication
    );
    assert_eq!(
        checkpoint_resume.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::CheckpointResume
    );
}

#[test]
fn executed_receipt_semantics_are_stable_across_common_and_direct_construction_paths() {
    let common_path_receipt = common_path::receipt()
        .execution(receipt_boundary())
        .with_provenance(historical_provenance());
    let direct_path_receipt = boundary_evidence()
        .receipt()
        .execution(receipt_boundary())
        .with_provenance(historical_provenance());

    assert_eq!(common_path_receipt, direct_path_receipt);
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase3_surface() {
    assert_eq!(
        boundary_evidence().receipt_kind_definitions(),
        lower_lane::receipts::foundational_boundary_evidence_receipt_kind_definitions()
    );
    assert_eq!(
        boundary_evidence().closeout_disposition_definitions(),
        lower_lane::receipts::foundational_boundary_evidence_closeout_disposition_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceReceiptFrontDoor =
        common_path::receipt();
}

trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for worth_proof::TransitionOutcome<T, E>
where
    E: core::fmt::Debug,
{
    fn expect_success(self, label: &str) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}
