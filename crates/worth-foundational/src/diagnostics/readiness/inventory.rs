use super::vocabulary::{
    FoundationalDiagnosticCertifiedSurface, FoundationalDiagnosticCertifiedSurfaceEvidence,
    FoundationalDiagnosticCompileFailBoundary, FoundationalDiagnosticCompileFailEvidence,
    FoundationalDiagnosticMilestone6PhaseGate, FoundationalDiagnosticPhaseGateEvidence,
    FoundationalDiagnosticResidualDebt, FoundationalDiagnosticRuntimeAssumption,
    FoundationalDiagnosticRuntimeNonAssumption, FoundationalDiagnosticSyntheticPressureEvidence,
    FoundationalDiagnosticSyntheticRuntimePressure, FoundationalDiagnosticWORTHProofApi,
    FoundationalDiagnosticWORTHProofApiEvidence, FoundationalDiagnosticWORTHProofForbiddenSurface,
    FoundationalDiagnosticWORTHProofSurface,
};

pub(super) fn certified_surfaces() -> Vec<FoundationalDiagnosticCertifiedSurface> {
    vec![
        FoundationalDiagnosticCertifiedSurface::PrimitiveAndCategoryLaw,
        FoundationalDiagnosticCertifiedSurface::OutcomeSubjectAndRowTopology,
        FoundationalDiagnosticCertifiedSurface::MaterializationSupportAndNamedGapLaw,
        FoundationalDiagnosticCertifiedSurface::CanonicalBasisAndComparisonLaw,
        FoundationalDiagnosticCertifiedSurface::CertifiedBundleAndAttachmentCompatibility,
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalDiagnosticSyntheticRuntimePressure> {
    vec![
        FoundationalDiagnosticSyntheticRuntimePressure::PrimitiveNonSubstitution,
        FoundationalDiagnosticSyntheticRuntimePressure::GenericRowCollapseRejection,
        FoundationalDiagnosticSyntheticRuntimePressure::HiddenRediscoveryDebtRejection,
        FoundationalDiagnosticSyntheticRuntimePressure::ThinOrEmptySupportOverclaimRejection,
        FoundationalDiagnosticSyntheticRuntimePressure::BlindConsumerCanonicalParity,
        FoundationalDiagnosticSyntheticRuntimePressure::HiddenSourceDigestOrCoverageWORTHry,
        FoundationalDiagnosticSyntheticRuntimePressure::ExplanationProvenanceBoundaryPreservation,
    ]
}

pub(super) fn certified_surface_evidence() -> Vec<FoundationalDiagnosticCertifiedSurfaceEvidence> {
    vec![
        FoundationalDiagnosticCertifiedSurfaceEvidence::new(
            FoundationalDiagnosticCertifiedSurface::PrimitiveAndCategoryLaw,
            FoundationalDiagnosticSyntheticRuntimePressure::PrimitiveNonSubstitution,
            FoundationalDiagnosticCompileFailBoundary::PrimitiveAndCategoryPreserveNonSubstitution,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/primitives/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/primitives/scope_cannot_satisfy_code_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/primitives/mod.rs"),
        ),
        FoundationalDiagnosticCertifiedSurfaceEvidence::new(
            FoundationalDiagnosticCertifiedSurface::OutcomeSubjectAndRowTopology,
            FoundationalDiagnosticSyntheticRuntimePressure::GenericRowCollapseRejection,
            FoundationalDiagnosticCompileFailBoundary::RowTopologyPreservesFamilyAndLocatorLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/rows/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/rows/aggregate_row_cannot_satisfy_decision_row_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/rows/mod.rs"),
        ),
        FoundationalDiagnosticCertifiedSurfaceEvidence::new(
            FoundationalDiagnosticCertifiedSurface::MaterializationSupportAndNamedGapLaw,
            FoundationalDiagnosticSyntheticRuntimePressure::ThinOrEmptySupportOverclaimRejection,
            FoundationalDiagnosticCompileFailBoundary::MaterializationAndSupportPreserveExplicitSeams,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/materialization/explanation_bundle_cannot_satisfy_support_report_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
        ),
        FoundationalDiagnosticCertifiedSurfaceEvidence::new(
            FoundationalDiagnosticCertifiedSurface::CanonicalBasisAndComparisonLaw,
            FoundationalDiagnosticSyntheticRuntimePressure::BlindConsumerCanonicalParity,
            FoundationalDiagnosticCompileFailBoundary::BasisAndComparisonPreserveBlindConsumerCanonicalLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/basis/raw_support_report_cannot_satisfy_diagnostic_comparison_bundle_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
        ),
        FoundationalDiagnosticCertifiedSurfaceEvidence::new(
            FoundationalDiagnosticCertifiedSurface::CertifiedBundleAndAttachmentCompatibility,
            FoundationalDiagnosticSyntheticRuntimePressure::HiddenSourceDigestOrCoverageWORTHry,
            FoundationalDiagnosticCompileFailBoundary::CertifiedBundleAndAttachmentReuseProofLane,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/certified/raw_support_report_cannot_satisfy_certified_bundle_api.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
        ),
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalDiagnosticCompileFailBoundary> {
    vec![
        FoundationalDiagnosticCompileFailBoundary::PrimitiveAndCategoryPreserveNonSubstitution,
        FoundationalDiagnosticCompileFailBoundary::RowTopologyPreservesFamilyAndLocatorLaw,
        FoundationalDiagnosticCompileFailBoundary::MaterializationAndSupportPreserveExplicitSeams,
        FoundationalDiagnosticCompileFailBoundary::BasisAndComparisonPreserveBlindConsumerCanonicalLaw,
        FoundationalDiagnosticCompileFailBoundary::CertifiedBundleAndAttachmentReuseProofLane,
        FoundationalDiagnosticCompileFailBoundary::DiagnosticReadinessRequiresCertifiedArtifact,
        FoundationalDiagnosticCompileFailBoundary::DiagnosticReadinessAuthorityCannotBeMinted,
    ]
}

pub(super) fn worth_proof_required_surfaces() -> Vec<FoundationalDiagnosticWORTHProofSurface> {
    vec![
        FoundationalDiagnosticWORTHProofSurface::CertifiedDiagnosticAttachmentAuthority,
        FoundationalDiagnosticWORTHProofSurface::ProofBearingCertifiedDiagnosticBundle,
        FoundationalDiagnosticWORTHProofSurface::CertifiedBundleBoundaryBridge,
        FoundationalDiagnosticWORTHProofSurface::CertifiedBundleReadmitWithAuthority,
        FoundationalDiagnosticWORTHProofSurface::ProductionReadinessCertificationArtifact,
    ]
}

pub(super) fn worth_proof_api_appendix() -> Vec<FoundationalDiagnosticWORTHProofApi> {
    vec![
        FoundationalDiagnosticWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
        FoundationalDiagnosticWORTHProofApi::ProofFromAuthorityWitness,
        FoundationalDiagnosticWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
        FoundationalDiagnosticWORTHProofApi::ArtifactBridgeTrustBoundary,
        FoundationalDiagnosticWORTHProofApi::ArtifactReadmitWithAuthority,
    ]
}

pub(super) fn worth_proof_api_evidence() -> Vec<FoundationalDiagnosticWORTHProofApiEvidence> {
    vec![
        FoundationalDiagnosticWORTHProofApiEvidence::new(
            FoundationalDiagnosticWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
            "src/diagnostics/readiness/certification.rs",
            "AuthorityWitness::from_authority_marker",
        ),
        FoundationalDiagnosticWORTHProofApiEvidence::new(
            FoundationalDiagnosticWORTHProofApi::ProofFromAuthorityWitness,
            "src/diagnostics/readiness/certification.rs",
            "Proof::from_authority_witness",
        ),
        FoundationalDiagnosticWORTHProofApiEvidence::new(
            FoundationalDiagnosticWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
            "src/diagnostics/certified/attachments.rs",
            "Artifact::with_proofs_and_current_basis",
        ),
        FoundationalDiagnosticWORTHProofApiEvidence::new(
            FoundationalDiagnosticWORTHProofApi::ArtifactBridgeTrustBoundary,
            "src/diagnostics/certified/surfaces.rs",
            ".bridge_trust_boundary()",
        ),
        FoundationalDiagnosticWORTHProofApiEvidence::new(
            FoundationalDiagnosticWORTHProofApi::ArtifactReadmitWithAuthority,
            "src/diagnostics/certified/surfaces.rs",
            ".readmit_with_authority(",
        ),
    ]
}

pub(super) fn worth_proof_forbidden_surfaces(
) -> Vec<FoundationalDiagnosticWORTHProofForbiddenSurface> {
    vec![
        FoundationalDiagnosticWORTHProofForbiddenSurface::PlainDiagnosticPrimitives,
        FoundationalDiagnosticWORTHProofForbiddenSurface::PlainDiagnosticRowsAndBundles,
        FoundationalDiagnosticWORTHProofForbiddenSurface::PlainMaterializationVocabulary,
        FoundationalDiagnosticWORTHProofForbiddenSurface::PlainCanonicalComparisonVocabulary,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalDiagnosticRuntimeAssumption> {
    vec![
        FoundationalDiagnosticRuntimeAssumption::Milestone2CanonicalizationRemainsAuthorityForDiagnosticBasis,
        FoundationalDiagnosticRuntimeAssumption::Milestone3ProfilesGovernRichnessSupportAndCertificationPosture,
        FoundationalDiagnosticRuntimeAssumption::Milestone4ArtifactLawGovernsDiagnosticCategoryAndDeliveryMeaning,
        FoundationalDiagnosticRuntimeAssumption::Milestone5TransitionAndCurrentBasisSurfacesRemainAuthorityForTransitionAttachedDiagnostics,
        FoundationalDiagnosticRuntimeAssumption::CertifiedDiagnosticBundlesReuseWORTHProofLane,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalDiagnosticRuntimeNonAssumption> {
    vec![
        FoundationalDiagnosticRuntimeNonAssumption::Milestone7ProvenanceAndReceiptOntologyAlreadyOwnedHere,
        FoundationalDiagnosticRuntimeNonAssumption::OneDiagnosticsStoreOrReplayEngineExistsInFoundational,
        FoundationalDiagnosticRuntimeNonAssumption::AdoptingRuntimeCoverageParityAlreadyProven,
        FoundationalDiagnosticRuntimeNonAssumption::DescriptiveDiagnosticsBecomeAuthority,
        FoundationalDiagnosticRuntimeNonAssumption::BoundaryCrossingPreservesCertifiedCurrentBasisWithoutReadmission,
    ]
}

pub(super) fn residual_debt() -> Vec<FoundationalDiagnosticResidualDebt> {
    vec![
        FoundationalDiagnosticResidualDebt::AdoptingRuntimeParityDeferred,
        FoundationalDiagnosticResidualDebt::Milestone7ProvenanceAndReceiptDeepeningDeferred,
        FoundationalDiagnosticResidualDebt::RuntimeSpecificSupportTaxonomiesDeferred,
    ]
}

pub(super) fn synthetic_pressure_evidence() -> Vec<FoundationalDiagnosticSyntheticPressureEvidence>
{
    vec![
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::PrimitiveNonSubstitution,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/primitives/mod.rs"),
        ),
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::GenericRowCollapseRejection,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/rows/mod.rs"),
        ),
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::HiddenRediscoveryDebtRejection,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
        ),
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::ThinOrEmptySupportOverclaimRejection,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
        ),
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::BlindConsumerCanonicalParity,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
        ),
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::HiddenSourceDigestOrCoverageWORTHry,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
        ),
        FoundationalDiagnosticSyntheticPressureEvidence::new(
            FoundationalDiagnosticSyntheticRuntimePressure::ExplanationProvenanceBoundaryPreservation,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
        ),
    ]
}

pub(super) fn compile_fail_evidence() -> Vec<FoundationalDiagnosticCompileFailEvidence> {
    vec![
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::PrimitiveAndCategoryPreserveNonSubstitution,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/primitives/scope_cannot_satisfy_code_api.rs"),
        ),
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::RowTopologyPreservesFamilyAndLocatorLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/rows/aggregate_row_cannot_satisfy_decision_row_api.rs"),
        ),
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::MaterializationAndSupportPreserveExplicitSeams,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/materialization/explanation_bundle_cannot_satisfy_support_report_api.rs"),
        ),
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::BasisAndComparisonPreserveBlindConsumerCanonicalLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/basis/raw_support_report_cannot_satisfy_diagnostic_comparison_bundle_api.rs"),
        ),
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::CertifiedBundleAndAttachmentReuseProofLane,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/certified/raw_support_report_cannot_satisfy_certified_bundle_api.rs"),
        ),
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::DiagnosticReadinessRequiresCertifiedArtifact,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/readiness_boundaries/plain_report_cannot_satisfy_diagnostic_production_readiness.rs"),
        ),
        FoundationalDiagnosticCompileFailEvidence::new(
            FoundationalDiagnosticCompileFailBoundary::DiagnosticReadinessAuthorityCannotBeMinted,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/diagnostics/readiness_boundaries/diagnostic_readiness_authority_cannot_be_minted.rs"),
        ),
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalDiagnosticPhaseGateEvidence> {
    vec![
        FoundationalDiagnosticPhaseGateEvidence::new(
            FoundationalDiagnosticMilestone6PhaseGate::PrimitiveAndCategoryLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/primitives/mod.rs"),
        ),
        FoundationalDiagnosticPhaseGateEvidence::new(
            FoundationalDiagnosticMilestone6PhaseGate::OutcomeSubjectAndRowTopology,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/rows/mod.rs"),
        ),
        FoundationalDiagnosticPhaseGateEvidence::new(
            FoundationalDiagnosticMilestone6PhaseGate::MaterializationSupportAndNamedGapLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
        ),
        FoundationalDiagnosticPhaseGateEvidence::new(
            FoundationalDiagnosticMilestone6PhaseGate::CanonicalBasisAndComparisonLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
        ),
        FoundationalDiagnosticPhaseGateEvidence::new(
            FoundationalDiagnosticMilestone6PhaseGate::CertifiedBundleAndAttachmentCompatibility,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
        ),
        FoundationalDiagnosticPhaseGateEvidence::new(
            FoundationalDiagnosticMilestone6PhaseGate::ProductionReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/readiness/mod.rs"),
        ),
    ]
}
