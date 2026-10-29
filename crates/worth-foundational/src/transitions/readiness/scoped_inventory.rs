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
        FoundationalTransitionCertifiedSurface::ScopedMergeRequestVocabulary,
        FoundationalTransitionCertifiedSurface::ScopedMergeAdmissionEvidence,
        FoundationalTransitionCertifiedSurface::ScopedMergeDenialUnavailableTopology,
        FoundationalTransitionCertifiedSurface::ScopedMergeCanonicalLocatorDiagnostics,
        FoundationalTransitionCertifiedSurface::ScopedMergeAdoptionContract,
    ]
}

pub(super) fn certified_surface_evidence() -> Vec<FoundationalTransitionCertifiedSurfaceEvidence> {
    vec![
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::ScopedMergeRequestVocabulary,
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeCategorySubstitutionHostility,
            FoundationalTransitionCompileFailBoundary::ScopedMergeScopeRequiresTypedLoci,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge.rs",
            "tests/ui/transitions/merge_admission/raw_string_cannot_satisfy_merge_scope_api.rs",
            "tests/worth_foundational/fast/certification/transitions/scoped_merge.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::ScopedMergeAdmissionEvidence,
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeProducerDiversityHostility,
            FoundationalTransitionCompileFailBoundary::SelectedNodeAndAspectRequestsAreNotSubstitutable,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_evidence.rs",
            "tests/ui/transitions/merge_admission/selected_node_cannot_satisfy_selected_aspect_entry_api.rs",
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_evidence.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::ScopedMergeDenialUnavailableTopology,
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeUnavailableDenialHonesty,
            FoundationalTransitionCompileFailBoundary::ScopedMergeScopeRequiresTypedLoci,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_denials.rs",
            "tests/ui/transitions/merge_admission/raw_string_cannot_satisfy_merge_scope_api.rs",
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_posture/mod.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::ScopedMergeCanonicalLocatorDiagnostics,
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeCanonicalLocatorStability,
            FoundationalTransitionCompileFailBoundary::SelectedScopeLocatorRequiresTypedLoci,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_canonical/mod.rs",
            "tests/ui/transitions/merge_admission/raw_string_cannot_satisfy_selected_scope_locator_api.rs",
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_diagnostics/mod.rs",
        ),
        FoundationalTransitionCertifiedSurfaceEvidence::new(
            FoundationalTransitionCertifiedSurface::ScopedMergeAdoptionContract,
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeRuntimeBoundaryHonesty,
            FoundationalTransitionCompileFailBoundary::TransitionReadinessRequiresCertifiedArtifact,
            "tests/worth_foundational/fast/certification/transitions/readiness.rs",
            "tests/ui/transitions/readiness_boundaries/plain_transition_bundle_cannot_satisfy_transition_production_readiness.rs",
            "docs/scoped-merge-adoption.md",
        ),
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalTransitionSyntheticRuntimePressure> {
    vec![
        FoundationalTransitionSyntheticRuntimePressure::ScopedMergeCategorySubstitutionHostility,
        FoundationalTransitionSyntheticRuntimePressure::ScopedMergeProducerDiversityHostility,
        FoundationalTransitionSyntheticRuntimePressure::ScopedMergeUnavailableDenialHonesty,
        FoundationalTransitionSyntheticRuntimePressure::ScopedMergeCanonicalLocatorStability,
        FoundationalTransitionSyntheticRuntimePressure::ScopedMergeRuntimeBoundaryHonesty,
    ]
}

pub(super) fn synthetic_pressure_evidence() -> Vec<FoundationalTransitionSyntheticPressureEvidence>
{
    vec![
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeCategorySubstitutionHostility,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeProducerDiversityHostility,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_evidence.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeUnavailableDenialHonesty,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_posture/mod.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeCanonicalLocatorStability,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_canonical/mod.rs",
        ),
        FoundationalTransitionSyntheticPressureEvidence::new(
            FoundationalTransitionSyntheticRuntimePressure::ScopedMergeRuntimeBoundaryHonesty,
            "docs/scoped-merge-adoption.md",
        ),
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalTransitionCompileFailBoundary> {
    vec![
        FoundationalTransitionCompileFailBoundary::ScopedMergeScopeRequiresTypedLoci,
        FoundationalTransitionCompileFailBoundary::SelectedScopeLocatorRequiresTypedLoci,
        FoundationalTransitionCompileFailBoundary::SelectedNodeAndAspectRequestsAreNotSubstitutable,
        FoundationalTransitionCompileFailBoundary::TransitionReadinessRequiresCertifiedArtifact,
        FoundationalTransitionCompileFailBoundary::TransitionReadinessAuthorityCannotBeMinted,
    ]
}

pub(super) fn compile_fail_evidence() -> Vec<FoundationalTransitionCompileFailEvidence> {
    vec![
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::ScopedMergeScopeRequiresTypedLoci,
            "tests/ui/transitions/merge_admission/raw_string_cannot_satisfy_merge_scope_api.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::SelectedScopeLocatorRequiresTypedLoci,
            "tests/ui/transitions/merge_admission/raw_string_cannot_satisfy_selected_scope_locator_api.rs",
        ),
        FoundationalTransitionCompileFailEvidence::new(
            FoundationalTransitionCompileFailBoundary::SelectedNodeAndAspectRequestsAreNotSubstitutable,
            "tests/ui/transitions/merge_admission/selected_node_cannot_satisfy_selected_aspect_entry_api.rs",
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

pub(super) fn worth_proof_required_surfaces() -> Vec<FoundationalTransitionWORTHProofSurface> {
    vec![
        FoundationalTransitionWORTHProofSurface::TransitionOutcomeAdmissionLane,
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
        FoundationalTransitionWORTHProofApi::ProofFromAuthorityWitness,
        FoundationalTransitionWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
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
        FoundationalTransitionWORTHProofForbiddenSurface::PlainScopedMergeStrings,
        FoundationalTransitionWORTHProofForbiddenSurface::PlainMergeVerdictVocabulary,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalTransitionRuntimeAssumption> {
    vec![
        FoundationalTransitionRuntimeAssumption::Milestone2CanonicalizationRemainsAuthorityForTransitionBasisReadiness,
        FoundationalTransitionRuntimeAssumption::Milestone3ProfilesGovernTransitionAttachmentAndElision,
        FoundationalTransitionRuntimeAssumption::ScopedMergeVocabularyMustPrecedeRuntimeExecution,
        FoundationalTransitionRuntimeAssumption::TransitionMeaningRemainsFacadeControlled,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalTransitionRuntimeNonAssumption> {
    vec![
        FoundationalTransitionRuntimeNonAssumption::FoundationalExecutesScopedMergeOrCherryPick,
        FoundationalTransitionRuntimeNonAssumption::AdoptingCratesMayInventScopedMergeDialect,
        FoundationalTransitionRuntimeNonAssumption::GenericBranchOrMergeEngineExistsInFoundational,
    ]
}

pub(super) fn residual_debt() -> Vec<FoundationalTransitionResidualDebt> {
    vec![
        FoundationalTransitionResidualDebt::AdoptingCrateScopedMergeExecutionDeferred,
        FoundationalTransitionResidualDebt::NativeCherryPickExecutionDeferred,
        FoundationalTransitionResidualDebt::RuntimeConflictMaterializationDeferred,
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalTransitionPhaseGateEvidence> {
    vec![
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::ScopedMergeRequestVocabulary,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::ScopedMergeAdmissionEvidence,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_evidence.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::ScopedMergeDenialUnavailableTopology,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_denials.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::ScopedMergeCanonicalLocatorDiagnostics,
            "tests/worth_foundational/fast/certification/transitions/scoped_merge_diagnostics/mod.rs",
        ),
        FoundationalTransitionPhaseGateEvidence::new(
            FoundationalTransitionMilestone5PhaseGate::ScopedMergeProductionReadiness,
            "tests/worth_foundational/fast/certification/transitions/readiness.rs",
        ),
    ]
}
