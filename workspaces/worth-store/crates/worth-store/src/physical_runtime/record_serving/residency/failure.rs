use worth_store_buffer_pool::{
    PhysicalFrameLoadTerminal, PhysicalResidencyDenial, PhysicalResidencyPressureDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecordResidencyFailure {
    kind: PhysicalRecordResidencyFailureKind,
    denial: PhysicalResidencyDenial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecordResidencyFailureKind {
    StoreIdentityMismatch,
    ConfigurationIncompatible,
    PhysicalPressure,
    WritebackStateConflict,
    SettlementAuthorityMismatch,
    AllocationUnavailable,
    AllocationAuthorityMismatch,
    AllocationUseConflict,
    CaptureSessionAuthorityMismatch,
    DirtyGenerationExhausted,
    SpeculativeContractConflict,
    PublicationConflict,
    LifecycleClosed,
    FrameLoadTerminated,
    FrameLoadConflict,
    CandidateContractConflict,
    FrameIdentityConflict,
    FrameStateConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecordResidencyFailureReason {
    WrongStore,
    FrameLargerThanResidentBudget,
    MetadataBudgetExceeded,
    PhysicalPressure,
    WritebackExceedsDirtyPosture,
    WritebackFrameNotDirty,
    WritebackFrameAlreadyClaimed,
    WritebackReceiptMismatch,
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
    FrameLoadTerminated,
    FrameNotResident,
    FramePinned,
    FrameDirty,
    IdentityAlreadyCurrent,
}

impl PhysicalRecordResidencyFailure {
    pub const fn kind(self) -> PhysicalRecordResidencyFailureKind {
        self.kind
    }

    pub const fn reason(self) -> PhysicalRecordResidencyFailureReason {
        match self.denial {
            PhysicalResidencyDenial::WrongStore => PhysicalRecordResidencyFailureReason::WrongStore,
            PhysicalResidencyDenial::FrameLargerThanResidentBudget => {
                PhysicalRecordResidencyFailureReason::FrameLargerThanResidentBudget
            }
            PhysicalResidencyDenial::MetadataBudgetExceeded => {
                PhysicalRecordResidencyFailureReason::MetadataBudgetExceeded
            }
            PhysicalResidencyDenial::Pressure(_) => {
                PhysicalRecordResidencyFailureReason::PhysicalPressure
            }
            PhysicalResidencyDenial::WriteBackExceedsDirtyPosture => {
                PhysicalRecordResidencyFailureReason::WritebackExceedsDirtyPosture
            }
            PhysicalResidencyDenial::WriteBackFrameNotDirty => {
                PhysicalRecordResidencyFailureReason::WritebackFrameNotDirty
            }
            PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed => {
                PhysicalRecordResidencyFailureReason::WritebackFrameAlreadyClaimed
            }
            PhysicalResidencyDenial::WriteBackReceiptMismatch => {
                PhysicalRecordResidencyFailureReason::WritebackReceiptMismatch
            }
            PhysicalResidencyDenial::CandidateCleanAuthorityMismatch => {
                PhysicalRecordResidencyFailureReason::CandidateCleanAuthorityMismatch
            }
            PhysicalResidencyDenial::WritebackCleanAuthorityMismatch => {
                PhysicalRecordResidencyFailureReason::WritebackCleanAuthorityMismatch
            }
            PhysicalResidencyDenial::AllocationFailed => {
                PhysicalRecordResidencyFailureReason::AllocationFailed
            }
            PhysicalResidencyDenial::AllocatorExceededReservation { requested, actual } => {
                PhysicalRecordResidencyFailureReason::AllocatorExceededReservation {
                    requested,
                    actual,
                }
            }
            PhysicalResidencyDenial::AllocationGrantMismatch => {
                PhysicalRecordResidencyFailureReason::AllocationGrantMismatch
            }
            PhysicalResidencyDenial::AllocationGrantResizeBelowActiveUse { requested, active } => {
                PhysicalRecordResidencyFailureReason::AllocationGrantResizeBelowActiveUse {
                    requested,
                    active,
                }
            }
            PhysicalResidencyDenial::DirtyGenerationCaptureSessionMismatch => {
                PhysicalRecordResidencyFailureReason::DirtyGenerationCaptureSessionMismatch
            }
            PhysicalResidencyDenial::DirtyGenerationExhausted => {
                PhysicalRecordResidencyFailureReason::DirtyGenerationExhausted
            }
            PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded {
                required,
                admitted,
            } => PhysicalRecordResidencyFailureReason::DirtyGenerationCaptureBudgetExceeded {
                required,
                admitted,
            },
            PhysicalResidencyDenial::SpeculativeAllocationMismatch { granted, required } => {
                PhysicalRecordResidencyFailureReason::SpeculativeAllocationMismatch {
                    granted,
                    required,
                }
            }
            PhysicalResidencyDenial::EmptySpeculativeRead => {
                PhysicalRecordResidencyFailureReason::EmptySpeculativeRead
            }
            PhysicalResidencyDenial::DuplicateSpeculativeFrame => {
                PhysicalRecordResidencyFailureReason::DuplicateSpeculativeFrame
            }
            PhysicalResidencyDenial::CandidatePublicationActive => {
                PhysicalRecordResidencyFailureReason::CandidatePublicationActive
            }
            PhysicalResidencyDenial::PoolClosed => PhysicalRecordResidencyFailureReason::PoolClosed,
            PhysicalResidencyDenial::BoundedLoadLimitConflict {
                active_limit,
                requested_limit,
            } => PhysicalRecordResidencyFailureReason::BoundedLoadLimitConflict {
                active_limit,
                requested_limit,
            },
            PhysicalResidencyDenial::EmptyCandidateBatch => {
                PhysicalRecordResidencyFailureReason::EmptyCandidateBatch
            }
            PhysicalResidencyDenial::CandidateCardinalityMismatch { declared, provided } => {
                PhysicalRecordResidencyFailureReason::CandidateCardinalityMismatch {
                    declared,
                    provided,
                }
            }
            PhysicalResidencyDenial::DuplicateCandidateIdentity => {
                PhysicalRecordResidencyFailureReason::DuplicateCandidateIdentity
            }
            PhysicalResidencyDenial::CandidateCoverageConflict => {
                PhysicalRecordResidencyFailureReason::CandidateCoverageConflict
            }
            PhysicalResidencyDenial::CandidateSequenceConflict => {
                PhysicalRecordResidencyFailureReason::CandidateSequenceConflict
            }
            PhysicalResidencyDenial::CompleteArtifactRequiresOffsetZero => {
                PhysicalRecordResidencyFailureReason::CompleteArtifactRequiresOffsetZero
            }
            PhysicalResidencyDenial::ArtifactIdentityOccupied => {
                PhysicalRecordResidencyFailureReason::ArtifactIdentityOccupied
            }
            PhysicalResidencyDenial::FrameIdentityOccupied => {
                PhysicalRecordResidencyFailureReason::FrameIdentityOccupied
            }
            PhysicalResidencyDenial::FrameLengthMismatch => {
                PhysicalRecordResidencyFailureReason::FrameLengthMismatch
            }
            PhysicalResidencyDenial::FrameNotDirty => {
                PhysicalRecordResidencyFailureReason::FrameNotDirty
            }
            PhysicalResidencyDenial::FrameAlreadyResident => {
                PhysicalRecordResidencyFailureReason::FrameAlreadyResident
            }
            PhysicalResidencyDenial::FrameLoadTerminated(_) => {
                PhysicalRecordResidencyFailureReason::FrameLoadTerminated
            }
            PhysicalResidencyDenial::FrameNotResident => {
                PhysicalRecordResidencyFailureReason::FrameNotResident
            }
            PhysicalResidencyDenial::FramePinned => {
                PhysicalRecordResidencyFailureReason::FramePinned
            }
            PhysicalResidencyDenial::FrameDirty => PhysicalRecordResidencyFailureReason::FrameDirty,
            PhysicalResidencyDenial::IdentityAlreadyCurrent => {
                PhysicalRecordResidencyFailureReason::IdentityAlreadyCurrent
            }
        }
    }

    pub const fn frame_load_terminal(self) -> Option<PhysicalFrameLoadTerminal> {
        match self.denial {
            PhysicalResidencyDenial::FrameLoadTerminated(terminal) => Some(terminal),
            _ => None,
        }
    }

    pub(in crate::physical_runtime::record_serving) const fn pressure_denial(
        self,
    ) -> Option<PhysicalResidencyPressureDenial> {
        match self.denial {
            PhysicalResidencyDenial::Pressure(pressure) => Some(pressure),
            _ => None,
        }
    }
}

impl From<PhysicalResidencyDenial> for PhysicalRecordResidencyFailure {
    fn from(denial: PhysicalResidencyDenial) -> Self {
        let kind = match denial {
            PhysicalResidencyDenial::WrongStore => {
                PhysicalRecordResidencyFailureKind::StoreIdentityMismatch
            }
            PhysicalResidencyDenial::FrameLargerThanResidentBudget
            | PhysicalResidencyDenial::MetadataBudgetExceeded => {
                PhysicalRecordResidencyFailureKind::ConfigurationIncompatible
            }
            PhysicalResidencyDenial::Pressure(_) => {
                PhysicalRecordResidencyFailureKind::PhysicalPressure
            }
            PhysicalResidencyDenial::WriteBackExceedsDirtyPosture
            | PhysicalResidencyDenial::WriteBackFrameNotDirty
            | PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed
            | PhysicalResidencyDenial::WriteBackReceiptMismatch => {
                PhysicalRecordResidencyFailureKind::WritebackStateConflict
            }
            PhysicalResidencyDenial::CandidateCleanAuthorityMismatch
            | PhysicalResidencyDenial::WritebackCleanAuthorityMismatch => {
                PhysicalRecordResidencyFailureKind::SettlementAuthorityMismatch
            }
            PhysicalResidencyDenial::AllocationFailed
            | PhysicalResidencyDenial::AllocatorExceededReservation { .. }
            | PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded { .. } => {
                PhysicalRecordResidencyFailureKind::AllocationUnavailable
            }
            PhysicalResidencyDenial::AllocationGrantMismatch => {
                PhysicalRecordResidencyFailureKind::AllocationAuthorityMismatch
            }
            PhysicalResidencyDenial::AllocationGrantResizeBelowActiveUse { .. } => {
                PhysicalRecordResidencyFailureKind::AllocationUseConflict
            }
            PhysicalResidencyDenial::DirtyGenerationCaptureSessionMismatch => {
                PhysicalRecordResidencyFailureKind::CaptureSessionAuthorityMismatch
            }
            PhysicalResidencyDenial::DirtyGenerationExhausted => {
                PhysicalRecordResidencyFailureKind::DirtyGenerationExhausted
            }
            PhysicalResidencyDenial::SpeculativeAllocationMismatch { .. }
            | PhysicalResidencyDenial::EmptySpeculativeRead
            | PhysicalResidencyDenial::DuplicateSpeculativeFrame => {
                PhysicalRecordResidencyFailureKind::SpeculativeContractConflict
            }
            PhysicalResidencyDenial::CandidatePublicationActive => {
                PhysicalRecordResidencyFailureKind::PublicationConflict
            }
            PhysicalResidencyDenial::PoolClosed => {
                PhysicalRecordResidencyFailureKind::LifecycleClosed
            }
            PhysicalResidencyDenial::FrameLoadTerminated(_) => {
                PhysicalRecordResidencyFailureKind::FrameLoadTerminated
            }
            PhysicalResidencyDenial::BoundedLoadLimitConflict { .. } => {
                PhysicalRecordResidencyFailureKind::FrameLoadConflict
            }
            PhysicalResidencyDenial::EmptyCandidateBatch
            | PhysicalResidencyDenial::CandidateCardinalityMismatch { .. }
            | PhysicalResidencyDenial::DuplicateCandidateIdentity
            | PhysicalResidencyDenial::CandidateCoverageConflict
            | PhysicalResidencyDenial::CandidateSequenceConflict => {
                PhysicalRecordResidencyFailureKind::CandidateContractConflict
            }
            PhysicalResidencyDenial::CompleteArtifactRequiresOffsetZero
            | PhysicalResidencyDenial::ArtifactIdentityOccupied
            | PhysicalResidencyDenial::FrameIdentityOccupied
            | PhysicalResidencyDenial::IdentityAlreadyCurrent => {
                PhysicalRecordResidencyFailureKind::FrameIdentityConflict
            }
            PhysicalResidencyDenial::FrameLengthMismatch
            | PhysicalResidencyDenial::FrameNotDirty
            | PhysicalResidencyDenial::FrameAlreadyResident
            | PhysicalResidencyDenial::FrameNotResident
            | PhysicalResidencyDenial::FramePinned
            | PhysicalResidencyDenial::FrameDirty => {
                PhysicalRecordResidencyFailureKind::FrameStateConflict
            }
        };
        Self { kind, denial }
    }
}

#[cfg(test)]
#[path = "failure/tests.rs"]
mod tests;
