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
            "tests/worth_foundational/fast/certification/boundary_artifacts/categories.rs",
            "tests/ui/boundary_artifacts/categories/local_generic_wrapper_cannot_satisfy_category_surface_trait.rs",
            "tests/worth_foundational/fast/certification/boundary_artifacts/categories.rs",
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::RoleAndAuthorityLaw,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::AuthorityDerivationSeparation,
            FoundationalBoundaryArtifactCompileFailBoundary::IllegalRoleAndAuthorityClaimsRejected,
            "tests/worth_foundational/fast/certification/boundary_artifacts/roles_and_authority.rs",
            "tests/ui/boundary_artifacts/role_legality/report_cannot_claim_receipt_evidence.rs",
            "tests/worth_foundational/fast/certification/boundary_artifacts/roles_and_authority.rs",
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::MaterializationAndBundles,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::MaterializationSeamHonesty,
            FoundationalBoundaryArtifactCompileFailBoundary::PlainPayloadCannotBypassMaterializationContracts,
            "tests/worth_foundational/fast/certification/boundary_artifacts/materialization.rs",
            "tests/ui/boundary_artifacts/materialization_contracts/raw_surface_cannot_skip_claim_admission_for_materialization.rs",
            "tests/worth_foundational/fast/certification/boundary_artifacts/materialization.rs",
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::CanonicalBasisParticipation,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::CanonicalBasisParity,
            FoundationalBoundaryArtifactCompileFailBoundary::RawMaterializedOutputsCannotSatisfyCanonicalBasisApis,
            "tests/worth_foundational/fast/certification/boundary_artifacts/canonical_basis.rs",
            "tests/ui/boundary_artifacts/basis_boundaries/raw_materialized_boundary_artifact_cannot_satisfy_basis_entry_api.rs",
            "tests/worth_foundational/fast/certification/boundary_artifacts/canonical_basis.rs",
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::CurrentBasisProofLane,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::CurrentBasisReadmissionBoundary,
            FoundationalBoundaryArtifactCompileFailBoundary::RawMaterializedOutputsCannotSatisfyCurrentBasisApis,
            "tests/worth_foundational/fast/certification/boundary_artifacts/current_basis.rs",
            "tests/ui/boundary_artifacts/current_basis_boundaries/raw_materialized_boundary_artifact_cannot_satisfy_current_basis_api.rs",
            "tests/worth_foundational/fast/certification/boundary_artifacts/current_basis.rs",
        ),
        FoundationalBoundaryArtifactCertifiedSurfaceEvidence::new(
            FoundationalBoundaryArtifactCertifiedSurface::DescriptiveExtensionLaw,
            FoundationalBoundaryArtifactSyntheticRuntimePressure::ReservedAuthorityTransitionFailClosedBoundary,
            FoundationalBoundaryArtifactCompileFailBoundary::DescriptiveExtensionsCannotSatisfyAuthorityOrReservedAuthorityApis,
            "tests/worth_foundational/fast/certification/boundary_artifacts/descriptive_extensions.rs",
            "tests/ui/boundary_artifacts/descriptive_extensions/planned_work_boundary_artifact_cannot_satisfy_current_basis_api.rs",
            "tests/worth_foundational/fast/certification/boundary_artifacts/descriptive_extensions.rs",
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
            "tests/worth_foundational/fast/certification/boundary_artifacts/categories.rs",
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::RoleAndAuthority,
            "tests/worth_foundational/fast/certification/boundary_artifacts/roles_and_authority.rs",
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::MaterializationAndBundles,
            "tests/worth_foundational/fast/certification/boundary_artifacts/materialization.rs",
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::CanonicalBasisParticipation,
            "tests/worth_foundational/fast/certification/boundary_artifacts/canonical_basis.rs",
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::CurrentBasisProofLane,
            "tests/worth_foundational/fast/certification/boundary_artifacts/current_basis.rs",
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::DescriptiveExtensions,
            "tests/worth_foundational/fast/certification/boundary_artifacts/descriptive_extensions.rs",
        ),
        FoundationalBoundaryArtifactPhaseGateEvidence::new(
            FoundationalBoundaryArtifactMilestone4PhaseGate::ProductionReadiness,
            "tests/worth_foundational/fast/certification/boundary_artifacts/readiness.rs",
        ),
    ]
}
