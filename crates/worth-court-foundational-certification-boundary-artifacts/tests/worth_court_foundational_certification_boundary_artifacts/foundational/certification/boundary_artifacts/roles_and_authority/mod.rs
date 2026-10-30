#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_artifacts::roles_and_authority::*;
use {
    worth_foundational::admit_authoritative_current_boundary_surface,
    worth_foundational::boundary_role_definitions,
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::claim_planned_work_boundary_surface,
    worth_foundational::claim_receipt_evidence_boundary_surface,
    worth_foundational::claim_support_only_boundary_surface,
    worth_foundational::evaluate_boundary_role_claim_legality,
    worth_foundational::foundational_boundary_authority_admission,
};

#[test]
fn role_definitions_are_blind_consumer_interpretable() {
    let definitions = boundary_role_definitions();
    let names: Vec<_> = definitions
        .iter()
        .map(|definition| definition.name())
        .collect();

    assert_eq!(
        names,
        vec![
            "authoritative_current",
            "derived_projection",
            "support_only",
            "planned_work",
            "receipt_evidence",
        ]
    );
    assert!(definitions
        .iter()
        .all(|definition| !definition.intended_claim().trim().is_empty()));
}

#[test]
fn descriptive_role_claims_preserve_category_and_role_meaning() {
    let summary = worth_foundational::FoundationalBoundarySummarySurface::new("overview", 1)
        .expect("summary surface");
    let report = worth_foundational::FoundationalBoundaryReportSurface::new(vec!["row"], 1)
        .expect("report surface");
    let artifact =
        worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 2);
    let receipt =
        worth_foundational::FoundationalBoundaryReceiptSurface::new("completed boundary", 1)
            .expect("receipt");

    let derived = claim_derived_projection_boundary_surface(summary);
    assert_eq!(
        derived.category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Summary
    );
    assert_eq!(
        derived.role(),
        worth_foundational::FoundationalBoundaryArtifactRole::DerivedProjection
    );

    let support = claim_support_only_boundary_surface(report);
    assert_eq!(
        support.category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Report
    );
    assert_eq!(
        support.role(),
        worth_foundational::FoundationalBoundaryArtifactRole::SupportOnly
    );

    let planned = claim_planned_work_boundary_surface(artifact);
    assert_eq!(
        planned.category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(
        planned.role(),
        worth_foundational::FoundationalBoundaryArtifactRole::PlannedWork
    );

    let receipt_evidence = claim_receipt_evidence_boundary_surface(receipt);
    assert_eq!(
        receipt_evidence.category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Receipt
    );
    assert_eq!(
        receipt_evidence.role(),
        worth_foundational::FoundationalBoundaryArtifactRole::ReceiptEvidence
    );
}

#[test]
fn legality_evaluation_names_illegal_role_category_combinations() {
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            worth_foundational::FoundationalBoundaryArtifactCategory::Receipt,
            worth_foundational::FoundationalBoundaryArtifactRole::DerivedProjection,
        ),
        Err(worth_foundational::FoundationalBoundaryRoleClaimDenial::DerivedProjectionCannotUseReceiptCategory)
    );
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            worth_foundational::FoundationalBoundaryArtifactCategory::Receipt,
            worth_foundational::FoundationalBoundaryArtifactRole::SupportOnly,
        ),
        Err(worth_foundational::FoundationalBoundaryRoleClaimDenial::SupportOnlyCannotUseReceiptCategory)
    );
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            worth_foundational::FoundationalBoundaryArtifactCategory::Receipt,
            worth_foundational::FoundationalBoundaryArtifactRole::PlannedWork,
        ),
        Err(worth_foundational::FoundationalBoundaryRoleClaimDenial::PlannedWorkCannotUseReceiptCategory)
    );
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            worth_foundational::FoundationalBoundaryArtifactCategory::Report,
            worth_foundational::FoundationalBoundaryArtifactRole::ReceiptEvidence,
        ),
        Err(worth_foundational::FoundationalBoundaryRoleClaimDenial::ReceiptEvidenceRequiresReceiptCategory)
    );
}

#[test]
fn authoritative_current_requires_explicit_admission_and_artifact_category() {
    let artifact = worth_foundational::FoundationalBoundaryArtifactSurface::new(
        vec![worth_foundational::FoundationalBoundaryCategoryConstructionDenial::ReportRequiresAtLeastOneRow],
        2,
    );
    let admitted = admit_authoritative_current_boundary_surface(
        artifact,
        foundational_boundary_authority_admission(),
    );

    assert_eq!(
        admitted.claim().role(),
        worth_foundational::FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
    assert_eq!(
        admitted.claim().category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(admitted.surface().attachment_slot_count(), 2);
    accepts_authority_admission_proof(admitted.proofs());
}
