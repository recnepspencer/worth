//! Read-only metadata from the actual active request lineage.
use super::{NestedStopOwner, ACTIVE_METER};
use crate::authority::SerialMemoryBudget;
use std::time::Instant;

/// Observe a joined stop before a child scope replaces the invoking meter.
/// Unlike polling, this neither clears its owner nor latches a checkpoint.
pub(crate) fn active_nested_stop_owner() -> Option<NestedStopOwner> {
    ACTIVE_METER.with(|active| active.borrow().last()?.borrow().nested_stop.owner())
}

pub(crate) fn active_serial_memory() -> Option<SerialMemoryBudget> {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .last()
            .and_then(|meter| meter.borrow().limits.serial_memory.clone())
    })
}

pub(crate) fn active_deadline() -> Option<Instant> {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .last()
            .and_then(|meter| meter.borrow().limits.deadline)
    })
}
