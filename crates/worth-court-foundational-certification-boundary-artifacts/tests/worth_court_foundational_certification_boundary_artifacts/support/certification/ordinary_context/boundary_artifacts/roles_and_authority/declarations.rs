pub(crate) use {
    worth_foundational::admit_authoritative_current_boundary_surface,
    worth_foundational::boundary_role_definitions,
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::claim_planned_work_boundary_surface,
    worth_foundational::claim_receipt_evidence_boundary_surface,
    worth_foundational::claim_support_only_boundary_surface,
    worth_foundational::evaluate_boundary_role_claim_legality,
    worth_foundational::foundational_boundary_authority_admission,
    worth_foundational::FoundationalBoundaryArtifactCategory,
    worth_foundational::FoundationalBoundaryArtifactRole,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalBoundaryAuthorityAdmitted,
    worth_foundational::FoundationalBoundaryCategoryConstructionDenial,
    worth_foundational::FoundationalBoundaryReceiptSurface,
    worth_foundational::FoundationalBoundaryReportSurface,
    worth_foundational::FoundationalBoundaryRoleClaimDenial,
    worth_foundational::FoundationalBoundarySummarySurface,
};

pub(crate) fn accepts_authority_admission_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryAuthorityAdmitted,
        worth_foundational::FoundationalBoundaryAuthorityAdmission,
    >,
) {
}
