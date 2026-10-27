use serde::{Deserialize, Serialize};

/// The cause retained by both commit preparation and derived-index execution.
/// Memory refusals retain the refusing limit and its remaining room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalExecutionDenialCause {
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
    WorkerLimitExceedsParent,
    MemoryLimitExceedsParent,
    WorkLimitExceedsParent,
    PolicyMemoryExhausted {
        requested: u64,
        admitted: u64,
        ancestor: u32,
    },
    ProcessMemoryExhausted {
        requested: u64,
        admitted: u64,
    },
    DeclaredMemoryExhausted {
        requested: u64,
        admitted: u64,
    },
    ChargedBytesOverflow,
    UnrelatedNestedLease,
    NoActiveExecutionScope,
    EquivalenceContractUnavailable,
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    AccountingUnavailable,
    WorkExhausted,
    NestedStopped,
    ResultCapacityExceeded,
    WorkerFailed,
    AllocationFailed,
    ScratchCapacityExceeded,
    UncheckedCustomKernel,
    ExpectedIdentitiesNotCanonical,
    MemoryOverflow,
}

impl RelationalExecutionDenialCause {
    /// Project only execution-kernel stops; storage and admission refusals
    /// retain their native payload rather than becoming synthetic map stops.
    pub fn as_kernel_stop(self) -> Option<worth_execution::MapKernelStop> {
        use worth_execution::MapKernelStop as Stop;
        match self {
            Self::Cancelled => Some(Stop::Cancelled),
            Self::DeadlineElapsed => Some(Stop::DeadlineElapsed),
            Self::WorkCounterOverflow => Some(Stop::WorkCounterOverflow),
            Self::AccountingUnavailable => Some(Stop::AccountingUnavailable),
            Self::WorkExhausted => Some(Stop::WorkCeiling),
            Self::NestedStopped => Some(Stop::NestedStopped),
            _ => None,
        }
    }
}
