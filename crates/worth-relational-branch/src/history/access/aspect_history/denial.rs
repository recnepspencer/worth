//! Finite record-history cursor and selection refusals.
#[derive(Debug)]
pub enum RelationalRecordHistoryDenial {
    ForeignOwner,
    Storage(crate::execution::denial_cause::RelationalExecutionDenialCause),
    SubjectUnavailable,
    EmptySubjectSelection,
    EmptyAspectSelection,
    CommitUnavailable,
    NotExactSelectedCommit,
    InvalidContinuation,
    Retention(crate::branch::basis_denial::RelationalBranchBasisDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Request(worth_execution::WorkCeilingDenial),
    Interrupted(worth_execution::MapKernelStop),
    PageStorageTooSmall,
    SizeOverflow,
    SelectionLimitExceeded,
}

impl From<worth_execution::ExecutionAllocationDenial> for RelationalRecordHistoryDenial {
    fn from(cause: worth_execution::ExecutionAllocationDenial) -> Self {
        Self::Allocation(cause)
    }
}

impl From<worth_execution::MapKernelStop> for RelationalRecordHistoryDenial {
    fn from(cause: worth_execution::MapKernelStop) -> Self {
        Self::Interrupted(cause)
    }
}

impl From<worth_execution::ExecutionRequestArrayDenial> for RelationalRecordHistoryDenial {
    fn from(cause: worth_execution::ExecutionRequestArrayDenial) -> Self {
        match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => Self::Request(cause),
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
                Self::Allocation(cause)
            }
        }
    }
}
