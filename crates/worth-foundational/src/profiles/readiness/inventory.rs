use super::vocabulary::{
    FoundationalProfileCertifiedSurface, FoundationalProfileCertifiedSurfaceEvidence,
    FoundationalProfileCompileFailBoundary, FoundationalProfileMilestone3PhaseGate,
    FoundationalProfilePhaseGateEvidence, FoundationalProfileResidualDebt,
    FoundationalProfileRuntimeAssumption, FoundationalProfileRuntimeNonAssumption,
    FoundationalProfileSyntheticRuntimePressure, FoundationalProfileWORTHProofApi,
    FoundationalProfileWORTHProofForbiddenSurface, FoundationalProfileWORTHProofSurface,
};
use crate::profiles_api::{
    profile_public_surface_inventory, FoundationalProfilePublicSurfaceEntry,
};

pub(super) fn certified_surfaces() -> Vec<FoundationalProfileCertifiedSurface> {
    vec![
        FoundationalProfileCertifiedSurface::ProfileFamilies,
        FoundationalProfileCertifiedSurface::ProfileComposition,
        FoundationalProfileCertifiedSurface::ProgressionAndAttachment,
        FoundationalProfileCertifiedSurface::CanonicalIdentityAndDifference,
        FoundationalProfileCertifiedSurface::MaterializationAndElision,
        FoundationalProfileCertifiedSurface::CertificationStrengthening,
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalProfileSyntheticRuntimePressure> {
    vec![
        FoundationalProfileSyntheticRuntimePressure::FamilyAdjacencyHostility,
        FoundationalProfileSyntheticRuntimePressure::IndependentConstructionParity,
        FoundationalProfileSyntheticRuntimePressure::ReducedRichnessSuppression,
        FoundationalProfileSyntheticRuntimePressure::AttachmentTargetLaw,
        FoundationalProfileSyntheticRuntimePressure::ProofBearingCertificationBoundary,
    ]
}

pub(super) fn certified_surface_evidence() -> Vec<FoundationalProfileCertifiedSurfaceEvidence> {
    vec![
        FoundationalProfileCertifiedSurfaceEvidence::new(
            FoundationalProfileCertifiedSurface::ProfileFamilies,
            FoundationalProfileSyntheticRuntimePressure::FamilyAdjacencyHostility,
            FoundationalProfileCompileFailBoundary::RawLabelsCannotSatisfyProfileFamilyApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/composition/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/family_boundaries/raw_string_cannot_satisfy_richness_profile.rs"),
        ),
        FoundationalProfileCertifiedSurfaceEvidence::new(
            FoundationalProfileCertifiedSurface::ProfileComposition,
            FoundationalProfileSyntheticRuntimePressure::IndependentConstructionParity,
            FoundationalProfileCompileFailBoundary::PartialOrBagConstructionCannotSatisfyProfileSetApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/composition/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/set_construction/raw_collection_cannot_satisfy_profile_set_api.rs"),
        ),
        FoundationalProfileCertifiedSurfaceEvidence::new(
            FoundationalProfileCertifiedSurface::ProgressionAndAttachment,
            FoundationalProfileSyntheticRuntimePressure::AttachmentTargetLaw,
            FoundationalProfileCompileFailBoundary::PlainPayloadCannotSatisfyAttachmentApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/progression_and_attachment/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/attachment_boundaries/plain_payload_cannot_satisfy_boundary_profiled_api.rs"),
        ),
        FoundationalProfileCertifiedSurfaceEvidence::new(
            FoundationalProfileCertifiedSurface::CanonicalIdentityAndDifference,
            FoundationalProfileSyntheticRuntimePressure::IndependentConstructionParity,
            FoundationalProfileCompileFailBoundary::RawDigestCannotSatisfyProfileIdentityApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/identity_and_difference/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/identity_boundaries/raw_digest_cannot_satisfy_profile_identity_api.rs"),
        ),
        FoundationalProfileCertifiedSurfaceEvidence::new(
            FoundationalProfileCertifiedSurface::MaterializationAndElision,
            FoundationalProfileSyntheticRuntimePressure::ReducedRichnessSuppression,
            FoundationalProfileCompileFailBoundary::IllegalTargetSurfaceInventoriesCannotBeWorthd,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/materialization/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/materialization_boundaries/target_surface_inventory_fields_are_private.rs"),
        ),
        FoundationalProfileCertifiedSurfaceEvidence::new(
            FoundationalProfileCertifiedSurface::CertificationStrengthening,
            FoundationalProfileSyntheticRuntimePressure::ProofBearingCertificationBoundary,
            FoundationalProfileCompileFailBoundary::WrongStrengthProofBearingCertificationCannotSatisfyStrongerApis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/certification_posture/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/certification_boundaries/evidence_backed_artifact_cannot_satisfy_production_certified_boundary_api.rs"),
        ),
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalProfileCompileFailBoundary> {
    vec![
        FoundationalProfileCompileFailBoundary::RawLabelsCannotSatisfyProfileFamilyApis,
        FoundationalProfileCompileFailBoundary::PartialOrBagConstructionCannotSatisfyProfileSetApis,
        FoundationalProfileCompileFailBoundary::PlainPayloadCannotSatisfyAttachmentApis,
        FoundationalProfileCompileFailBoundary::RawDigestCannotSatisfyProfileIdentityApis,
        FoundationalProfileCompileFailBoundary::IllegalTargetSurfaceInventoriesCannotBeWorthd,
        FoundationalProfileCompileFailBoundary::WrongStrengthProofBearingCertificationCannotSatisfyStrongerApis,
        FoundationalProfileCompileFailBoundary::ProfileReadinessRequiresCertifiedArtifact,
    ]
}

pub(super) fn worth_proof_required_surfaces() -> Vec<FoundationalProfileWORTHProofSurface> {
    vec![
        FoundationalProfileWORTHProofSurface::ArtifactCarrier,
        FoundationalProfileWORTHProofSurface::TransitionOutcome,
        FoundationalProfileWORTHProofSurface::AuthorityWitness,
        FoundationalProfileWORTHProofSurface::BoundaryBridgeTrustBoundary,
        FoundationalProfileWORTHProofSurface::BoundaryReadmitWithAuthority,
        FoundationalProfileWORTHProofSurface::CurrentBasisArtifactConstructor,
    ]
}

pub(super) fn worth_proof_api_appendix() -> Vec<FoundationalProfileWORTHProofApi> {
    vec![
        FoundationalProfileWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
        FoundationalProfileWORTHProofApi::ArtifactNew,
        FoundationalProfileWORTHProofApi::ArtifactWithCurrentBasis,
        FoundationalProfileWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
        FoundationalProfileWORTHProofApi::TransitionOutcomeStructuredCategories,
        FoundationalProfileWORTHProofApi::ArtifactBridgeTrustBoundary,
        FoundationalProfileWORTHProofApi::ArtifactReadmitWithAuthority,
    ]
}

pub(super) fn worth_proof_forbidden_surfaces() -> Vec<FoundationalProfileWORTHProofForbiddenSurface>
{
    vec![
        FoundationalProfileWORTHProofForbiddenSurface::PlainProfileFamilyVocabulary,
        FoundationalProfileWORTHProofForbiddenSurface::PlainProfileCompositionData,
        FoundationalProfileWORTHProofForbiddenSurface::PlainDescriptiveSurfaceVocabulary,
        FoundationalProfileWORTHProofForbiddenSurface::PlainProfileIdentityBasisEntries,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalProfileRuntimeAssumption> {
    vec![
        FoundationalProfileRuntimeAssumption::CanonicalBasisLawCertified,
        FoundationalProfileRuntimeAssumption::ProfileMeaningRemainsFacadeControlled,
        FoundationalProfileRuntimeAssumption::ReducedRichnessAffectsOnlyOptionalDescriptiveSurfaces,
        FoundationalProfileRuntimeAssumption::ProofBearingCertificationUsesExplicitAuthorityProgression,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalProfileRuntimeNonAssumption> {
    vec![
        FoundationalProfileRuntimeNonAssumption::RuntimePolicyExecutionExistsInFoundational,
        FoundationalProfileRuntimeNonAssumption::AdoptingCrateLoweringParityAlreadyProven,
        FoundationalProfileRuntimeNonAssumption::DiagnosticsOrProvenanceOntologyAlreadyOwnedHere,
        FoundationalProfileRuntimeNonAssumption::BoundaryCrossingPreservesStrongerCertificationWithoutReadmission,
    ]
}

pub(super) fn residual_debt() -> Vec<FoundationalProfileResidualDebt> {
    vec![
        FoundationalProfileResidualDebt::AdoptingCrateParityDeferred,
        FoundationalProfileResidualDebt::RealRuntimePolicyLoweringDeferred,
        FoundationalProfileResidualDebt::LaterArtifactDiagnosticsAndProvenanceOntologyDeferred,
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalProfilePhaseGateEvidence> {
    vec![
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::TypedFamilies,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/composition/mod.rs"),
        ),
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::ComposedProfileSet,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/composition/mod.rs"),
        ),
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::ProgressionAndAttachment,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/progression_and_attachment/mod.rs"),
        ),
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::CanonicalIdentityAndDifference,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/identity_and_difference/mod.rs"),
        ),
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::MaterializationAndElision,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/materialization/mod.rs"),
        ),
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::CertificationStrengthening,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/certification_posture/mod.rs"),
        ),
        FoundationalProfilePhaseGateEvidence::new(
            FoundationalProfileMilestone3PhaseGate::ProductionReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/readiness/mod.rs"),
        ),
    ]
}

pub(super) fn public_surface_inventory() -> Vec<FoundationalProfilePublicSurfaceEntry> {
    profile_public_surface_inventory().to_vec()
}

pub(super) const fn public_surface_evidence_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/profiles/grouped_surface/mod.rs")
}

pub(super) const fn public_surface_compile_fail_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/profiles/grouped_surface/raw_profile_set_cannot_enter_grouped_common_path_progression.rs")
}
