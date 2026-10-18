use super::super::{ExecutionLeaseStatus, ExecutionResourceLease};
use super::{ExecutionAllocationDenial, ExecutionAllocationDenialKind};
use crate::{ExecutionMemoryReservation, LeaseDenial};

/// The caller chooses before allocation. There is no fallback or default.
/// System allocation remains uncharged. Arrays admit inline payload backing;
/// newly owned nested payload needs its own explicit reservation. Neither
/// reservation nor allocation grants graph/model authority.
#[derive(Clone, Copy, Debug)]
pub enum ExecutionAllocationPolicy<'scope, 'authority> {
    SystemAllocation,
    Execution(&'scope ExecutionResourceLease<'authority>),
}

impl ExecutionAllocationPolicy<'_, '_> {
    /// Reserve declared owned payload before copying or allocating it. This
    /// performs no allocation and never adopts or funds an existing raw copy.
    /// The owner must retain the reservation until that payload's final drop.
    pub fn reserve_memory(
        self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, ExecutionAllocationDenial> {
        let Self::Execution(lease) = self else {
            return Ok(None);
        };
        let status = lease.status();
        check_status(&status, Some(bytes))?;
        let reservation = lease.reserve_memory(bytes).map_err(|cause| {
            ExecutionAllocationDenial::new(
                ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(cause)),
                Some(bytes),
            )
        });
        // Preserve interruption precedence, also when memory was exhausted.
        check_status(&status, Some(bytes))?;
        Ok(Some(reservation?))
    }

    /// Check before a layout quote exists; a stop therefore carries no quote.
    pub fn check_live(self) -> Result<(), ExecutionAllocationDenial> {
        match self {
            Self::SystemAllocation => Ok(()),
            Self::Execution(lease) => check_status(&lease.status(), None),
        }
    }
}

#[cfg(test)]
mod tests;

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
