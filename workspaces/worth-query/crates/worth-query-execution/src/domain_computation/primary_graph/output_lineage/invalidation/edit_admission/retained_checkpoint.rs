//! A replacement checkpoint names retained index bytes, never preparation bytes.

use super::InvalidationEditAdmission;
use worth_relational::facade::mvcc::CompanionPreflightStop;

/// Minted only by the admission that records surviving index allocations.
/// The private representation prevents a preparation-byte total from serving
/// as the start of a retained-index delta.
pub(in super::super) struct RetainedIndexCheckpoint {
    index_bytes: u64,
}

impl InvalidationEditAdmission {
    pub(in super::super) fn index_checkpoint(&self) -> RetainedIndexCheckpoint {
        RetainedIndexCheckpoint {
            index_bytes: self.charged_index_bytes(),
        }
    }

    pub(in super::super) fn index_bytes_since(
        &self,
        checkpoint: RetainedIndexCheckpoint,
    ) -> Result<u64, CompanionPreflightStop> {
        self.charged_index_bytes()
            .checked_sub(checkpoint.index_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)
    }
}
