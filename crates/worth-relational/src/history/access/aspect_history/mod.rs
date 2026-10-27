//! Native finite change pages. Product ancestry remains the composition owner's job.

mod cursor;
mod page;
mod selection;

pub use cursor::RelationalCanonicalRecordHistoryCursor;
pub use page::{
    RelationalPreparedRecordHistoryPage, RelationalRecordHistoryChange,
    RelationalRecordHistoryPage, RelationalRecordHistoryPageLimits,
};
pub use selection::RelationalRecordHistorySubject;

#[derive(Debug)]
pub enum RelationalRecordHistoryDenial {
    ForeignOwner,
    SubjectUnavailable,
    EmptySubjectSelection,
    EmptyAspectSelection,
    CommitUnavailable,
    NotExactSelectedCommit,
    InvalidContinuation,
    Retention(crate::branch::RelationalBranchBasisDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Request(worth_execution::WorkCeilingDenial),
    Interrupted(worth_execution::MapKernelStop),
    PageStorageTooSmall,
    SizeOverflow,
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
