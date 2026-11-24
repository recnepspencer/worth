use crate::history::data::record_allocation_class::RecordAllocationClass;
use crate::identity::data::PartitionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReclaimedRecordSlot {
    pub(super) class: RecordAllocationClass,
    pub(super) partition_id: PartitionId,
    pub(super) slot: usize,
}

impl ReclaimedRecordSlot {
    pub fn new(class: RecordAllocationClass, partition_id: PartitionId, slot: usize) -> Self {
        Self {
            class,
            partition_id,
            slot,
        }
    }
}
