mod authority;
mod certification;
mod checklist;
mod closure_inventory;
mod inventory;
mod phase_inventory;
mod report;
mod vocabulary;

pub use authority::FoundationalPerformanceProductionReadinessAuthority;
pub use certification::{
    certify_foundational_performance_milestone8_production_test_readiness,
    foundational_performance_milestone8_readiness_report,
    require_foundational_performance_milestone8_production_test_readiness,
    FoundationalPerformanceProductionTestReadyArtifact,
};
pub use report::FoundationalPerformanceProductionReadinessReport;
pub use vocabulary::{
    FoundationalPerformanceCertifiedSurface, FoundationalPerformanceCertifiedSurfaceEvidence,
    FoundationalPerformanceCompileFailBoundary, FoundationalPerformanceHarnessExpansionPoint,
    FoundationalPerformanceMilestone8PhaseGate, FoundationalPerformancePhaseGateEvidence,
    FoundationalPerformanceProductionReadinessScope,
    FoundationalPerformancePublicSurfaceDocumentationCoverage, FoundationalPerformanceResidualDebt,
    FoundationalPerformanceRuntimeAdoptionPressure,
    FoundationalPerformanceRuntimeAdoptionPressureEvidence,
    FoundationalPerformanceRuntimeAssumption, FoundationalPerformanceRuntimeNonAssumption,
    FoundationalPerformanceSyntheticRuntimePressure, FoundationalPerformanceWORTHProofApi,
    FoundationalPerformanceWORTHProofForbiddenSurface, FoundationalPerformanceWORTHProofSurface,
};
