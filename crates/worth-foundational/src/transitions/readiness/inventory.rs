use super::vocabulary::{
    FoundationalTransitionCertifiedSurface, FoundationalTransitionCertifiedSurfaceEvidence,
    FoundationalTransitionCompileFailBoundary, FoundationalTransitionCompileFailEvidence,
    FoundationalTransitionMilestone5PhaseGate, FoundationalTransitionPhaseGateEvidence,
    FoundationalTransitionResidualDebt, FoundationalTransitionRuntimeAssumption,
    FoundationalTransitionRuntimeNonAssumption, FoundationalTransitionSyntheticPressureEvidence,
    FoundationalTransitionSyntheticRuntimePressure, FoundationalTransitionWORTHProofApi,
    FoundationalTransitionWORTHProofApiEvidence, FoundationalTransitionWORTHProofForbiddenSurface,
    FoundationalTransitionWORTHProofSurface,
};

pub(super) fn certified_surfaces() -> Vec<FoundationalTransitionCertifiedSurface> {
    vec![
        FoundationalTransitionCertifiedSurface::BranchLocalSeparation,
        FoundationalTransitionCertifiedSurface::MergeVerdictLaw,
        FoundationalTransitionCertifiedSurface::CommittedAuthorityTransitions,
        FoundationalTransitionCertifiedSurface::CommitReceiptsAndBundles,
        FoundationalTransitionCertifiedSurface::CanonicalBasisAndLocatorIntegration,
        FoundationalTransitionCertifiedSurface::ProfileRichnessAndCurrentBasisBehavior,
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalTransitionSyntheticRuntimePressure> {
    vec![
        FoundationalTransitionSyntheticRuntimePressure::AuthoritySeparation,
        FoundationalTransitionSyntheticRuntimePressure::MergeTopologyHonesty,
        FoundationalTransitionSyntheticRuntimePressure::NoOpVersusCommitClassification,
        FoundationalTransitionSyntheticRuntimePressure::ReceiptIssuanceBoundary,
        FoundationalTransitionSyntheticRuntimePressure::ReplayInterpretationBoundary,
        FoundationalTransitionSyntheticRuntimePressure::ReducedRichnessPreservation,
        FoundationalTransitionSyntheticRuntimePressure::AmbientBasisChoiceHostility,
        FoundationalTransitionSyntheticRuntimePressure::HiddenStrategyInfluenceHostility,
        FoundationalTransitionSyntheticRuntimePressure::ThinReceiptRejection,
        FoundationalTransitionSyntheticRuntimePressure::GenericTransitionResultBagRejection,
        FoundationalTransitionSyntheticRuntimePressure::CheapConvenienceBypassRejection,
    ]
}

pub(super) fn certified_surface_evidence() -> Vec<FoundationalTransitionCertifiedSurfaceEvidence> {
    vec![
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::BranchLocalSeparation,
            FoundationalTransitionSyntheticRuntimePressure::AuthoritySeparation,
            FoundationalTransitionCompileFailBoundary::BranchLocalSurfacesCannotSatisfyAuthorityApis,
            "tests/worth_foundational/fast/certification/transitions/branch_local.rs",
            "tests/ui/transitions/branch_local/branch_candidate_cannot_satisfy_authoritative_boundary_api.rs",
            "tests/worth_foundational/fast/certification/transitions/branch_local.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::MergeVerdictLaw,
            FoundationalTransitionSyntheticRuntimePressure::MergeTopologyHonesty,
            FoundationalTransitionCompileFailBoundary::MergeAdmissionSurfacesRemainNonAuthoritative,
            "tests/worth_foundational/fast/certification/transitions/merge_verdicts.rs",
            "tests/ui/transitions/merge_admission/merge_verdict_has_no_receipt_api.rs",
            "tests/worth_foundational/fast/certification/transitions/merge_verdicts.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::CommittedAuthorityTransitions,
            FoundationalTransitionSyntheticRuntimePressure::NoOpVersusCommitClassification,
            FoundationalTransitionCompileFailBoundary::CommittedAuthorityRequiresProofBearingAdmission,
            "tests/worth_foundational/fast/certification/transitions/committed_authority.rs",
            "tests/ui/transitions/committed_authority/merge_verdict_has_no_committed_authority_api.rs",
            "tests/worth_foundational/fast/certification/transitions/committed_authority.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::CommitReceiptsAndBundles,
            FoundationalTransitionSyntheticRuntimePressure::ReceiptIssuanceBoundary,
            FoundationalTransitionCompileFailBoundary::ReceiptAndCloseoutPreserveAuthoritySeparation,
            "tests/worth_foundational/fast/certification/transitions/receipts_and_bundles.rs",
            "tests/ui/transitions/receipt_boundaries/plain_boundary_receipt_surface_has_no_transition_receipt_api.rs",
            "tests/worth_foundational/fast/certification/transitions/receipts_and_bundles.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::CanonicalBasisAndLocatorIntegration,
            FoundationalTransitionSyntheticRuntimePressure::AmbientBasisChoiceHostility,
            FoundationalTransitionCompileFailBoundary::Phase5BasisAndCurrentBasisRequireStrengthenedArtifacts,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
            "tests/ui/transitions/phase5_boundaries/raw_committed_authority_cannot_satisfy_current_basis_transition_api.rs",
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::ProfileRichnessAndCurrentBasisBehavior,
            FoundationalTransitionSyntheticRuntimePressure::ReducedRichnessPreservation,
            FoundationalTransitionCompileFailBoundary::Phase5BasisAndCurrentBasisRequireStrengthenedArtifacts,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
            "tests/ui/transitions/phase5_boundaries/raw_commit_receipt_cannot_satisfy_current_basis_transition_api.rs",
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalTransitionCompileFailBoundary> {
    vec![
        FoundationalTransitionCompileFailBoundary::BranchLocalSurfacesCannotSatisfyAuthorityApis,
        FoundationalTransitionCompileFailBoundary::MergeAdmissionSurfacesRemainNonAuthoritative,
        FoundationalTransitionCompileFailBoundary::CommittedAuthorityRequiresProofBearingAdmission,
        FoundationalTransitionCompileFailBoundary::ReceiptAndCloseoutPreserveAuthoritySeparation,
        FoundationalTransitionCompileFailBoundary::Phase5BasisAndCurrentBasisRequireStrengthenedArtifacts,
        FoundationalTransitionCompileFailBoundary::TransitionReadinessRequiresCertifiedArtifact,
        FoundationalTransitionCompileFailBoundary::TransitionReadinessAuthorityCannotBeMinted,
    ]
}

pub(super) fn worth_proof_required_surfaces() -> Vec<FoundationalTransitionWORTHProofSurface> {
    vec![
        FoundationalTransitionWORTHProofSurface::TransitionOutcomeAdmissionLane,
        FoundationalTransitionWORTHProofSurface::AuthorityWitnessScopedAdmission,
        FoundationalTransitionWORTHProofSurface::ProofBearingCommittedAuthorityArtifact,
        FoundationalTransitionWORTHProofSurface::ProofBearingCommitReceiptArtifact,
        FoundationalTransitionWORTHProofSurface::CurrentBasisArtifactConstructor,
        FoundationalTransitionWORTHProofSurface::BoundaryBridgeTrustBoundary,
        FoundationalTransitionWORTHProofSurface::BoundaryReadmitWithAuthority,
        FoundationalTransitionWORTHProofSurface::ProductionReadinessCertificationArtifact,
    ]
}

pub(super) fn worth_proof_api_appendix() -> Vec<FoundationalTransitionWORTHProofApi> {
    vec![
        FoundationalTransitionWORTHProofApi::TransitionOutcomeStructuredCategories,
        FoundationalTransitionWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
        FoundationalTransitionWORTHProofApi::ProofFromAuthorityWitness,
        FoundationalTransitionWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
        FoundationalTransitionWORTHProofApi::ArtifactWithCurrentBasis,
        FoundationalTransitionWORTHProofApi::ArtifactBridgeTrustBoundary,
        FoundationalTransitionWORTHProofApi::ArtifactReadmitWithAuthority,
    ]
}

pub(super) fn worth_proof_api_evidence() -> Vec<FoundationalTransitionWORTHProofApiEvidence> {
    vec![
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::TransitionOutcomeStructuredCategories,
            "src/transitions/merges/admission.rs",
            "TransitionOutcome::",
        ),
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
            "src/transitions/commits/authority.rs",
            "AuthorityWitness::from_authority_marker",
        ),
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::ProofFromAuthorityWitness,
            "src/transitions/receipts/issuance.rs",
            "Proof::from_authority_witness",
        ),
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
            "src/transitions/commits/authority.rs",
            "Artifact::with_proofs_and_current_basis",
        ),
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::ArtifactWithCurrentBasis,
            "src/transitions/basis/current_basis.rs",
            "Artifact::with_current_basis",
        ),
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::ArtifactBridgeTrustBoundary,
            "src/transitions/basis/current_basis.rs",
            ".bridge_trust_boundary()",
        ),
        FoundationalTransitionWORTHProofApiEvidence::new(
            FoundationalTransitionWORTHProofApi::ArtifactReadmitWithAuthority,
            "src/transitions/basis/current_basis.rs",
            ".readmit_with_authority(",
        ),
    ]
}

pub(super) fn worth_proof_forbidden_surfaces(
) -> Vec<FoundationalTransitionWORTHProofForbiddenSurface> {
    vec![
        FoundationalTransitionWORTHProofForbiddenSurface::PlainBranchLocalVocabulary,
        FoundationalTransitionWORTHProofForbiddenSurface::PlainMergeVerdictVocabulary,
        FoundationalTransitionWORTHProofForbiddenSurface::PlainReceiptAndBundleVocabulary,
        FoundationalTransitionWORTHProofForbiddenSurface::PlainCanonicalBasisAndLocatorVocabulary,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalTransitionRuntimeAssumption> {
    vec![
        FoundationalTransitionRuntimeAssumption::Milestone2CanonicalizationRemainsAuthorityForTransitionBasisReadiness,
        FoundationalTransitionRuntimeAssumption::Milestone3ProfilesGovernTransitionAttachmentAndElision,
        FoundationalTransitionRuntimeAssumption::StrongerCommittedAuthorityAndReceiptClaimsUseWORTHProof,
        FoundationalTransitionRuntimeAssumption::TransitionMeaningRemainsFacadeControlled,
    ]
}

pub(super) fn synthetic_pressure_evidence() -> Vec<FoundationalTransitionSyntheticPressureEvidence>
{
    vec![
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::AuthoritySeparation,
            "tests/worth_foundational/fast/certification/transitions/branch_local.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::MergeTopologyHonesty,
            "tests/worth_foundational/fast/certification/transitions/merge_verdicts.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::NoOpVersusCommitClassification,
            "tests/worth_foundational/fast/certification/transitions/committed_authority.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ReceiptIssuanceBoundary,
            "tests/worth_foundational/fast/certification/transitions/receipts_and_bundles.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ReplayInterpretationBoundary,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ReducedRichnessPreservation,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::AmbientBasisChoiceHostility,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::HiddenStrategyInfluenceHostility,
            "tests/worth_foundational/fast/certification/transitions/merge_verdicts.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ThinReceiptRejection,
            "tests/worth_foundational/fast/certification/transitions/receipts_and_bundles.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::GenericTransitionResultBagRejection,
            "tests/worth_foundational/fast/certification/transitions/receipts_and_bundles.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::CheapConvenienceBypassRejection,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
    ]
}

pub(super) fn compile_fail_evidence() -> Vec<FoundationalTransitionCompileFailEvidence> {
    vec![
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::BranchLocalSurfacesCannotSatisfyAuthorityApis,
            "tests/ui/transitions/branch_local/branch_candidate_cannot_satisfy_authoritative_boundary_api.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::MergeAdmissionSurfacesRemainNonAuthoritative,
            "tests/ui/transitions/merge_admission/merge_verdict_has_no_receipt_api.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::CommittedAuthorityRequiresProofBearingAdmission,
            "tests/ui/transitions/committed_authority/caller_cannot_mint_committed_authority_admission.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::ReceiptAndCloseoutPreserveAuthoritySeparation,
            "tests/ui/transitions/receipt_boundaries/plain_boundary_receipt_surface_has_no_transition_receipt_api.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::Phase5BasisAndCurrentBasisRequireStrengthenedArtifacts,
            "tests/ui/transitions/phase5_boundaries/raw_committed_authority_cannot_satisfy_current_basis_transition_api.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::TransitionReadinessRequiresCertifiedArtifact,
            "tests/ui/transitions/readiness_boundaries/plain_transition_bundle_cannot_satisfy_transition_production_readiness.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::TransitionReadinessAuthorityCannotBeMinted,
            "tests/ui/transitions/readiness_boundaries/transition_readiness_authority_cannot_be_minted.rs",
        ),
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalTransitionRuntimeNonAssumption> {
    vec![
        FoundationalTransitionRuntimeNonAssumption::DiagnosticsOntologyAlreadyOwnedHere,
        FoundationalTransitionRuntimeNonAssumption::ProvenanceOntologyBeyondTransitionRowsAlreadyOwnedHere,
        FoundationalTransitionRuntimeNonAssumption::AdoptingRuntimeMergeStrategyParityAlreadyProven,
        FoundationalTransitionRuntimeNonAssumption::BoundaryCrossingPreservesCurrentBasisWithoutReadmission,
        FoundationalTransitionRuntimeNonAssumption::GenericBranchOrMergeEngineExistsInFoundational,
    ]
}

pub(super) fn residual_debt() -> Vec<FoundationalTransitionResidualDebt> {
    vec![
        FoundationalTransitionResidualDebt::AdoptingRuntimeParityDeferred,
        FoundationalTransitionResidualDebt::LaterDiagnosticsAndProvenanceOntologyDeferred,
        FoundationalTransitionResidualDebt::RuntimeStrategyRegistryAndExecutionDeferred,
        FoundationalTransitionResidualDebt::FullLineageSupportBeyondTransitionRowsDeferred,
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalTransitionPhaseGateEvidence> {
    vec![
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::BranchLocalSeparation,
            "tests/worth_foundational/fast/certification/transitions/branch_local.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::MergeVerdictLaw,
            "tests/worth_foundational/fast/certification/transitions/merge_verdicts.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::CommittedAuthorityTransitionLaw,
            "tests/worth_foundational/fast/certification/transitions/committed_authority.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::CommitReceiptsAndBundles,
            "tests/worth_foundational/fast/certification/transitions/receipts_and_bundles.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::CanonicalBasisLocatorAndProfileIntegration,
            "tests/worth_foundational/fast/certification/transitions/phase5_basis.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::ProductionReadiness,
            "tests/worth_foundational/fast/certification/transitions/readiness.rs",
        ),
    ]
}
