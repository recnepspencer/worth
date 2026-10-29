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
            "tests/worth_foundational/fast/certification/diagnostics/primitives.rs",
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::FamilyDistinctRowTopologyMeaning,
            "tests/worth_foundational/fast/certification/diagnostics/rows.rs",
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::MaterializationRichnessAndDebtMeaning,
            "tests/worth_foundational/fast/certification/diagnostics/materialization.rs",
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::CanonicalBundleAndComparisonMeaning,
            "tests/worth_foundational/fast/certification/diagnostics/basis.rs",
        ),
        FoundationalDiagnosticCanonicalGoldenArtifactEvidence::new(
            FoundationalDiagnosticCanonicalGoldenArtifact::CertifiedCoverageAndAttachmentMeaning,
            "tests/worth_foundational/fast/certification/diagnostics/certified.rs",
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
            "tests/worth_foundational/fast/certification/diagnostics/primitives.rs",
            "ordering hostility and canonical token-segment rejection",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::RowFamilyOrderingAndSemanticTieBreaks,
            "tests/worth_foundational/fast/certification/diagnostics/basis.rs",
            "tied-common-field row ordering must still canonicalize by family-specific meaning",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::RichnessElisionPreservesTruthUnderPartiality,
            "tests/worth_foundational/fast/certification/diagnostics/materialization.rs",
            "reduced richness narrows breadth while preserving outcome truth and explicit partiality",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::IndependentProducerCanonicalParity,
            "tests/worth_foundational/fast/certification/diagnostics/basis.rs",
            "independent producer layouts and input order must still lower to one diagnostic meaning",
        ),
        FoundationalDiagnosticPropertySeedEvidence::new(
            FoundationalDiagnosticPropertySeed::CertifiedCoverageNamedGapParity,
            "tests/worth_foundational/fast/certification/diagnostics/certified.rs",
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
            "tests/worth_foundational/fast/certification/diagnostics/basis.rs",
        ),
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::RichnessAvailabilityAndFallbackReplayMatrix,
            "tests/worth_foundational/fast/certification/diagnostics/materialization.rs",
        ),
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::BlindConsumerInterpretationReplaySuite,
            "tests/worth_foundational/fast/certification/diagnostics/rows.rs",
        ),
        FoundationalDiagnosticHarnessExpansionEvidence::new(
            FoundationalDiagnosticHarnessExpansionPoint::CertifiedCoverageAttachmentParityMatrix,
            "tests/worth_foundational/fast/certification/diagnostics/certified.rs",
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
