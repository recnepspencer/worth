//! Read-only metadata from the actual active request lineage.
use super::ACTIVE_METER;
use crate::authority::SerialMemoryBudget;

pub(crate) fn active_serial_memory() -> Option<SerialMemoryBudget> {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .last()
            .and_then(|meter| meter.borrow().limits.serial_memory.clone())
    })
}

pub(crate) fn active_deadline() -> Option<crate::MonotonicDeadline> {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .last()
            .and_then(|meter| meter.borrow().limits.deadline.clone())
    })
}

/// Cancellation-only reads do not consult clocks, accounting or scope observers.
/// A positive result returns to the original complete consultation owner.
pub(crate) fn active_cancelled() -> bool {
    ACTIVE_METER.with(|active| {
        active.borrow().last().is_some_and(|meter| {
            let meter = meter.borrow();
            meter.limits.tokens.iter().any(|token| token.is_cancelled())
                || meter.checkpoint_stop == Some(super::KernelStop::Cancelled)
        })
    })
}
