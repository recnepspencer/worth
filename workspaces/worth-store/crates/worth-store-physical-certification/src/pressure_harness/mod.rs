mod execution;
mod replay;
mod scenario;
mod vocabulary;

#[cfg(any(test, feature = "certification-test-support"))]
pub(crate) mod fixtures;

pub use execution::IoPressureExecutionCounters;
pub use replay::IoPressureHarnessEvidenceDenial;
pub use scenario::{
    IoPressureBackendSafetyQualificationDenial, IoPressureEvidenceMaturity, IoPressureFaultKind,
    IoPressureHarnessEvidence, IoPressureHarnessScenario, IoPressureHarnessSecureIoPosture,
    IoPressureOracleObservation, PhysicalFaultEvidenceClass, RealBackendSafetyQualification,
};
pub use vocabulary::{all_io_pressure_fault_evidence_classes, all_io_pressure_fault_kinds};

#[cfg(feature = "certification-test-support")]
pub use fixtures::replay_bundle_for as test_replay_bundle_for;

#[cfg(test)]
mod tests;
