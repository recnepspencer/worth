use serde::{Deserialize, Serialize};

/// The cause retained by both commit preparation and derived-index execution.
/// Memory refusals retain the refusing limit and its remaining room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalExecutionDenialCause {
    HistoricalVisibility(crate::visibility::snapshot_states::historical_denial::HistoricalVisibilityDenial),
    ProjectionContract(
        crate::visibility::materialization::read_records::projection::contracts::denial::RelationalProjectionContractDenial,
    ),
    StorageUnavailable(crate::storage::data::storage_unavailable::RelationalStorageUnavailable),
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
    /// Independently checked retained content disagrees with its identity.
    ContentInconsistency,
    /// The exact semantic refusal from Native content readmission.
    Content(crate::content::denial::RelationalContentRejection),
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

    /// The request refusal this cause was mapped from, for an owner that
    /// reports request stops in execution terms. Storage, content and
    /// allocation refusals have none.
    pub fn as_request_denial(self) -> Option<worth_execution::WorkCeilingDenial> {
        use worth_execution::{
            LeaseDenial as Lease, MemoryLimitDenial, MemoryLimitLevel as Level,
            WorkCeilingDenial as Denial,
        };
        let capacity = |requested, admitted, level| {
            Denial::Admission(Lease::MemoryExhausted(MemoryLimitDenial::Capacity {
                requested,
                admitted,
                level,
            }))
        };
        Some(match self {
            Self::WorkerLimitExceedsParent => Denial::Admission(Lease::WorkerLimitExceedsParent),
            Self::MemoryLimitExceedsParent => Denial::Admission(Lease::MemoryLimitExceedsParent),
            Self::WorkLimitExceedsParent => Denial::Admission(Lease::WorkLimitExceedsParent),
            Self::PolicyMemoryExhausted {
                requested,
                admitted,
                ancestor,
            } => capacity(requested, admitted, Level::Policy { ancestor }),
            Self::ProcessMemoryExhausted {
                requested,
                admitted,
            } => capacity(requested, admitted, Level::Process),
            Self::DeclaredMemoryExhausted {
                requested,
                admitted,
            } => capacity(requested, admitted, Level::Declared),
            Self::ChargedBytesOverflow => Denial::Admission(Lease::ChargedBytesOverflow),
            Self::UnrelatedNestedLease => Denial::Admission(Lease::UnrelatedNestedLease),
            Self::NoActiveExecutionScope => Denial::Admission(Lease::NoActiveExecutionScope),
            Self::EquivalenceContractUnavailable => {
                Denial::Admission(Lease::EquivalenceContractUnavailable)
            }
            Self::WorkerFailed => Denial::Panicked,
            stop => Denial::Stopped(stop.as_kernel_stop()?),
        })
    }
}
