use worth_foundational::{
    admit_authoritative_current_boundary_surface, boundary_role_definitions,
    claim_derived_projection_boundary_surface, claim_planned_work_boundary_surface,
    claim_receipt_evidence_boundary_surface, claim_support_only_boundary_surface,
    evaluate_boundary_role_claim_legality, foundational_boundary_authority_admission,
    FoundationalBoundaryArtifactCategory, FoundationalBoundaryArtifactRole,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryAuthorityAdmitted,
    FoundationalBoundaryCategoryConstructionDenial, FoundationalBoundaryReceiptSurface,
    FoundationalBoundaryReportSurface, FoundationalBoundaryRoleClaimDenial,
    FoundationalBoundarySummarySurface,
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
    let summary = FoundationalBoundarySummarySurface::new("overview", 1).expect("summary surface");
    let report = FoundationalBoundaryReportSurface::new(vec!["row"], 1).expect("report surface");
    let artifact = FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 2);
    let receipt =
        FoundationalBoundaryReceiptSurface::new("completed boundary", 1).expect("receipt");

    let derived = claim_derived_projection_boundary_surface(summary);
    assert_eq!(
        derived.category(),
        FoundationalBoundaryArtifactCategory::Summary
    );
    assert_eq!(
        derived.role(),
        FoundationalBoundaryArtifactRole::DerivedProjection
    );

    let support = claim_support_only_boundary_surface(report);
    assert_eq!(
        support.category(),
        FoundationalBoundaryArtifactCategory::Report
    );
    assert_eq!(
        support.role(),
        FoundationalBoundaryArtifactRole::SupportOnly
    );

    let planned = claim_planned_work_boundary_surface(artifact);
    assert_eq!(
        planned.category(),
        FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(
        planned.role(),
        FoundationalBoundaryArtifactRole::PlannedWork
    );

    let receipt_evidence = claim_receipt_evidence_boundary_surface(receipt);
    assert_eq!(
        receipt_evidence.category(),
        FoundationalBoundaryArtifactCategory::Receipt
    );
    assert_eq!(
        receipt_evidence.role(),
        FoundationalBoundaryArtifactRole::ReceiptEvidence
    );
}

#[test]
fn legality_evaluation_names_illegal_role_category_combinations() {
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            FoundationalBoundaryArtifactCategory::Receipt,
            FoundationalBoundaryArtifactRole::DerivedProjection,
        ),
        Err(FoundationalBoundaryRoleClaimDenial::DerivedProjectionCannotUseReceiptCategory)
    );
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            FoundationalBoundaryArtifactCategory::Receipt,
            FoundationalBoundaryArtifactRole::SupportOnly,
        ),
        Err(FoundationalBoundaryRoleClaimDenial::SupportOnlyCannotUseReceiptCategory)
    );
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            FoundationalBoundaryArtifactCategory::Receipt,
            FoundationalBoundaryArtifactRole::PlannedWork,
        ),
        Err(FoundationalBoundaryRoleClaimDenial::PlannedWorkCannotUseReceiptCategory)
    );
    assert_eq!(
        evaluate_boundary_role_claim_legality(
            FoundationalBoundaryArtifactCategory::Report,
            FoundationalBoundaryArtifactRole::ReceiptEvidence,
        ),
        Err(FoundationalBoundaryRoleClaimDenial::ReceiptEvidenceRequiresReceiptCategory)
    );
}

#[test]
fn authoritative_current_requires_explicit_admission_and_artifact_category() {
    let artifact = FoundationalBoundaryArtifactSurface::new(
        vec![FoundationalBoundaryCategoryConstructionDenial::ReportRequiresAtLeastOneRow],
        2,
    );
    let admitted = admit_authoritative_current_boundary_surface(
        artifact,
        foundational_boundary_authority_admission(),
    );

    assert_eq!(
        admitted.claim().role(),
        FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
    assert_eq!(
        admitted.claim().category(),
        FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(admitted.surface().attachment_slot_count(), 2);
    accepts_authority_admission_proof(admitted.proofs());
}

fn accepts_authority_admission_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryAuthorityAdmitted,
        worth_foundational::FoundationalBoundaryAuthorityAdmission,
    >,
) {
}
