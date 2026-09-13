mod denial;
mod dirty_generation;
mod frame_access;
mod integrity_validation;
mod lease;
mod limits;
mod observation;
mod operation_allocation;
mod pool;
mod pool_ownership;
mod pressure;
mod speculation;
mod work_kind;
mod writeback_range_posture;

pub use denial::PhysicalResidencyDenial;
pub use dirty_generation::{
    PhysicalDirtyFrameBasis, PhysicalDirtyGeneration, PhysicalDirtyGenerationCaptureCompletion,
    PhysicalDirtyGenerationCaptureSession, PhysicalDirtyGenerationCaptureStep,
    PhysicalDirtyGenerationSlice,
};
pub use frame_access::{
    PhysicalBoundedFrameAccess, PhysicalBoundedFrameFaultOwner, PhysicalBoundedFrameFaultWaiter,
    PhysicalFrameAccess, PhysicalFrameFaultError, PhysicalFrameFaultOwner,
    PhysicalFrameFaultWaiter, PhysicalFrameLoadTerminal, PhysicalFrameLoadTerminalKind,
    PhysicalFrameLoadingIdentity,
};
pub use integrity_validation::{
    CleanFrameIntegrityValidationDenial, PhysicalResidentFrameGeneration,
};
pub use lease::{
    DirtyPhysicalFrame, PhysicalCandidateBatchAdmission, PhysicalCandidateBatchReservation,
    PhysicalCandidateFrameReservation, PhysicalDirtyReplacementError,
    PhysicalDirtyReplacementReservation, PhysicalFrameLease, PhysicalWritebackClaim,
};
pub use limits::{
    PhysicalOperationAllocationScope, PhysicalResidencyDimension, PhysicalResidencyLimits,
    PhysicalResidencyLimitsAdmissionDenial,
};
pub(in crate::physical_residency) use observation::{
    PhysicalFrameRemoval, PhysicalResidencyAccounting, PhysicalResidencyActualAllocationUnits,
    PhysicalResidencyAllocationActualization, PhysicalResidencyAllocationEventRecorder,
    PhysicalResidencyRequestedAllocationUnits,
};
pub use observation::{
    PhysicalResidencyAllocationBoundaryEvent, PhysicalResidencyAllocationBoundaryKind,
    PhysicalResidencyAllocationEventCounters, PhysicalResidencyAllocationEventObserver,
    PhysicalResidencyAllocationEventSnapshot, PhysicalResidencyAllocationOperation,
    PhysicalResidencyAllocationTrace, PhysicalResidencyCounters, PhysicalResidencyShutdown,
};
pub use operation_allocation::{
    ForegroundReadAllocationGrant, ForegroundWriteAllocationGrant, MaintenanceAllocationGrant,
    OperationAllocationGrant, OperationAllocationObservation,
};
pub use pool::{
    PhysicalBoundedFrameKey, PhysicalCandidateFrameKey, PhysicalFrameKey,
    PhysicalResidencyIncarnation, PhysicalResidencyPool,
};
pub use pool_ownership::{
    CandidateFrameCleanAuthority, FrameWritebackCleanAuthority, PhysicalResidencyPoolOwner,
};
pub(in crate::physical_residency) use pressure::PhysicalResidencyPressureDemand;
pub use pressure::{PhysicalResidencyLimitsBuilder, PhysicalResidencyPressureDenial};
pub use speculation::{
    BufferPoolQueueDeclarationContext, BufferPoolQueueGroupingScope,
    BufferPoolQueueWriteDurability, BufferPoolReadQueueExecutionDeclaration,
    BufferPoolReadQueueExecutionKind, BufferPoolWritebackQueueExecutionDeclaration,
    PrefetchResidencyGrant, ReadAheadFrameGrant, ReadAheadResidencyGrant,
    WriteBehindResidencyGrant,
};
pub use work_kind::PhysicalSpeculativeWorkKind;
pub use writeback_range_posture::PhysicalWritebackRangePosture;

#[cfg(test)]
mod tests;
