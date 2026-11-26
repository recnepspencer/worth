use crate::storage::residency::{
    PreparedResidentRegion, RegionContentFailure, RelationalRegionRetention,
};

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::root::RelationalBranchRootCaptureDenial;

pub mod backing;
pub use backing::RelationalRootRegion;

impl RelationalRootRegion {
    pub(super) fn new(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        creation_root_id: u64,
        id: u64,
        partition: PartitionState,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        let partition_id = partition.partition_id;
        let prepared = PreparedResidentRegion::initial(lane, execution, partition, symbols)
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
            directory: prepared.directory,
            live_counts: prepared.live_counts,
            entity_slots: prepared.entity_slots,
            relation_slots: prepared.relation_slots,
            partition: prepared.retention,
            content_values_hashed: prepared.content_values_hashed,
            materialization_unavailable_records: prepared.unavailable_records,
            _memory: None,
        }
    }

    pub(super) fn from_checkpoint(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
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
            directory: checkpoint.directory,
            live_counts: checkpoint.live_counts,
            entity_slots: checkpoint.entity_slots,
            relation_slots: checkpoint.relation_slots,
            content_values_hashed: 0,
            materialization_unavailable_records: checkpoint.unavailable_records,
            _memory: None,
        }
    }

    pub(in crate::internal_port::branch) fn from_checkpoint_for_request(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        creation_root_id: u64,
        id: u64,
        checkpoint: super::root_checkpoint::RelationalRootRegionCheckpoint,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<std::sync::Arc<Self>, RelationalBranchRootCaptureDenial> {
        let memory = super::root::reserve_arc_memory::<Self>(Some(request))?;
        let mut region = Self::from_checkpoint(lane, creation_root_id, id, checkpoint);
        region._memory = memory;
        Ok(std::sync::Arc::new(region))
    }

    #[doc(hidden)]
    pub fn clone_with_inventory_for_test(
        &self,
        id: u64,
        partition_id: PartitionId,
        inventory: crate::storage::overlay::RelationalPartitionAllocationInventory,
    ) -> Self {
        Self {
            creation_root_id: self.creation_root_id,
            id,
            partition_id,
            partition: self.partition.clone(),
            allocation_inventory: Some(inventory),
            content_digest: self.content_digest,
            kind_commitment: self.kind_commitment,
            directory: std::sync::Arc::clone(&self.directory),
            live_counts: self.live_counts,
            entity_slots: self.entity_slots,
            relation_slots: self.relation_slots,
            content_values_hashed: self.content_values_hashed,
            materialization_unavailable_records: self.materialization_unavailable_records,
            _memory: None,
        }
    }

    pub(super) fn allocation_inventory(
        &self,
    ) -> Result<
        crate::storage::overlay::RelationalPartitionAllocationInventory,
        crate::storage::substrate::StorageAllocationOverflow,
    > {
        match self.allocation_inventory {
            Some(inventory) => Ok(inventory),
            None => self.partition.cell().resident_inventory(),
        }
    }

    pub(super) fn materialization_unavailable_records(&self) -> usize {
        self.materialization_unavailable_records
    }

    /// A cold region refuses rather than reading as absent.
    pub(super) fn resident_partition<'s>(
        &'s self,
        scope: &'s crate::storage::residency::OperationResidency,
    ) -> Result<&'s PartitionState, crate::storage::data::RelationalStorageUnavailable> {
        self.partition.try_admitted_partition(scope)?.ok_or(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(self.partition_id),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            },
        )
    }

    pub(super) fn observation(
        &self,
    ) -> Result<RelationalRootRegionObservation, crate::storage::substrate::StorageAllocationOverflow>
    {
        let inventory = self.allocation_inventory()?;
        let optional_cache_bytes = inventory
            .optional_cache_bytes
            .checked_add(self.partition.cell().resident_commitment_bytes())
            .ok_or(crate::storage::substrate::StorageAllocationOverflow)?;
        Ok(RelationalRootRegionObservation {
            creation_root_id: self.creation_root_id,
            region_id: self.id,
            partition_id: self.partition_id,
            root_region_bytes: std::mem::size_of::<Self>() as u64,
            partition_state_bytes: self.partition.cell().residency_allocation_bytes(),
            authoritative_bytes: inventory.authoritative_bytes,
            diagnostic_bytes: inventory.diagnostic_bytes,
            retention_metadata_bytes: inventory.retention_metadata_bytes,
            allocator_bookkeeping_bytes: inventory.allocator_bookkeeping_bytes,
            optional_cache_bytes,
        })
    }

    pub(super) fn observe_reclaimable_unique_authoritative_bytes(
        &self,
        reclaimable: &mut crate::storage::overlay::ReclaimableAllocations,
    ) -> Result<u64, crate::storage::substrate::StorageAllocationOverflow> {
        use crate::storage::substrate::StorageAllocationOverflow;
        let mut bytes = std::mem::size_of::<Self>() as u64;
        if self.partition.is_uniquely_owned() {
            bytes = bytes
                .checked_add(self.partition.cell().residency_allocation_bytes())
                .ok_or(StorageAllocationOverflow)?;
            self.partition
                .cell()
                .observe_reclaimable_resident_payload(reclaimable);
        }
        Ok(bytes)
    }
}

pub(super) fn capture_failure(
    partition: PartitionId,
    failure: RegionContentFailure,
) -> RelationalBranchRootCaptureDenial {
    match failure {
        RegionContentFailure::Execution(cause) => {
            RelationalBranchRootCaptureDenial::Execution(cause)
        }
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::Request(cause),
        ) => RelationalBranchRootCaptureDenial::Request(cause),
        RegionContentFailure::Storage(failure) => {
            RelationalBranchRootCaptureDenial::StorageUnavailable(failure)
        }
        RegionContentFailure::Adjacency => {
            RelationalBranchRootCaptureDenial::InvalidAdjacencyProjection(partition)
        }
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::Execution(cause),
        ) => RelationalBranchRootCaptureDenial::Execution(cause),
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::UnresolvedContentSymbol(symbol),
        ) => RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::InvalidHistoryCoverage,
        ) => RelationalBranchRootCaptureDenial::InvalidHistoryCoverage(partition),
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::Encoding,
        ) => RelationalBranchRootCaptureDenial::ContentEncoding(partition),
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::AllocationAccountingOverflow,
        ) => RelationalBranchRootCaptureDenial::AllocationAccountingOverflow(partition),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalRootRegionObservation {
    pub creation_root_id: u64,
    pub region_id: u64,
    pub partition_id: PartitionId,
    pub root_region_bytes: u64,
    pub partition_state_bytes: u64,
    pub authoritative_bytes: u64,
    pub diagnostic_bytes: u64,
    pub retention_metadata_bytes: u64,
    pub allocator_bookkeeping_bytes: u64,
    pub optional_cache_bytes: u64,
}
