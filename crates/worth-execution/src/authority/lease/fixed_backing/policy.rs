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

impl<'scope, 'authority> ExecutionAllocationPolicy<'scope, 'authority> {
    /// Reserve declared owned payload before copying or allocating it. This
    /// performs no allocation and never adopts or funds an existing raw copy.
    /// The owner must retain the reservation until that payload's final drop.
    pub fn reserve_memory(
        self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, ExecutionAllocationDenial> {
        self.with_effective_lease(Some(bytes), |lease| {
            let Some(lease) = lease else {
                return Ok(None);
            };
            let status = lease.status();
            check_status(&status, Some(bytes))?;
            if !crate::backend::RunLimits::native_accounting(Some(lease)) {
                return Ok(None);
            }
            let reservation = lease.reserve_memory(bytes).map_err(|cause| {
                ExecutionAllocationDenial::new(
                    ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(cause)),
                    Some(bytes),
                )
            });
            // Preserve interruption precedence, also when memory was exhausted.
            check_status(&status, Some(bytes))?;
            Ok(Some(reservation?))
        })
    }

    /// Check before a layout quote exists; a stop therefore carries no quote.
    pub fn check_live(self) -> Result<(), ExecutionAllocationDenial> {
        self.status(None)?
            .as_ref()
            .map_or(Ok(()), |status| check_status(status, None))
    }

    /// Capture controls from the same allocation owner as its funding. A
    /// builder keeps this exact status after its calling scope has returned.
    pub(super) fn status(
        self,
        quote: Option<u64>,
    ) -> Result<Option<ExecutionLeaseStatus>, ExecutionAllocationDenial> {
        self.with_effective_lease(quote, |lease| Ok(lease.map(ExecutionResourceLease::status)))
    }

    fn with_effective_lease<T>(
        self,
        quote: Option<u64>,
        use_lease: impl FnOnce(
            Option<&ExecutionResourceLease<'authority>>,
        ) -> Result<T, ExecutionAllocationDenial>,
    ) -> Result<T, ExecutionAllocationDenial> {
        let Self::Execution(original) = self else {
            return use_lease(None);
        };
        let active = original.current_descendant();
        let lease = active.as_ref().unwrap_or(original);
        if active.is_none() && !lease.accepts_current_worker_context() {
            // Misuse inside another lease's worker stops that invoking run,
            // as every other unrelated admission does, even when swallowed.
            return Err(ExecutionAllocationDenial::new(
                ExecutionAllocationDenialKind::Lease(crate::backend::request_stop(
                    LeaseDenial::UnrelatedNestedLease,
                )),
                quote,
            ));
        }
        use_lease(Some(lease))
    }

    /// Whether this allocation's metadata uses accounted bookkeeping.
    /// System allocation inherits an active scope's mode; outside a scope it
    /// retains the explicitly accounted layout. This read-only projection
    /// admits no allocation and grants no unrelated lease authority.
    pub fn uses_accounting(self) -> bool {
        match self {
            Self::Execution(lease) => crate::backend::RunLimits::native_accounting(Some(lease)),
            Self::SystemAllocation => {
                !crate::backend::RunLimits::has_parent()
                    || crate::backend::RunLimits::native_accounting(None)
            }
        }
    }
}

impl ExecutionMemoryReservation {
    /// Grow mutable allocation backing under its bound lease's controls.
    /// `bytes` is the new total and cannot shrink the prior reservation.
    /// A stop takes precedence over exhausted memory and keeps the prior charge.
    /// Serial-budget tickets have no lease control; their budget still applies.
    /// This grants no request, model, or publication authority. Cleanup uses
    /// ordinary `resize`, which must remain possible after cancellation.
    pub fn grow_for_allocation(&mut self, bytes: u64) -> Result<(), ExecutionAllocationDenial> {
        let status = self.allocation_status();
        if let Some(status) = &status {
            check_status(status, Some(bytes))?;
        }
        let prior = self.bytes();
        if bytes < prior {
            return Err(ExecutionAllocationDenial::new(
                ExecutionAllocationDenialKind::Layout,
                Some(bytes),
            ));
        }
        let growth = self.resize(bytes).map_err(ExecutionAllocationDenial::from);
        if let Some(status) = &status
            && let Err(stop) = check_status(status, Some(bytes))
        {
            if growth.is_ok() {
                self.resize(prior)
                    .expect("releasing newly admitted growth cannot exhaust memory");
            }
            return Err(stop);
        }
        growth
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
