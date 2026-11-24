//! Immutable radix backing lent to selected reads and updated by root publication.
use crate::branch::root_region::backing::RelationalRootRegion;
use crate::identity::data::PartitionId;
use std::sync::Arc;

pub(super) const PARTITION_KEY_BITS: u32 = u32::BITS;

/// Exact immutable radix index whose fixed 32-bit paths keep ordinary reads
/// and replacements independent of publication ancestry.
#[derive(Debug)]
pub struct RelationalPersistentRegionSet {
    pub(super) set_id: u64,
    pub(super) index_root: Option<Arc<RelationalPersistentRegionNode>>,
    pub(super) count: usize,
    pub(super) live_counts: crate::storage::residency::LiveRecordCounts,
    pub(super) kinds: crate::storage::residency::KindRegionDirectory,
    pub(super) materialization_unavailable_records: usize,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub(super) struct RelationalPersistentRegionNode {
    pub(super) node_id: u64,
    pub(super) zero: Option<Arc<Self>>,
    pub(super) one: Option<Arc<Self>>,
    pub(super) leaf: Option<RelationalPersistentRegionLeaf>,
    pub(super) commitment: [u8; 32],
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub(super) enum RelationalPersistentRegionLeaf {
    Present(Box<RelationalRootRegionReference>),
    Removed(Box<RelationalRemovedPartition>),
}
#[derive(Debug)]
pub(super) struct RelationalRootRegionReference {
    pub(super) region: Arc<RelationalRootRegion>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub(super) struct RelationalRemovedPartition {
    pub(super) partition_id: PartitionId,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalPersistentRegionSet {
    pub fn get(&self, partition_id: PartitionId) -> Option<&Arc<RelationalRootRegion>> {
        get_region(self.index_root.as_ref(), partition_id)
    }

    pub fn live_counts(&self) -> crate::storage::residency::LiveRecordCounts {
        self.live_counts
    }
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn materialization_unavailable_records(&self) -> usize {
        self.materialization_unavailable_records
    }
}

pub(super) fn get_region(
    mut current: Option<&Arc<RelationalPersistentRegionNode>>,
    partition_id: PartitionId,
) -> Option<&Arc<RelationalRootRegion>> {
    for depth in 0..PARTITION_KEY_BITS {
        let node = current?;
        current = if key_bit(partition_id, depth) == 0 {
            node.zero.as_ref()
        } else {
            node.one.as_ref()
        };
    }
    match current?.leaf.as_ref()? {
        RelationalPersistentRegionLeaf::Present(region) => Some(&region.region),
        RelationalPersistentRegionLeaf::Removed(_) => None,
    }
}

pub(super) fn key_bit(partition_id: PartitionId, depth: u32) -> u32 {
    (partition_id.0 >> (PARTITION_KEY_BITS - depth - 1)) & 1
}
