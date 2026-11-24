#[path = "root_region_access/kind_directory.rs"]
mod kind_directory;
use super::*;
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::branch::symbol_read::SymbolAdmittedRootRead;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

impl RelationalBranchRoot {
    pub(crate) fn symbol_admitted_projection<'a>(
        &'a self,
        symbols: &'a RuntimeSymbolTable,
    ) -> SymbolAdmittedRootRead<'a> {
        SymbolAdmittedRootRead::new(&self.regions, symbols)
    }

    pub(crate) fn project_selected_head(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        self.regions.project_selected_head(partition)
    }

    pub(crate) fn reconstruct_region_head(
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

    pub(crate) fn live_counts(&self) -> crate::storage::residency::LiveRecordCounts {
        self.regions.live_counts()
    }
    pub(crate) fn commit_change(&self) -> Option<&Arc<crate::branch::maintenance::CommitChange>> {
        self.committed.as_ref()?.commit_change.as_ref()
    }

    pub(crate) fn accepted_head_region_witness(
        &self,
        id: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>,
        >,
        PartitionId,
    > {
        self.regions.accepted_head_region_witness(id)
    }

    pub(crate) fn resident_full_region_witness(
        &self,
        id: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
        >,
        crate::storage::data::RelationalStorageUnavailable,
    > {
        self.regions
            .get(id)
            .map(|region| region.partition.pin_lease())
            .transpose()
    }

    pub(crate) fn accepted_full_region(&self, id: PartitionId) -> Option<&PartitionState> {
        self.regions.accepted_full_region(id)
    }
    pub(crate) fn demand_region_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
        >,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        self.regions.demand_region_full(execution, id)
    }

    pub(crate) fn accepted_head_region_view(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.regions.accepted_head_region_view(id)
    }

    pub(crate) fn probe_head_region(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.regions.probe_head_region(id)
    }
    pub(crate) fn demand_region_head_cached(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.regions.demand_region_head_cached(execution, id)
    }

    pub(crate) fn peek_head_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        match self.regions.get(id) {
            None => Probe::Absent,
            Some(region) => match region.partition.peek_head_view_with_symbols(symbols) {
                Some(view) => Probe::Resident(view),
                None => Probe::Cold,
            },
        }
    }

    pub(crate) fn peek_full_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::PartitionProbe as Probe;
        match self.regions.get(id) {
            None => Probe::Absent,
            Some(region) => match region.partition.peek_full_partition_with_symbols(symbols) {
                Some(partition) => Probe::Resident(partition),
                None => Probe::Cold,
            },
        }
    }

    pub(crate) fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.regions.demand_all_full(execution, symbols)
    }
    pub(crate) fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.regions
            .demand_regions_head(execution, symbols, partitions)
    }
    pub(crate) fn probe_full_region(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        self.regions.probe_full_region(id)
    }
    pub(crate) fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.regions
            .demand_regions_full(execution, symbols, partitions)
    }
    pub(crate) fn demand_region_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>,
        >,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        self.regions.demand_region_head(execution, partition)
    }
    pub(crate) fn partition_state(
        &self,
        partition_id: PartitionId,
    ) -> Result<Option<&PartitionState>, crate::storage::data::RelationalStorageUnavailable> {
        let Some(region) = self.regions.get(partition_id) else {
            return Ok(None);
        };
        region.resident_partition().map(Some)
    }

    pub(crate) fn visit_content_cache_allocations(
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

    pub(crate) fn partition_ids(&self) -> Vec<PartitionId> {
        self.regions.materialize().keys().copied().collect()
    }

    pub(crate) fn partition_ids_iter(&self) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.partition_ids_iter()
    }

    pub(crate) fn try_visit_partitions<E>(
        &self,
        visit: impl FnMut(crate::branch::root_regions::RelationalPartitionVisit) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_visit_partitions(visit)
    }

    pub(crate) fn try_for_each_partition_id<E>(
        &self,
        visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_for_each_partition_id(visit)
    }

    pub(crate) fn checkpoint_regions(
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

    pub(crate) fn checkpoint_region_count(&self) -> usize {
        self.regions.len()
    }

    /// The caller admits its destination before retaining these fixed rows.
    /// Traversal borrows the immutable radix and allocates no roster or map.
    pub(crate) fn try_visit_checkpoint_regions<E>(
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
    pub(crate) fn source_retirement_regions(
        &self,
    ) -> super::super::root_regions::RelationalRegionSourceWalk {
        super::super::root_regions::RelationalRegionSourceWalk::new(self.regions.clone())
    }
}
