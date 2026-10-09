use super::super::receipt_construction::BlobStreamingReadCounterBackedPerformanceReceipt;
use crate::{
    BlobGeneration, BlobObjectId, BlobStreamingReadCounterSnapshot, ChunkTreeRoot,
    LogicalContentDigest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobStreamingVerifiedRead {
    pub(crate) object_id: BlobObjectId,
    pub(crate) generation: BlobGeneration,
    pub(crate) chunk_tree_root: ChunkTreeRoot,
    pub(crate) logical_content_digest: LogicalContentDigest,
    pub(crate) counters: BlobStreamingReadCounterSnapshot,
    pub(crate) performance: BlobStreamingReadCounterBackedPerformanceReceipt,
}

impl BlobStreamingVerifiedRead {
    #[cfg(test)]
    /// Constructs a comparison-only law fixture; never certifies Store byte I/O.
    pub(crate) fn for_content_comparison_test(
        object_id: BlobObjectId,
        generation: BlobGeneration,
        chunk_tree_root: ChunkTreeRoot,
        logical_content_digest: LogicalContentDigest,
        bytes_read: u64,
    ) -> Self {
        use super::super::receipt_construction::performance::counter_backed_streaming_read_performance_receipt;
        let counters = BlobStreamingReadCounterSnapshot::start(
            worth_store_budgets::CounterEvidenceStrength::Exact,
        )
        .observe_read_window(bytes_read)
        .record_verified_chunk();
        let performance = counter_backed_streaming_read_performance_receipt(counters);
        Self {
            object_id,
            generation,
            chunk_tree_root,
            logical_content_digest,
            counters,
            performance,
        }
    }

    pub const fn object_id(&self) -> &BlobObjectId {
        &self.object_id
    }

    pub const fn generation(&self) -> BlobGeneration {
        self.generation
    }

    pub const fn chunk_tree_root(&self) -> &ChunkTreeRoot {
        &self.chunk_tree_root
    }

    pub const fn logical_content_digest(&self) -> &LogicalContentDigest {
        &self.logical_content_digest
    }

    pub const fn counters(&self) -> BlobStreamingReadCounterSnapshot {
        self.counters
    }

    pub const fn counter_backed_performance_receipt(
        &self,
    ) -> &BlobStreamingReadCounterBackedPerformanceReceipt {
        &self.performance
    }
}
