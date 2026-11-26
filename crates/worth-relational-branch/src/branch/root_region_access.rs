#[path = "root_region_access/kind_directory.rs"]
pub mod kind_directory;
use super::*;

/// Custody-only accessors; partition reads go through [`RelationalBranchRoot::read`].
impl RelationalBranchRoot {
    pub fn reconstruct_region_head(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<
        Option<crate::storage::residency::ReconstructedRegionHead>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        self.regions
            .get(id)
            .map(|region| region.partition.reconstruct_head(lane, execution))
            .transpose()
    }

    /// Diagnostic census of one region's cell: (Head, Full) served with
    /// symbols admitted. No custody, admission or I/O.
    pub fn region_residency_census(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Option<(bool, bool)> {
        self.regions
            .get(id)
            .map(|region| region.partition.residency_census(symbols))
    }

    pub fn live_counts(&self) -> crate::storage::residency::LiveRecordCounts {
        self.regions.live_counts()
    }
    pub fn commit_change(&self) -> Option<&Arc<crate::branch::maintenance::CommitChange>> {
        self.committed.as_ref()?.commit_change.as_ref()
    }

    pub fn visit_content_cache_allocations(
        &self,
        partition_id: PartitionId,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        if let Some(region) = self.regions.get(partition_id) {
            region
                .partition
                .cell()
                .visit_resident_commitment_allocations(visitor);
        }
    }

    pub fn partition_ids(&self) -> Vec<PartitionId> {
        self.regions.materialize().keys().copied().collect()
    }

    pub fn partition_ids_iter(&self) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.partition_ids_iter()
    }

    pub fn try_visit_partitions<E>(
        &self,
        visit: impl FnMut(crate::branch::root_regions::RelationalPartitionVisit) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_visit_partitions(visit)
    }

    pub fn try_for_each_partition_id<E>(
        &self,
        visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_for_each_partition_id(visit)
    }

    pub fn checkpoint_regions(
        &self,
    ) -> Vec<super::super::root_checkpoint::RelationalRootRegionCheckpoint> {
        self.regions
            .values()
            .map(
                |region| super::super::root_checkpoint::RelationalRootRegionCheckpoint {
                    partition_id: region.partition_id,
                    content_digest: region.content_digest,
                    kind_commitment: region.kind_commitment,
                    directory: region.directory.clone(),
                    live_counts: region.live_counts,
                    entity_slots: region.entity_slots,
                    relation_slots: region.relation_slots,
                    unavailable_records: region.materialization_unavailable_records(),
                    cell: region.partition.cell().clone(),
                    kind_authority: region.partition.kind_authority(),
                },
            )
            .collect()
    }

    pub fn checkpoint_region_count(&self) -> usize {
        self.regions.len()
    }

    /// The caller admits its destination before retaining these fixed rows.
    /// Traversal borrows the immutable radix and allocates no roster or map.
    pub fn try_visit_checkpoint_regions<E>(
        &self,
        mut visit: impl FnMut(
            super::super::root_checkpoint::RelationalRootRegionCheckpoint,
        ) -> Result<(), E>,
    ) -> Result<(), E> {
        for id in self.regions.partition_ids_iter() {
            let region = self.regions.get(id)
                .expect(/* Proof: The borrowed iterator and lookup address the same immutable radix; each yielded ID has a present leaf. */ "selected region exists");
            visit(
                super::super::root_checkpoint::RelationalRootRegionCheckpoint {
                    partition_id: region.partition_id,
                    content_digest: region.content_digest,
                    kind_commitment: region.kind_commitment,
                    directory: region.directory.clone(),
                    live_counts: region.live_counts,
                    entity_slots: region.entity_slots,
                    relation_slots: region.relation_slots,
                    unavailable_records: region.materialization_unavailable_records(),
                    cell: region.partition.cell().clone(),
                    kind_authority: region.partition.kind_authority(),
                },
            )?;
        }
        Ok(())
    }
}

impl super::RelationalBranchRoot {
    pub fn source_retirement_regions(
        &self,
    ) -> super::super::root_regions::RelationalRegionSourceWalk {
        super::super::root_regions::RelationalRegionSourceWalk::new(self.regions.clone())
    }
}
