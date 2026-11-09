#[path = "root_region_access/kind_directory.rs"]
mod kind_directory;
use super::*;

impl RelationalBranchRoot {
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
    pub(crate) fn commit_change(
        &self,
    ) -> Option<&Arc<crate::storage::substrate::maintenance::CommitChange>> {
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
        match self.regions.get(id) {
            None => Ok(None),
            Some(region) => region.partition.accepted_head_lease().map(Some).ok_or(id),
        }
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
        self.regions.get(id)?.partition.accepted_full_partition()
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
        self.regions
            .get(id)
            .map(|region| region.partition.demand_full(execution, 1 << 30))
            .transpose()
    }

    pub(crate) fn accepted_head_region_view(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.regions
            .get(id)
            .and_then(|region| region.partition.accepted_head_view())
    }

    pub(crate) fn probe_head_region(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        match self.regions.get(id) {
            None => Probe::Absent,
            Some(region) => region
                .partition
                .admitted_head_view()
                .map_or(Probe::Cold, Probe::Resident),
        }
    }
    pub(crate) fn demand_region_head_cached(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        if let Some(region) = self.regions.get(id) {
            region.partition.demand_head_cached(execution, 1 << 30)?;
        }
        Ok(())
    }

    pub(crate) fn probe_head_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        match self.probe_head_region(id) {
            Probe::Resident(view) => {
                if self
                    .regions
                    .get(id)
                    .is_some_and(|region| region.partition.head_symbols_admitted(symbols))
                {
                    Probe::Resident(view)
                } else {
                    Probe::Cold
                }
            }
            other => other,
        }
    }

    pub(crate) fn demand_region_head_with_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if let Some(lease) = self.demand_region_head(execution, id)? {
            lease.admit_symbols(execution, symbols)?;
        }
        Ok(())
    }
    pub(crate) fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_regions_full(execution, symbols, self.partition_ids_iter())
    }
    pub(crate) fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        for partition in partitions {
            self.demand_region_head_with_symbols(execution, partition, symbols)?;
        }
        Ok(())
    }
    pub(crate) fn probe_full_region(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::PartitionProbe;
        match self.regions.get(id) {
            None => PartitionProbe::Absent,
            Some(region) => match region.partition.admitted_partition() {
                Some(partition) => PartitionProbe::Resident(partition),
                None => PartitionProbe::Cold,
            },
        }
    }
    pub(crate) fn probe_full_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::PartitionProbe as Probe;
        match self.probe_full_region(id) {
            Probe::Resident(partition) => {
                let Some(region) = self.regions.get(id) else {
                    return Probe::Absent;
                };
                if region.partition.admitted_full_lease().is_some_and(|lease| {
                    symbols.region_symbols_admitted(lease.symbol_admission_identity())
                }) {
                    Probe::Resident(partition)
                } else {
                    Probe::Cold
                }
            }
            other => other,
        }
    }
    pub(crate) fn demand_full_region_cached(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if let Some(region) = self.regions.get(id) {
            region.partition.demand_full(execution, 1 << 30)?;
        }
        Ok(())
    }
    pub(crate) fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        for partition in partitions {
            if let Some(region) = self.regions.get(partition) {
                region
                    .partition
                    .demand_full(execution, 1 << 30)?
                    .admit_symbols(execution, symbols)?;
            }
        }
        Ok(())
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
        self.regions
            .get(partition)
            .map(|region| region.partition.demand_head(execution, 1 << 30))
            .transpose()
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
        visit: impl FnMut(crate::branch::RelationalPartitionVisit) -> Result<(), E>,
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
