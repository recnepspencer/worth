//! Surviving index allocation quotes, distinct from preparation and other tickets.
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{index_capacity, IndexAdmission};

/// Only state-edit meters implement this: publication root/hint preparation
/// uses separate capacity tickets and cannot silently become an index edit.
pub(in super::super) trait RetainedIndexAdmission: IndexAdmission {
    /// Record already-prepared storage that survives in the resulting index.
    /// The preparation charges remain with the original owner and allowance.
    fn record_index_bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop>;

    fn index_bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        self.bytes(bytes)?;
        self.record_index_bytes(bytes)
    }

    fn index_edit<K, V>(&mut self, entries: usize) -> Result<(), CompanionPreflightStop> {
        self.ordered_edit::<K, V>(entries)?;
        self.record_index_bytes(
            index_capacity::ordered_insertion_bytes::<K, V>(entries)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )
    }

    fn index_remove<K, V>(&mut self, entries: usize) -> Result<(), CompanionPreflightStop> {
        self.ordered_remove::<K, V>(entries)?;
        self.record_index_bytes(
            index_capacity::ordered_edit_bytes::<K, V>(entries)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )
    }

}
