use serde::{Deserialize, Serialize};

/// The cause retained by both commit preparation and derived-index execution.
/// Memory refusals retain the refusing limit and its remaining room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalExecutionDenialCause {
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
    EquivalenceContractUnavailable,
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkExhausted,
    NestedStopped,
    ResultCapacityExceeded,
    WorkerFailed,
    ScratchCapacityExceeded,
    UncheckedCustomKernel,
    ExpectedIdentitiesNotCanonical,
    MemoryOverflow,
}
