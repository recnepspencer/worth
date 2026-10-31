//! Read-only metadata from the actual active request lineage.
use super::ACTIVE_METER;
use crate::authority::SerialMemoryBudget;
use std::time::Instant;

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
