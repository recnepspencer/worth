mod authority;
mod certification;
mod checklist;
mod inventory;
mod production_test_contract;
mod production_test_handoff;
mod report;
mod vocabulary;

pub use authority::FoundationalDiagnosticProductionReadinessAuthority;
pub use certification::{
    certify_foundational_diagnostic_milestone6_production_test_readiness,
    foundational_diagnostic_milestone6_readiness_report,
    require_foundational_diagnostic_milestone6_production_test_readiness,
    FoundationalDiagnosticProductionTestReadyArtifact,
};
pub use production_test_contract::{
    FoundationalDiagnosticAdoptionShapedFollowthrough,
    FoundationalDiagnosticCanonicalGoldenArtifact,
    FoundationalDiagnosticCanonicalGoldenArtifactEvidence,
    FoundationalDiagnosticHarnessExpansionEvidence, FoundationalDiagnosticHarnessExpansionPoint,
    FoundationalDiagnosticPropertySeed, FoundationalDiagnosticPropertySeedEvidence,
    FoundationalDiagnosticRuntimeAdoptionFailurePressure,
};
pub use report::FoundationalDiagnosticProductionReadinessReport;
pub use vocabulary::{
    FoundationalDiagnosticCertifiedSurface, FoundationalDiagnosticCertifiedSurfaceEvidence,
    FoundationalDiagnosticCompileFailBoundary, FoundationalDiagnosticCompileFailEvidence,
    FoundationalDiagnosticMilestone6PhaseGate, FoundationalDiagnosticPhaseGateEvidence,
    FoundationalDiagnosticProductionReadinessScope, FoundationalDiagnosticResidualDebt,
    FoundationalDiagnosticRuntimeAssumption, FoundationalDiagnosticRuntimeNonAssumption,
    FoundationalDiagnosticSyntheticPressureEvidence,
    FoundationalDiagnosticSyntheticRuntimePressure, FoundationalDiagnosticWORTHProofApi,
    FoundationalDiagnosticWORTHProofApiEvidence, FoundationalDiagnosticWORTHProofForbiddenSurface,
    FoundationalDiagnosticWORTHProofSurface,
};
