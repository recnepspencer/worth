//! Prepared group values and the next native partitioning handoff.
use super::*;
/// One marked partition, gathered again for its kernel.
pub(in super::super) struct Recomputed<Key, Gathered> {
    pub(in super::super) value: GatheredComputationPartition<Key, Gathered>,
    pub(in super::super) key: Arc<Key>,
    pub(in super::super) gather: Option<ComputationCallCharge>,
}

/// The run's items, keys and routes, carried or made again.
pub(in super::super) struct NextPartitioning<Item> {
    pub(in super::super) items: Items<Item>,
    pub(in super::super) digests: ItemDigests,
    /// `None` when the membership or an item's key was made again and its
    /// charge did not measure: the run cannot be carried again.
    pub(in super::super) membership: Option<RetainedCall>,
    pub(in super::super) item_keys:
        Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>,
    pub(in super::super) routing: Arc<ComputationPartitionRouting>,
    pub(in super::super) carried_membership: bool,
    pub(in super::super) carried_items: super::super::retained::CarriedItems,
}

/// What computing an incremental run produced.
pub(in super::super::super) struct ComputedIncremental<Reduced> {
    pub(in super::super::super) reduced: Reduced,
    /// The kernels' and the combines' work, as a full run charges it.
    pub(in super::super::super) computed_work: Option<u64>,
    pub(in super::super::super) completed: CompletedComputationRetention,
}
