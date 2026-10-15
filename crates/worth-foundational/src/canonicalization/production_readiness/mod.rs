mod authority;
mod certification;
mod inventory;
mod report;
mod vocabulary;

pub use authority::CanonicalProductionReadinessAuthority;
pub use certification::{
    canonical_milestone2_production_readiness_report,
    certify_canonical_milestone2_production_readiness, require_canonical_production_test_readiness,
    CanonicalProductionTestReadyArtifact,
};
pub use report::CanonicalProductionReadinessReport;
pub use vocabulary::{
    CanonicalCertifiedSurface, CanonicalCertifiedSurfaceEvidence, CanonicalCompileFailBoundary,
    CanonicalCostCounterEvidence, CanonicalFixtureManifestEvidence,
    CanonicalGoldenArtifactEvidence, CanonicalHarnessExpansionPoint, CanonicalMilestone2PhaseGate,
    CanonicalPhaseGateEvidence, CanonicalProductionReadinessScope, CanonicalPropertySeed,
    CanonicalPropertySeedEvidence, CanonicalResidualDebt, CanonicalRuntimeAssumption,
    CanonicalRuntimeNonAssumption, CanonicalSyntheticRuntimePressure,
};
