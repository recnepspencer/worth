use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::root::RelationalBranchRootCaptureDenial;

#[derive(Debug)]
pub(crate) struct RelationalRootRegion {
    pub(super) creation_root_id: u64,
    pub(super) id: u64,
    pub(super) partition_id: PartitionId,
    pub(super) partition: Arc<PartitionState>,
    pub(super) allocation_inventory:
        crate::storage::overlay::RelationalPartitionAllocationInventory,
    pub(super) content_digest: [u8; 32],
    materialization_unavailable_records: usize,
}

impl RelationalRootRegion {
    pub(super) fn new(
        creation_root_id: u64,
        id: u64,
        mut partition: PartitionState,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        partition.clear_runtime_pin_counters();
        let content_digest = partition
            .authoritative_content_digest(symbols)
            .map_err(|error| match error {
                crate::storage::overlay::PartitionContentDigestError::UnresolvedContentSymbol(
                    symbol,
                ) => RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
            })?;
        let allocation_inventory = partition.allocation_inventory();
        let materialization_unavailable_records = partition
            .entity_arena
            .lifecycle_counts()
            .unavailable
            .saturating_add(partition.relation_arena.lifecycle_counts().unavailable);
        Ok(Self {
            creation_root_id,
            id,
            partition_id: partition.partition_id,
            allocation_inventory,
            content_digest,
            materialization_unavailable_records,
            partition: Arc::new(partition),
        })
    }

    pub(super) fn materialization_unavailable_records(&self) -> usize {
        self.materialization_unavailable_records
    }

    pub(super) fn observation(&self) -> RelationalRootRegionObservation {
        debug_assert_eq!(self.partition.partition_id, self.partition_id);
        RelationalRootRegionObservation {
            creation_root_id: self.creation_root_id,
            region_id: self.id,
            partition_id: self.partition_id,
            root_region_bytes: std::mem::size_of::<Self>() as u64,
            partition_state_bytes: std::mem::size_of::<PartitionState>() as u64,
            authoritative_bytes: self.allocation_inventory.authoritative_bytes,
            diagnostic_bytes: self.allocation_inventory.diagnostic_bytes,
            retention_metadata_bytes: self.allocation_inventory.retention_metadata_bytes,
            allocator_bookkeeping_bytes: self.allocation_inventory.allocator_bookkeeping_bytes,
            optional_cache_bytes: self.allocation_inventory.optional_cache_bytes,
        }
    }

    pub(super) fn reclaimable_unique_authoritative_bytes(&self) -> u64 {
        let mut bytes = std::mem::size_of::<Self>() as u64;
        if Arc::strong_count(&self.partition) == 1 {
            bytes = bytes
                .saturating_add(std::mem::size_of::<PartitionState>() as u64)
                .saturating_add(self.allocation_inventory.authoritative_bytes);
        }
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalRootRegionObservation {
    pub(crate) creation_root_id: u64,
    pub(crate) region_id: u64,
    pub(crate) partition_id: PartitionId,
    pub(crate) root_region_bytes: u64,
    pub(crate) partition_state_bytes: u64,
    pub(crate) authoritative_bytes: u64,
    pub(crate) diagnostic_bytes: u64,
    pub(crate) retention_metadata_bytes: u64,
    pub(crate) allocator_bookkeeping_bytes: u64,
    pub(crate) optional_cache_bytes: u64,
}
