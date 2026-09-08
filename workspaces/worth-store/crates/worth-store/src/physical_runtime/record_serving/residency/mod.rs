pub(in crate::physical_runtime) mod artifact_tree;
pub(super) mod candidate_frame_publishers;
pub(super) mod candidate_frame_residency;
mod capability;
#[cfg(feature = "certification-test-authority")]
mod certification;
mod dirty;
mod failure;
pub(super) mod frame_load_failure;
pub(super) mod frame_loading;
pub(super) mod frame_ports;
pub(in crate::physical_runtime) use frame_ports::RecordFramePorts;
mod frame_read_failure;
pub(super) mod frame_work_trace;
pub(super) mod initialization_artifacts;
mod pressure_evidence;
pub(super) mod publication_artifacts;
pub(super) mod record_frame_reader;
mod residency_observation;
pub(in crate::physical_runtime) mod scheduled_writeback;
mod scoped_allocation;
pub(super) mod serving_artifacts;
mod speculation;

pub(super) use capability::PhysicalResidencyWorkPort;
#[cfg(feature = "certification-test-authority")]
pub use certification::{
    AdmittedDirtyFrame, AdmittedPhysicalWriteback, CertificationFrameFaultCause,
    CertificationFrameReadFailure, CertificationFrameWorkFailure, CertificationResidentFrame,
    CertificationScopeAdmissionFailure, CertificationScopePressure, CertificationScopedAllocation,
    PhysicalDirtyTransitionFailure, PhysicalResidencyCertification, PhysicalWritebackExecution,
    PhysicalWritebackInspectionRequired, PhysicalWritebackSettlement,
    PhysicalWritebackTransitionFailure, PreparedPhysicalWriteback, ReadyPhysicalWriteback,
    RetryablePhysicalWriteback,
};
pub(super) use dirty::FrameWritebackPort;
pub use dirty::{
    PhysicalRecordWritebackFailureCause, PhysicalRecordWritebackFailureEvidence,
    PhysicalWritebackFailureCause,
};
pub use failure::{
    PhysicalRecordResidencyFailure, PhysicalRecordResidencyFailureKind,
    PhysicalRecordResidencyFailureReason,
};
pub use frame_read_failure::{
    PhysicalFrameFaultCause, PhysicalFrameReadFailure, PhysicalFrameWorkFailure,
};
pub use pressure_evidence::{
    PhysicalRecordPressureBasis, PhysicalRecordPressureEvidence, PhysicalResidencyRetryPosture,
};
#[cfg(feature = "certification-test-authority")]
pub use residency_observation::{
    PhysicalResidencyAllocationBoundaryEvent, PhysicalResidencyAllocationBoundaryKind,
    PhysicalResidencyAllocationTrace,
};
pub use residency_observation::{
    PhysicalResidencyAllocationEventSnapshot, PhysicalResidencyAllocationSnapshot,
    PhysicalResidencyCounterSnapshot, PhysicalResidencyObservation,
    PhysicalWritebackCounterSnapshot,
};
pub use scoped_allocation::{
    BlobPhysicalAllocation, MaintenancePhysicalAllocation, PhysicalScopedAllocationAdmission,
    PhysicalScopedAllocationFailure, RecoveryPhysicalAllocation, ScrubPhysicalAllocation,
    VerificationPhysicalAllocation,
};
pub use speculation::{
    PhysicalPrefetchIntent, PhysicalPrefetchOutcome, PhysicalReadAheadBatch,
    PhysicalReadAheadFrameOutcome, PhysicalReadAheadIntent, PhysicalReadAheadIntentDenial,
    PhysicalReadAheadOutcome, PhysicalSpeculativeReadDrop, PhysicalSpeculativeReadFailure,
};
