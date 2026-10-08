use super::super::{ExecutionLeaseStatus, ExecutionResourceLease};
use super::{ExecutionAllocationDenial, ExecutionAllocationDenialKind};

/// The caller chooses before allocation. There is no fallback or default.
/// System allocation remains uncharged; Execution admits only payload backing,
/// not nested element heaps, allocator/Arc/ledger metadata, or graph/model authority.
#[derive(Clone, Copy, Debug)]
pub enum ExecutionAllocationPolicy<'scope, 'authority> {
    SystemAllocation,
    Execution(&'scope ExecutionResourceLease<'authority>),
}

impl ExecutionAllocationPolicy<'_, '_> {
    /// Check before a layout quote exists; a stop therefore carries no quote.
    pub fn check_live(self) -> Result<(), ExecutionAllocationDenial> {
        match self {
            Self::SystemAllocation => Ok(()),
            Self::Execution(lease) => check_status(&lease.status(), None),
        }
    }
}

pub(super) fn check_status(
    status: &ExecutionLeaseStatus,
    quote: Option<u64>,
) -> Result<(), ExecutionAllocationDenial> {
    let kind = if status.is_cancelled() {
        Some(ExecutionAllocationDenialKind::Cancelled)
    } else if status.deadline_elapsed() {
        Some(ExecutionAllocationDenialKind::DeadlineElapsed)
    } else {
        None
    };
    kind.map_or(Ok(()), |kind| {
        Err(ExecutionAllocationDenial::new(kind, quote))
    })
}
