#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_evidence::receipts::*;
use {
    worth_foundational::boundary_evidence,
    worth_foundational::foundational_boundary_evidence_closeout_disposition_definitions,
    worth_foundational::foundational_boundary_evidence_receipt_kind_definitions,
};

use {
    crate::certification::ordinary_context::boundary_evidence::provenance::authority_path,
    crate::certification::ordinary_context::boundary_evidence::provenance::digest_basis,
    crate::certification::ordinary_context::boundary_evidence::provenance::profile_basis,
    crate::certification::ordinary_context::boundary_evidence::provenance::source_basis,
    crate::certification::ordinary_context::boundary_evidence::provenance::strategy_basis,
};

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
    let provenance = worth_foundational::boundary_evidence_api::common_path::provenance()
        .historical(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .canonical_digest_basis(digest_basis())
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
        .expect_success("provenance");

    let planning = worth_foundational::boundary_evidence_api::common_path::receipt()
        .planning(receipt_boundary())
        .with_provenance(provenance.clone());
    let execution = worth_foundational::boundary_evidence_api::common_path::receipt()
        .execution(receipt_boundary())
        .with_provenance(provenance);

    assert_eq!(
        planning.receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Planning
    );
    assert_eq!(
        planning.execution_posture(),
        worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Planned
    );
    assert_eq!(
        execution.receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Execution
    );
    assert_eq!(
        execution.execution_posture(),
        worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Executed
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
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Closeout
    );
    assert_eq!(
        blocked.closeout_disposition(),
        Some(worth_foundational::FoundationalBoundaryEvidenceCloseoutDisposition::Blocked)
    );
    assert_eq!(
        blocked.execution_posture(),
        worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::NotExecuted
    );
    assert!(!blocked.did_execute());
    assert_eq!(
        denied.closeout_disposition(),
        Some(worth_foundational::FoundationalBoundaryEvidenceCloseoutDisposition::Denied)
    );
    assert_eq!(
        denied.execution_posture(),
        worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::NotExecuted
    );
    assert!(!denied.did_execute());
    assert_eq!(blocked.completed_boundary(), denied.completed_boundary());
}

#[test]
fn completed_receipt_families_are_explicit_and_stable() {
    let provenance = boundary_evidence()
        .provenance()
        .snapshot_bound(
            worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                worth_foundational::BoundaryArtifactLocator::new(
                    worth_foundational::BoundaryArtifactId::new(12),
                    worth_foundational::BoundaryArtifactField::Basis,
                ),
            ),
        )
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
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Admission
    );
    assert_eq!(
        publication.receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Publication
    );
    assert_eq!(
        restoration.receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::Restoration
    );
    assert_eq!(
        support_publication.receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::SupportPublication
    );
    assert_eq!(
        checkpoint_resume.receipt_kind(),
        worth_foundational::FoundationalBoundaryEvidenceReceiptKind::CheckpointResume
    );
}

#[test]
fn executed_receipt_semantics_are_stable_across_common_and_direct_construction_paths() {
    let common_path_receipt = worth_foundational::boundary_evidence_api::common_path::receipt()
        .execution(receipt_boundary())
        .with_provenance(historical_provenance());
    let direct_path_receipt = boundary_evidence()
        .receipt()
        .execution(receipt_boundary())
        .with_provenance(historical_provenance());

    assert_eq!(common_path_receipt, direct_path_receipt);
}

#[test]
fn common_path_and_lower_lane_expose_the_same_surface() {
    assert_eq!(
        boundary_evidence().receipt_kind_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::receipts::foundational_boundary_evidence_receipt_kind_definitions()
    );
    assert_eq!(
        boundary_evidence().closeout_disposition_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::receipts::foundational_boundary_evidence_closeout_disposition_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceReceiptFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::receipt();
}
