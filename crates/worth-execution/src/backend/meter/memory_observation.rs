//! Observe serial ceilings across allocation scopes, without lending their authority.
use super::ACTIVE_METER;

pub(crate) fn available_serial_memory_bytes() -> Option<u64> {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .iter()
            .filter_map(|meter| {
                meter
                    .borrow()
                    .limits
                    .serial_memory
                    .as_ref()
                    .and_then(crate::SerialMemoryBudget::available_memory_bytes)
            })
            .min()
    })
}
