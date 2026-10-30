use super::vocabulary::{
    FoundationalBoundaryArtifactCertifiedSurface,
    FoundationalBoundaryArtifactCertifiedSurfaceEvidence,
    FoundationalBoundaryArtifactCompileFailBoundary,
    FoundationalBoundaryArtifactMilestone4PhaseGate, FoundationalBoundaryArtifactPhaseGateEvidence,
    FoundationalBoundaryArtifactResidualDebt, FoundationalBoundaryArtifactRuntimeAssumption,
    FoundationalBoundaryArtifactRuntimeNonAssumption,
    FoundationalBoundaryArtifactSyntheticRuntimePressure,
    FoundationalBoundaryArtifactWORTHProofApi,
    FoundationalBoundaryArtifactWORTHProofForbiddenSurface,
    FoundationalBoundaryArtifactWORTHProofSurface,
};

pub(super) fn certified_surfaces() -> Vec<FoundationalBoundaryArtifactCertifiedSurface> {
    vec![
        FoundationalBoundaryArtifactCertifiedSurface::CategoryVocabulary,
        FoundationalBoundaryArtifactCertifiedSurface::RoleAndAuthorityLaw,
        FoundationalBoundaryArtifactCertifiedSurface::MaterializationAndBundles,
        FoundationalBoundaryArtifactCertifiedSurface::CanonicalBasisParticipation,
        FoundationalBoundaryArtifactCertifiedSurface::CurrentBasisProofLane,
        FoundationalBoundaryArtifactCertifiedSurface::DescriptiveExtensionLaw,
    ]
}

pub(super) fn certified_surface_evidence(
) -> Vec<FoundationalBoundaryArtifactCertifiedSurfaceEvidence> {
    vec![
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::CategoryVocabulary,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::CategoryAdjacencyHostility,
            FoundationalBoundaryArtifactCompileFailBoundary::CategoryWrapperCollapseRejected,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/categories/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_artifacts/categories/local_generic_wrapper_cannot_satisfy_category_surface_trait.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/categories/mod.rs"),
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::RoleAndAuthorityLaw,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::AuthorityDerivationSeparation,
            FoundationalBoundaryArtifactCompileFailBoundary::IllegalRoleAndAuthorityClaimsRejected,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/roles_and_authority/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_artifacts/role_legality/report_cannot_claim_receipt_evidence.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/roles_and_authority/mod.rs"),
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::MaterializationAndBundles,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::MaterializationSeamHonesty,
            FoundationalBoundaryArtifactCompileFailBoundary::PlainPayloadCannotBypassMaterializationContracts,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/materialization/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_artifacts/materialization_contracts/raw_surface_cannot_skip_claim_admission_for_materialization.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/materialization/mod.rs"),
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::CanonicalBasisParticipation,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::CanonicalBasisParity,
            FoundationalBoundaryArtifactCompileFailBoundary::RawMaterializedOutputsCannotSatisfyCanonicalBasisApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/canonical_basis/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_artifacts/basis_boundaries/raw_materialized_boundary_artifact_cannot_satisfy_basis_entry_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/canonical_basis/mod.rs"),
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::CurrentBasisProofLane,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::CurrentBasisReadmissionBoundary,
            FoundationalBoundaryArtifactCompileFailBoundary::RawMaterializedOutputsCannotSatisfyCurrentBasisApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/current_basis/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_artifacts/current_basis_boundaries/raw_materialized_boundary_artifact_cannot_satisfy_current_basis_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/current_basis/mod.rs"),
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::DescriptiveExtensionLaw,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::ReservedAuthorityTransitionFailClosedBoundary,
            FoundationalBoundaryArtifactCompileFailBoundary::DescriptiveExtensionsCannotSatisfyAuthorityOrReservedAuthorityApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/descriptive_extensions/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_artifacts/descriptive_extensions/planned_work_boundary_artifact_cannot_satisfy_current_basis_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/descriptive_extensions/mod.rs"),
        ),
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalBoundaryArtifactSyntheticRuntimePressure> {
    vec![
        FoundationalBoundaryArtifactSyntheticRuntimePressure::CategoryAdjacencyHostility,
        FoundationalBoundaryArtifactSyntheticRuntimePressure::AuthorityDerivationSeparation,
        FoundationalBoundaryArtifactSyntheticRuntimePressure::MaterializationSeamHonesty,
        FoundationalBoundaryArtifactSyntheticRuntimePressure::CanonicalBasisParity,
        FoundationalBoundaryArtifactSyntheticRuntimePressure::CurrentBasisReadmissionBoundary,
        FoundationalBoundaryArtifactSyntheticRuntimePressure::ReservedAuthorityTransitionFailClosedBoundary,
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalBoundaryArtifactCompileFailBoundary> {
    vec![
        FoundationalBoundaryArtifactCompileFailBoundary::CategoryWrapperCollapseRejected,
        FoundationalBoundaryArtifactCompileFailBoundary::IllegalRoleAndAuthorityClaimsRejected,
        FoundationalBoundaryArtifactCompileFailBoundary::PlainPayloadCannotBypassMaterializationContracts,
        FoundationalBoundaryArtifactCompileFailBoundary::RawMaterializedOutputsCannotSatisfyCanonicalBasisApis,
        FoundationalBoundaryArtifactCompileFailBoundary::RawMaterializedOutputsCannotSatisfyCurrentBasisApis,
        FoundationalBoundaryArtifactCompileFailBoundary::DescriptiveExtensionsCannotSatisfyAuthorityOrReservedAuthorityApis,
        FoundationalBoundaryArtifactCompileFailBoundary::BoundaryArtifactReadinessRequiresCertifiedArtifact,
    ]
}

pub(super) fn worth_proof_required_surfaces() -> Vec<FoundationalBoundaryArtifactWORTHProofSurface>
{
    vec![
        FoundationalBoundaryArtifactWORTHProofSurface::AuthorityWitness,
        FoundationalBoundaryArtifactWORTHProofSurface::AuthorityAdmissionProofBearingClaim,
        FoundationalBoundaryArtifactWORTHProofSurface::TransitionOutcome,
        FoundationalBoundaryArtifactWORTHProofSurface::CurrentBasisArtifactConstructor,
        FoundationalBoundaryArtifactWORTHProofSurface::BoundaryBridgeTrustBoundary,
        FoundationalBoundaryArtifactWORTHProofSurface::BoundaryReadmitWithAuthority,
        FoundationalBoundaryArtifactWORTHProofSurface::ProductionReadinessCertificationArtifact,
    ]
}

pub(super) fn worth_proof_api_appendix() -> Vec<FoundationalBoundaryArtifactWORTHProofApi> {
    vec![
        FoundationalBoundaryArtifactWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
        FoundationalBoundaryArtifactWORTHProofApi::ProofFromAuthorityWitness,
        FoundationalBoundaryArtifactWORTHProofApi::ArtifactWithCurrentBasisProofs,
        FoundationalBoundaryArtifactWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
        FoundationalBoundaryArtifactWORTHProofApi::TransitionOutcomeStructuredCategories,
        FoundationalBoundaryArtifactWORTHProofApi::ArtifactBridgeTrustBoundary,
        FoundationalBoundaryArtifactWORTHProofApi::ArtifactReadmitWithAuthority,
    ]
}

pub(super) fn worth_proof_forbidden_surfaces(
) -> Vec<FoundationalBoundaryArtifactWORTHProofForbiddenSurface> {
    vec![
        FoundationalBoundaryArtifactWORTHProofForbiddenSurface::PlainCategoryVocabulary,
        FoundationalBoundaryArtifactWORTHProofForbiddenSurface::PlainRoleAndMaterializationVocabulary,
        FoundationalBoundaryArtifactWORTHProofForbiddenSurface::PlainBundleMembershipData,
        FoundationalBoundaryArtifactWORTHProofForbiddenSurface::PlainSameFamilyDescriptiveNouns,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalBoundaryArtifactRuntimeAssumption> {
    vec![
        FoundationalBoundaryArtifactRuntimeAssumption::Milestone2CanonicalizationRemainsAuthorityForBasisReadiness,
        FoundationalBoundaryArtifactRuntimeAssumption::Milestone3ProfilesGovernAttachmentAndElision,
        FoundationalBoundaryArtifactRuntimeAssumption::StrongerAuthorityAndCurrentBasisClaimsRequireProofLane,
        FoundationalBoundaryArtifactRuntimeAssumption::BoundaryArtifactMeaningRemainsFacadeControlled,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalBoundaryArtifactRuntimeNonAssumption> {
    vec![
        FoundationalBoundaryArtifactRuntimeNonAssumption::ReservedAuthorityTransitionOntologyAlreadyOwnedHere,
        FoundationalBoundaryArtifactRuntimeNonAssumption::AdoptingCrateParityAlreadyProven,
        FoundationalBoundaryArtifactRuntimeNonAssumption::DiagnosticsOrProvenanceOntologyAlreadyOwnedHere,
        FoundationalBoundaryArtifactRuntimeNonAssumption::ReceiptSemanticsBeyondCategoryLawAlreadyOwnedHere,
    ]
}

pub(super) fn residual_debt() -> Vec<FoundationalBoundaryArtifactResidualDebt> {
    vec![
        FoundationalBoundaryArtifactResidualDebt::AdoptingCrateParityDeferred,
        FoundationalBoundaryArtifactResidualDebt::ReservedAuthorityTransitionOntologyDeferred,
        FoundationalBoundaryArtifactResidualDebt::LaterDiagnosticsProvenanceAndReceiptSemanticsDeferred,
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalBoundaryArtifactPhaseGateEvidence> {
    vec![
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::Categories,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/categories/mod.rs"),
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::RoleAndAuthority,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/roles_and_authority/mod.rs"),
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::MaterializationAndBundles,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/materialization/mod.rs"),
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::CanonicalBasisParticipation,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/canonical_basis/mod.rs"),
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::CurrentBasisProofLane,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/current_basis/mod.rs"),
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::DescriptiveExtensions,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/descriptive_extensions/mod.rs"),
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::ProductionReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/boundary_artifacts/readiness/mod.rs"),
        ),
    ]
}
