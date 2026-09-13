#![doc = include_str!("physical_residency/compile_fail_proofs.md")]
#![forbid(unsafe_code)]

mod physical_residency;
pub use physical_residency::{
    BufferPoolQueueDeclarationContext, BufferPoolQueueGroupingScope,
    BufferPoolQueueWriteDurability, BufferPoolReadQueueExecutionDeclaration,
    BufferPoolReadQueueExecutionKind, BufferPoolWritebackQueueExecutionDeclaration,
    CandidateFrameCleanAuthority, CleanFrameIntegrityValidationDenial, DirtyPhysicalFrame,
    ForegroundReadAllocationGrant, ForegroundWriteAllocationGrant, FrameWritebackCleanAuthority,
    MaintenanceAllocationGrant, OperationAllocationGrant, OperationAllocationObservation,
    PhysicalBoundedFrameAccess, PhysicalBoundedFrameFaultOwner, PhysicalBoundedFrameFaultWaiter,
    PhysicalBoundedFrameKey, PhysicalCandidateBatchAdmission, PhysicalCandidateBatchReservation,
    PhysicalCandidateFrameKey, PhysicalCandidateFrameReservation, PhysicalDirtyFrameBasis,
    PhysicalDirtyGeneration, PhysicalDirtyGenerationCaptureCompletion,
    PhysicalDirtyGenerationCaptureSession, PhysicalDirtyGenerationCaptureStep,
    PhysicalDirtyGenerationSlice, PhysicalDirtyReplacementError,
    PhysicalDirtyReplacementReservation, PhysicalFrameAccess, PhysicalFrameFaultError,
    PhysicalFrameFaultOwner, PhysicalFrameFaultWaiter, PhysicalFrameKey, PhysicalFrameLease,
    PhysicalFrameLoadTerminal, PhysicalFrameLoadTerminalKind, PhysicalFrameLoadingIdentity,
    PhysicalOperationAllocationScope, PhysicalResidencyAllocationBoundaryEvent,
    PhysicalResidencyAllocationBoundaryKind, PhysicalResidencyAllocationEventCounters,
    PhysicalResidencyAllocationEventObserver, PhysicalResidencyAllocationEventSnapshot,
    PhysicalResidencyAllocationOperation, PhysicalResidencyAllocationTrace,
    PhysicalResidencyCounters, PhysicalResidencyDenial, PhysicalResidencyDimension,
    PhysicalResidencyIncarnation, PhysicalResidencyLimits, PhysicalResidencyLimitsAdmissionDenial,
    PhysicalResidencyLimitsBuilder, PhysicalResidencyPool, PhysicalResidencyPoolOwner,
    PhysicalResidencyPressureDenial, PhysicalResidencyShutdown, PhysicalResidentFrameGeneration,
    PhysicalSpeculativeWorkKind, PhysicalWritebackClaim, PhysicalWritebackRangePosture,
    PrefetchResidencyGrant, ReadAheadFrameGrant, ReadAheadResidencyGrant,
    WriteBehindResidencyGrant,
};
