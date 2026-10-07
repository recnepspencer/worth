use super::PhysicalFrameLoadTerminal;
use super::PhysicalResidencyPressureDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalResidencyDenial {
    WrongStore,
    FrameLargerThanResidentBudget,
    MetadataBudgetExceeded,
    Pressure(PhysicalResidencyPressureDenial),
    WriteBackExceedsDirtyPosture,
    WriteBackFrameNotDirty,
    WriteBackFrameAlreadyClaimed,
    WriteBackReceiptMismatch,
    CandidateCleanAuthorityMismatch,
    WritebackCleanAuthorityMismatch,
    AllocationFailed,
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
    AllocationGrantMismatch,
    AllocationGrantResizeBelowActiveUse {
        requested: u64,
        active: u64,
    },
    DirtyGenerationCaptureSessionMismatch,
    DirtyGenerationExhausted,
    DirtyGenerationCaptureBudgetExceeded {
        required: u64,
        admitted: u64,
    },
    SpeculativeAllocationMismatch {
        granted: u64,
        required: u64,
    },
    EmptySpeculativeRead,
    DuplicateSpeculativeFrame,
    CandidatePublicationActive,
    PoolClosed,
    BoundedLoadLimitConflict {
        active_limit: u32,
        requested_limit: u32,
    },
    EmptyCandidateBatch,
    CandidateCardinalityMismatch {
        declared: usize,
        provided: usize,
    },
    DuplicateCandidateIdentity,
    CandidateCoverageConflict,
    CandidateSequenceConflict,
    CompleteArtifactRequiresOffsetZero,
    ArtifactIdentityOccupied,
    FrameIdentityOccupied,
    FrameLengthMismatch,
    FrameNotDirty,
    FrameAlreadyResident,
    FrameLoadTerminated(PhysicalFrameLoadTerminal),
    FrameNotResident,
    FramePinned,
    FrameDirty,
    IdentityAlreadyCurrent,
}
