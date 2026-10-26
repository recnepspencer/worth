use super::*;

impl RelationalBranchRoot {
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

    pub(crate) fn admitted_head_region(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.regions
            .get(id)
            .and_then(|region| region.partition.admitted_head_view())
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
                let lease = region.partition.lease();
                if symbols.region_symbols_admitted(lease.symbol_admission_identity()) {
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
        for partition in partitions {
            if let Some(region) = self.regions.get(partition) {
                region.partition.demand_full(execution, 1 << 30)?;
                region.partition.lease().admit_symbols(execution, symbols)?;
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
    pub(crate) fn partition_state(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.regions
            .get(partition_id)
            .map(|region| region.partition.as_ref())
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
                    entity_slots: region.entity_slots,
                    relation_slots: region.relation_slots,
                    unavailable_records: region.materialization_unavailable_records(),
                    cell: region.partition.cell().clone(),
                    kind_authority: region.partition.kind_authority(),
                },
            )
            .collect()
    }
}
