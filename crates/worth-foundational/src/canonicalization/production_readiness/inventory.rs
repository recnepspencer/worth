use super::vocabulary::{
    CanonicalCertifiedSurface, CanonicalCertifiedSurfaceEvidence, CanonicalCompileFailBoundary,
    CanonicalCostCounterEvidence, CanonicalFixtureManifestEvidence,
    CanonicalGoldenArtifactEvidence, CanonicalHarnessExpansionPoint, CanonicalMilestone2PhaseGate,
    CanonicalPhaseGateEvidence, CanonicalPropertySeed, CanonicalPropertySeedEvidence,
    CanonicalResidualDebt, CanonicalRuntimeAssumption, CanonicalRuntimeNonAssumption,
    CanonicalSyntheticRuntimePressure,
};
use crate::canonicalization_api::{
    canonical_public_surface_inventory, CanonicalPublicSurfaceEntry,
};

pub(super) fn certified_surfaces() -> Vec<CanonicalCertifiedSurface> {
    vec![
        CanonicalCertifiedSurface::BasisGrammar,
        CanonicalCertifiedSurface::MilestoneOneBasisBuilders,
        CanonicalCertifiedSurface::EquivalenceBasis,
        CanonicalCertifiedSurface::MismatchBasis,
        CanonicalCertifiedSurface::ExportBundles,
        CanonicalCertifiedSurface::DigestAlgorithmSlots,
    ]
}

pub(super) fn certified_surface_evidence() -> Vec<CanonicalCertifiedSurfaceEvidence> {
    vec![
        CanonicalCertifiedSurfaceEvidence::new(
            CanonicalCertifiedSurface::BasisGrammar,
            CanonicalSyntheticRuntimePressure::OrderedAuthorityProducer,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/basis/basis_grammar/mod.rs"),
            CanonicalCompileFailBoundary::RawBasisCannotSatisfyReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/canonicalization/basis/readiness/raw_sequence_cannot_satisfy_ready_basis_api.rs"),
            CanonicalCostCounterEvidence::BasisSequenceConstruction,
            Some(CanonicalGoldenArtifactEvidence::AspectContractBasis),
        ),
        CanonicalCertifiedSurfaceEvidence::new(
            CanonicalCertifiedSurface::MilestoneOneBasisBuilders,
            CanonicalSyntheticRuntimePressure::ReorderedCompatibilityProducer,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/digest_preparation/canonical_basis_builders/mod.rs"),
            CanonicalCompileFailBoundary::RawBasisCannotSatisfyReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/digest_preparation/non_ready_state_cannot_satisfy_digest_basis_api.rs"),
            CanonicalCostCounterEvidence::MilestoneOneSurfaceLowering,
            Some(CanonicalGoldenArtifactEvidence::AuthoritativeStateBasis),
        ),
        CanonicalCertifiedSurfaceEvidence::new(
            CanonicalCertifiedSurface::EquivalenceBasis,
            CanonicalSyntheticRuntimePressure::BlindMismatchConsumer,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/equivalence/comparison_readiness/mod.rs"),
            CanonicalCompileFailBoundary::RawComparisonCannotSatisfyEquivalence,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/canonicalization/equivalence/raw_ready_basis_cannot_satisfy_comparison_api.rs"),
            CanonicalCostCounterEvidence::BasisSequenceConstruction,
            Some(CanonicalGoldenArtifactEvidence::EquivalenceBasis),
        ),
        CanonicalCertifiedSurfaceEvidence::new(
            CanonicalCertifiedSurface::MismatchBasis,
            CanonicalSyntheticRuntimePressure::BlindMismatchConsumer,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/equivalence/comparison_readiness/mod.rs"),
            CanonicalCompileFailBoundary::RawComparisonCannotSatisfyEquivalence,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/canonicalization/equivalence/raw_ready_basis_cannot_satisfy_comparison_api.rs"),
            CanonicalCostCounterEvidence::BasisSequenceConstruction,
            Some(CanonicalGoldenArtifactEvidence::MismatchBasis),
        ),
        CanonicalCertifiedSurfaceEvidence::new(
            CanonicalCertifiedSurface::ExportBundles,
            CanonicalSyntheticRuntimePressure::SupportExportConsumer,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/export/export_ready_fixtures/mod.rs"),
            CanonicalCompileFailBoundary::BoundaryExportRequiresReadmission,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/canonicalization/export/boundary_bridged_export_cannot_satisfy_current_export_api.rs"),
            CanonicalCostCounterEvidence::ExportManifestRows,
            Some(CanonicalGoldenArtifactEvidence::ExportBundleManifest),
        ),
        CanonicalCertifiedSurfaceEvidence::new(
            CanonicalCertifiedSurface::DigestAlgorithmSlots,
            CanonicalSyntheticRuntimePressure::CategoryAdjacentHostileProducer,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/digest_slots/digest_derivation/mod.rs"),
            CanonicalCompileFailBoundary::DigestDerivationRequiresReadyArtifact,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/canonicalization/digest_slots/raw_bytes_cannot_satisfy_digest_derivation.rs"),
            CanonicalCostCounterEvidence::DigestInputMetadata,
            Some(CanonicalGoldenArtifactEvidence::DigestSlotDerivedValue),
        ),
    ]
}

pub(super) fn synthetic_pressures() -> Vec<CanonicalSyntheticRuntimePressure> {
    vec![
        CanonicalSyntheticRuntimePressure::OrderedAuthorityProducer,
        CanonicalSyntheticRuntimePressure::ReorderedCompatibilityProducer,
        CanonicalSyntheticRuntimePressure::SupportExportConsumer,
        CanonicalSyntheticRuntimePressure::CategoryAdjacentHostileProducer,
        CanonicalSyntheticRuntimePressure::BlindMismatchConsumer,
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<CanonicalCompileFailBoundary> {
    vec![
        CanonicalCompileFailBoundary::RawBasisCannotSatisfyReadiness,
        CanonicalCompileFailBoundary::RawComparisonCannotSatisfyEquivalence,
        CanonicalCompileFailBoundary::BoundaryExportRequiresReadmission,
        CanonicalCompileFailBoundary::DigestDerivationRequiresReadyArtifact,
        CanonicalCompileFailBoundary::ProductionReadinessRequiresCertifiedArtifact,
    ]
}

pub(super) fn golden_artifacts() -> Vec<CanonicalGoldenArtifactEvidence> {
    vec![
        CanonicalGoldenArtifactEvidence::ValueFamilies,
        CanonicalGoldenArtifactEvidence::AspectContractBasis,
        CanonicalGoldenArtifactEvidence::AspectMaskBasis,
        CanonicalGoldenArtifactEvidence::AuthoritativeStateBasis,
        CanonicalGoldenArtifactEvidence::AuthoritativePatchBasis,
        CanonicalGoldenArtifactEvidence::CompatibilityLoweredStateBasis,
        CanonicalGoldenArtifactEvidence::IdentityAndLocator,
        CanonicalGoldenArtifactEvidence::EquivalenceBasis,
        CanonicalGoldenArtifactEvidence::MismatchBasis,
        CanonicalGoldenArtifactEvidence::ExportBundleManifest,
        CanonicalGoldenArtifactEvidence::DigestSlotDerivedValue,
    ]
}

pub(super) fn property_seeds() -> Vec<CanonicalPropertySeed> {
    vec![
        CanonicalPropertySeed::OrderingIndependence,
        CanonicalPropertySeed::CategoryAdjacency,
        CanonicalPropertySeed::CompatibilityLoweringParity,
        CanonicalPropertySeed::EquivalenceScope,
        CanonicalPropertySeed::MismatchLocus,
        CanonicalPropertySeed::DigestSlotHostility,
    ]
}

pub(super) fn property_seed_evidence() -> Vec<CanonicalPropertySeedEvidence> {
    vec![
        CanonicalPropertySeedEvidence::new(
            CanonicalPropertySeed::OrderingIndependence,
            "basis entries canonicalize across insertion and builder order",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/basis/basis_grammar/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalPropertySeedEvidence::new(
            CanonicalPropertySeed::CategoryAdjacency,
            "storage-equal values in adjacent semantic categories remain distinct",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/digest_slots/digest_derivation/mod.rs"),
            CanonicalHarnessExpansionPoint::DigestSlotHostilityLane,
        ),
        CanonicalPropertySeedEvidence::new(
            CanonicalPropertySeed::CompatibilityLoweringParity,
            "native state and compatibility-lowered state share canonical truth",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/digest_preparation/canonical_basis_builders/mod.rs"),
            CanonicalHarnessExpansionPoint::RuntimeParityRunMatrix,
        ),
        CanonicalPropertySeedEvidence::new(
            CanonicalPropertySeed::EquivalenceScope,
            "exact and compatibility-lowered equivalence are declared before comparison",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/equivalence/comparison_readiness/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalPropertySeedEvidence::new(
            CanonicalPropertySeed::MismatchLocus,
            "blind consumers receive smallest canonical mismatch loci",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/equivalence/comparison_readiness/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalPropertySeedEvidence::new(
            CanonicalPropertySeed::DigestSlotHostility,
            "wrong domain, shape, version, and raw-byte digest inputs fail closed",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/digest_slots/digest_derivation/mod.rs"),
            CanonicalHarnessExpansionPoint::DigestSlotHostilityLane,
        ),
    ]
}

pub(super) fn cost_counter_evidence() -> Vec<CanonicalCostCounterEvidence> {
    vec![
        CanonicalCostCounterEvidence::BasisSequenceConstruction,
        CanonicalCostCounterEvidence::MilestoneOneSurfaceLowering,
        CanonicalCostCounterEvidence::ExportManifestRows,
        CanonicalCostCounterEvidence::DigestInputMetadata,
    ]
}

pub(super) fn harness_expansion_points() -> Vec<CanonicalHarnessExpansionPoint> {
    vec![
        CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        CanonicalHarnessExpansionPoint::ExportFixtureReplayLane,
        CanonicalHarnessExpansionPoint::DigestSlotHostilityLane,
        CanonicalHarnessExpansionPoint::RuntimeParityRunMatrix,
        CanonicalHarnessExpansionPoint::GroupedPublicSurfaceLane,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<CanonicalRuntimeAssumption> {
    vec![
        CanonicalRuntimeAssumption::FoundationalBasisLawCertified,
        CanonicalRuntimeAssumption::CanonicalOrderingStable,
        CanonicalRuntimeAssumption::ComparisonAndMismatchEvidenceSelfDescribing,
        CanonicalRuntimeAssumption::DigestDerivationGatedByReadiness,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<CanonicalRuntimeNonAssumption> {
    vec![
        CanonicalRuntimeNonAssumption::RealRuntimeLoweringCorrect,
        CanonicalRuntimeNonAssumption::FinalCryptographicPolicySelected,
        CanonicalRuntimeNonAssumption::ProfilesReceiptsDiagnosticsOrBranchOntologyExists,
        CanonicalRuntimeNonAssumption::DigestEqualityAuthorizesSemanticEquivalence,
    ]
}

pub(super) fn residual_debt() -> Vec<CanonicalResidualDebt> {
    vec![
        CanonicalResidualDebt::FinalCryptographicPolicyDeferred,
        CanonicalResidualDebt::RealRuntimeAdoptionParityDeferred,
        CanonicalResidualDebt::LaterMilestoneOntologyDeferred,
    ]
}

pub(super) fn phase_gates() -> Vec<CanonicalPhaseGateEvidence> {
    vec![
        CanonicalPhaseGateEvidence::new(
            CanonicalMilestone2PhaseGate::BasisGrammar,
            "tests/worth_foundational/fast/certification/canonicalization/basis",
        ),
        CanonicalPhaseGateEvidence::new(
            CanonicalMilestone2PhaseGate::MilestoneOneBasisBuilders,
            "tests/worth_foundational/fast/certification/canonicalization/digest_preparation",
        ),
        CanonicalPhaseGateEvidence::new(
            CanonicalMilestone2PhaseGate::EquivalenceAndMismatch,
            "tests/worth_foundational/fast/certification/canonicalization/equivalence",
        ),
        CanonicalPhaseGateEvidence::new(
            CanonicalMilestone2PhaseGate::ExportFixtures,
            "tests/worth_foundational/fast/certification/canonicalization/export",
        ),
        CanonicalPhaseGateEvidence::new(
            CanonicalMilestone2PhaseGate::DigestSlots,
            "tests/worth_foundational/fast/certification/canonicalization/digest_slots",
        ),
        CanonicalPhaseGateEvidence::new(
            CanonicalMilestone2PhaseGate::ProductionReadiness,
            "tests/worth_foundational/fast/certification/canonicalization/production_readiness",
        ),
    ]
}

pub(super) fn fixture_manifest() -> Vec<CanonicalFixtureManifestEvidence> {
    vec![
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::ValueFamilies,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/value_families/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::AspectContractBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/boundary_digest_bases/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::AspectMaskBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/boundary_digest_bases/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::AuthoritativeStateBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/boundary_digest_bases/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::AuthoritativePatchBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/boundary_digest_bases/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::CompatibilityLoweredStateBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/boundary_digest_bases/mod.rs"),
            CanonicalHarnessExpansionPoint::RuntimeParityRunMatrix,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::IdentityAndLocator,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/identity_and_locator/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::EquivalenceBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/equivalence_and_mismatch/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::MismatchBasis,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/equivalence_and_mismatch/mod.rs"),
            CanonicalHarnessExpansionPoint::CanonicalBasisReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::ExportBundleManifest,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/export/export_ready_fixtures/mod.rs"),
            CanonicalHarnessExpansionPoint::ExportFixtureReplayLane,
        ),
        CanonicalFixtureManifestEvidence::new(
            CanonicalGoldenArtifactEvidence::DigestSlotDerivedValue,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/golden_artifacts/digest_slots/mod.rs"),
            CanonicalHarnessExpansionPoint::DigestSlotHostilityLane,
        ),
    ]
}

pub(super) fn public_surface_inventory() -> Vec<CanonicalPublicSurfaceEntry> {
    canonical_public_surface_inventory().to_vec()
}

pub(super) const fn public_surface_evidence_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/canonicalization/grouped_surface/mod.rs")
}

pub(super) const fn public_surface_compile_fail_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/canonicalization/grouped_surface/raw_sequence_cannot_enter_grouped_common_path_comparison.rs")
}
