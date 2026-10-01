//! Heap backing owned by selected control custody, excluding shared checkpoints.

use super::{VerifiedSelectedCheckpointCustody, WitnessedSelectedControlFrame};

impl WitnessedSelectedControlFrame {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.bytes.capacity()).ok()
    }
}

impl VerifiedSelectedCheckpointCustody {
    /// The checkpoint stream is shared and is counted once by the caller.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.batches.len()).ok()?.checked_mul(
            u64::try_from(std::mem::size_of::<
                worth_store_physical_format::ReleaseCheckpointBatchV1,
            >())
            .ok()?,
        )
    }
}
