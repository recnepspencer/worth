use super::production_test_contract::{
    FoundationalDiagnosticAdoptionShapedFollowthrough,
    FoundationalDiagnosticCanonicalGoldenArtifact,
    FoundationalDiagnosticCanonicalGoldenArtifactEvidence,
    FoundationalDiagnosticHarnessExpansionEvidence, FoundationalDiagnosticHarnessExpansionPoint,
    FoundationalDiagnosticPropertySeed, FoundationalDiagnosticPropertySeedEvidence,
    FoundationalDiagnosticRuntimeAdoptionFailurePressure,
};

pub(super) fn canonical_golden_artifacts() -> Vec<FoundationalDiagnosticCanonicalGoldenArtifact> {
    vec![
        FoundationalDiagnosticCanonicalGoldenArtifact::PrimitiveCategoryAndMaterializationMeaning,
        FoundationalDiagnosticCanonicalGoldenArtifact::FamilyDistinctRowTopologyMeaning,
        FoundationalDiagnosticCanonicalGoldenArtifact::MaterializationRichnessAndDebtMeaning,
        FoundationalDiagnosticCanonicalGoldenArtifact::CanonicalBundleAndComparisonMeaning,
        FoundationalDiagnosticCanonicalGoldenArtifact::CertifiedCoverageAndAttachmentMeaning,
    ]
}

pub(super) fn canonical_golden_artifact_evidence(
) -> Vec<FoundationalDiagnosticCanonicalGoldenArtifactEvidence> {
    vec![
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::PrimitiveCategoryAndMaterializationMeaning,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/primitives/mod.rs"),
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::FamilyDistinctRowTopologyMeaning,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/rows/mod.rs"),
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::MaterializationRichnessAndDebtMeaning,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::CanonicalBundleAndComparisonMeaning,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::CertifiedCoverageAndAttachmentMeaning,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
        ),
    ]
}

pub(super) fn property_seed_inventory() -> Vec<FoundationalDiagnosticPropertySeed> {
    vec![
        FoundationalDiagnosticPropertySeed::PrimitiveOrderingAndTokenCanonicalization,
        FoundationalDiagnosticPropertySeed::RowFamilyOrderingAndSemanticTieBreaks,
        FoundationalDiagnosticPropertySeed::RichnessElisionPreservesTruthUnderPartiality,
        FoundationalDiagnosticPropertySeed::IndependentProducerCanonicalParity,
        FoundationalDiagnosticPropertySeed::CertifiedCoverageNamedGapParity,
    ]
}

pub(super) fn property_seed_evidence() -> Vec<FoundationalDiagnosticPropertySeedEvidence> {
    vec![
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::PrimitiveOrderingAndTokenCanonicalization,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/primitives/mod.rs"),
            "ordering hostility and canonical token-segment rejection",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::RowFamilyOrderingAndSemanticTieBreaks,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
            "tied-common-field row ordering must still canonicalize by family-specific meaning",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::RichnessElisionPreservesTruthUnderPartiality,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
            "reduced richness narrows breadth while preserving outcome truth and explicit partiality",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::IndependentProducerCanonicalParity,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
            "independent producer layouts and input order must still lower to one diagnostic meaning",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::CertifiedCoverageNamedGapParity,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
            "partial coverage and typed named gaps must stay canonical under certified attachment pressure",
        ),
    ]
}

pub(super) fn harness_expansion_points() -> Vec<FoundationalDiagnosticHarnessExpansionPoint> {
    vec![
        FoundationalDiagnosticHarnessExpansionPoint::IndependentProducerDiagnosticParityMatrix,
        FoundationalDiagnosticHarnessExpansionPoint::RichnessAvailabilityAndFallbackReplayMatrix,
        FoundationalDiagnosticHarnessExpansionPoint::BlindConsumerInterpretationReplaySuite,
        FoundationalDiagnosticHarnessExpansionPoint::CertifiedCoverageAttachmentParityMatrix,
    ]
}

pub(super) fn harness_expansion_evidence() -> Vec<FoundationalDiagnosticHarnessExpansionEvidence> {
    vec![
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::IndependentProducerDiagnosticParityMatrix,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/basis/mod.rs"),
        ),
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::RichnessAvailabilityAndFallbackReplayMatrix,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/materialization/mod.rs"),
        ),
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::BlindConsumerInterpretationReplaySuite,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/rows/mod.rs"),
        ),
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::CertifiedCoverageAttachmentParityMatrix,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-expressions/tests/worth_court_foundational_expressions/foundational/certification/diagnostics/certified/mod.rs"),
        ),
    ]
}

pub(super) fn runtime_adoption_failure_pressures(
) -> Vec<FoundationalDiagnosticRuntimeAdoptionFailurePressure> {
    vec![
        FoundationalDiagnosticRuntimeAdoptionFailurePressure::RuntimeLoweringMayMisclassifyEvidencePosture,
        FoundationalDiagnosticRuntimeAdoptionFailurePressure::RuntimeMaterializersMayOverclaimDurableOrCertifiedSupport,
        FoundationalDiagnosticRuntimeAdoptionFailurePressure::RuntimeCanonicalRowOrderingMayDriftAcrossStorageLayouts,
        FoundationalDiagnosticRuntimeAdoptionFailurePressure::RuntimeCoverageMatricesMayOmitRequiredFamilies,
        FoundationalDiagnosticRuntimeAdoptionFailurePressure::RuntimeProvenanceReadyRowsMayCollapseIntoExplanationRows,
    ]
}

pub(super) fn adoption_shaped_followthrough(
) -> Vec<FoundationalDiagnosticAdoptionShapedFollowthrough> {
    vec![
        FoundationalDiagnosticAdoptionShapedFollowthrough::WORTHHarnessDiagnosticProducerParityMatrix,
        FoundationalDiagnosticAdoptionShapedFollowthrough::WORTHHarnessRichnessAvailabilityAndFallbackReplaySuite,
        FoundationalDiagnosticAdoptionShapedFollowthrough::AdoptingRuntimeDiagnosticLoweringParityPressure,
        FoundationalDiagnosticAdoptionShapedFollowthrough::AdoptingRuntimeCertifiedCoverageAndAttachmentHostility,
    ]
}
