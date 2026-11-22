//! Prepared group values and the next native partitioning handoff.
use super::*;
/// One marked partition, gathered again for its kernel.
pub struct Recomputed<Key, Gathered> {
    pub value: GatheredComputationPartition<Key, Gathered>,
    pub key: Arc<Key>,
    pub gather: Option<ComputationCallCharge>,
}

/// The run's items, keys and routes, carried or made again.
pub struct NextPartitioning<Item> {
    pub items: Items<Item>,
    pub digests: ItemDigests,
    /// `None` when the membership or an item's key was made again and its
    /// charge did not measure: the run cannot be carried again.
    pub membership: Option<RetainedCall>,
    pub item_keys: Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>,
    pub routing: Arc<ComputationPartitionRouting>,
    pub carried_membership: bool,
    pub carried_items: super::super::retained::CarriedItems,
}

/// What computing an incremental run produced.
pub struct ComputedIncremental<Reduced> {
    pub reduced: Reduced,
    /// The kernels' and the combines' work, as a full run charges it.
    pub computed_work: Option<u64>,
    pub completed: CompletedComputationRetention,
}
