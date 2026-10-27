use crate::storage::residency::{
    PreparedResidentRegion, RegionContentFailure, RelationalRegionRetention,
};

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::root::RelationalBranchRootCaptureDenial;

#[derive(Debug)]
pub(crate) struct RelationalRootRegion {
    pub(super) creation_root_id: u64,
    pub(super) id: u64,
    pub(super) partition_id: PartitionId,
    pub(super) partition: RelationalRegionRetention,
    allocation_inventory: Option<crate::storage::overlay::RelationalPartitionAllocationInventory>,
    pub(super) content_digest: [u8; 32],
    pub(super) kind_commitment: crate::storage::residency::RelationalRegionKindCommitment,
    pub(super) entity_slots: usize,
    pub(super) relation_slots: usize,
    pub(super) content_values_hashed: u64,
    materialization_unavailable_records: usize,
}

impl RelationalRootRegion {
    pub(super) fn new(
        creation_root_id: u64,
        id: u64,
        partition: PartitionState,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        let partition_id = partition.partition_id;
        let prepared = PreparedResidentRegion::initial(partition, symbols)
            .map_err(|failure| capture_failure(partition_id, failure))?;
        Ok(Self::from_prepared(creation_root_id, id, prepared))
    }

    pub(super) fn from_prepared(
        creation_root_id: u64,
        id: u64,
        prepared: PreparedResidentRegion,
    ) -> Self {
        Self {
            creation_root_id,
            id,
            partition_id: prepared.partition_id,
            allocation_inventory: Some(prepared.inventory),
            content_digest: prepared.content_digest,
            kind_commitment: prepared.kind_commitment,
            entity_slots: prepared.entity_slots,
            relation_slots: prepared.relation_slots,
            partition: prepared.retention,
            content_values_hashed: prepared.content_values_hashed,
            materialization_unavailable_records: prepared.unavailable_records,
        }
    }

    pub(super) fn from_checkpoint(
        creation_root_id: u64,
        id: u64,
        checkpoint: super::root_checkpoint::RelationalRootRegionCheckpoint,
    ) -> Self {
        Self {
            creation_root_id,
            id,
            partition_id: checkpoint.partition_id,
            partition: RelationalRegionRetention::new(checkpoint.cell, checkpoint.kind_authority),
            allocation_inventory: None,
            content_digest: checkpoint.content_digest,
            kind_commitment: checkpoint.kind_commitment,
            entity_slots: checkpoint.entity_slots,
            relation_slots: checkpoint.relation_slots,
            content_values_hashed: 0,
            materialization_unavailable_records: checkpoint.unavailable_records,
        }
    }

    pub(super) fn allocation_inventory(
        &self,
    ) -> crate::storage::overlay::RelationalPartitionAllocationInventory {
        self.allocation_inventory
            .unwrap_or_else(|| self.partition.cell().resident_inventory())
    }

    pub(super) fn materialization_unavailable_records(&self) -> usize {
        self.materialization_unavailable_records
    }

    pub(super) fn observation(&self) -> RelationalRootRegionObservation {
        let inventory = self.allocation_inventory();
        RelationalRootRegionObservation {
            creation_root_id: self.creation_root_id,
            region_id: self.id,
            partition_id: self.partition_id,
            root_region_bytes: std::mem::size_of::<Self>() as u64,
            partition_state_bytes: self.partition.cell().residency_allocation_bytes(),
            authoritative_bytes: inventory.authoritative_bytes,
            diagnostic_bytes: inventory.diagnostic_bytes,
            retention_metadata_bytes: inventory.retention_metadata_bytes,
            allocator_bookkeeping_bytes: inventory.allocator_bookkeeping_bytes,
            optional_cache_bytes: inventory.optional_cache_bytes
                + self.partition.cell().resident_commitment_bytes(),
        }
    }

    pub(super) fn reclaimable_unique_authoritative_bytes(&self) -> u64 {
        let mut bytes = std::mem::size_of::<Self>() as u64;
        if self.partition.is_uniquely_owned() {
            bytes = bytes.saturating_add(self.partition.cell().residency_allocation_bytes());
            bytes = bytes.saturating_add(
                self.partition
                    .cell()
                    .reclaimable_unique_resident_payload_bytes(),
            );
        }
        bytes
    }
}

pub(super) fn capture_failure(
    partition: PartitionId,
    failure: RegionContentFailure,
) -> RelationalBranchRootCaptureDenial {
    match failure {
        RegionContentFailure::Storage(failure) => {
            RelationalBranchRootCaptureDenial::StorageUnavailable(failure)
        }
        RegionContentFailure::Adjacency => {
            RelationalBranchRootCaptureDenial::InvalidAdjacencyProjection(partition)
        }
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::UnresolvedContentSymbol(symbol),
        ) => RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::InvalidHistoryCoverage,
        ) => RelationalBranchRootCaptureDenial::InvalidHistoryCoverage(partition),
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
