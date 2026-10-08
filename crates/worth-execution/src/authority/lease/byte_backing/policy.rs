use super::super::{ExecutionLeaseStatus, ExecutionResourceLease};
use super::{ExecutionByteAllocationDenial, ExecutionByteAllocationDenialKind};

/// The caller chooses before allocation. There is no fallback or default.
/// System allocation remains uncharged; Execution admits only payload backing,
/// not allocator, Arc, or ledger metadata and not graph/model authority.
#[derive(Clone, Copy, Debug)]
pub enum ExecutionByteAllocationPolicy<'scope, 'authority> {
    SystemAllocation,
    Execution(&'scope ExecutionResourceLease<'authority>),
}

impl ExecutionByteAllocationPolicy<'_, '_> {
    /// Check before a layout quote exists; a stop therefore carries no quote.
    pub fn check_live(self) -> Result<(), ExecutionByteAllocationDenial> {
        match self {
            Self::SystemAllocation => Ok(()),
            Self::Execution(lease) => check_status(&lease.status(), None),
        }
    }
}

pub(super) fn check_status(
    status: &ExecutionLeaseStatus,
    quote: Option<u64>,
) -> Result<(), ExecutionByteAllocationDenial> {
    let kind = if status.is_cancelled() {
        Some(ExecutionByteAllocationDenialKind::Cancelled)
    } else if status.deadline_elapsed() {
        Some(ExecutionByteAllocationDenialKind::DeadlineElapsed)
    } else {
        None
    };
    kind.map_or(Ok(()), |kind| {
        Err(ExecutionByteAllocationDenial::new(kind, quote))
    })
}
