//! Head, Full, and kind reads of the actual selected radix backing.
use super::backing::RelationalPersistentRegionSet;
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};

impl RelationalPersistentRegionSet {
    pub fn project_selected_head(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        SelectedSnapshotHead::new(
            partition,
            self.get(partition).map(|region| &region.partition),
        )
    }

    pub fn accepted_head_region_witness(
        &self,
        id: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>,
        >,
        PartitionId,
    > {
        self.project_selected_head(id).witness()
    }

    pub fn accepted_head_region_view(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.project_selected_head(id).view()
    }

    pub fn accepted_full_region(&self, id: PartitionId) -> Option<&PartitionState> {
        self.get(id)?.partition.accepted_full_partition()
    }

    pub fn demand_region_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
        >,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        self.get(id)
            .map(|region| region.partition.demand_full(execution, 1 << 30))
            .transpose()
    }

    pub fn probe_head_region(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        match self.get(id) {
            None => Probe::Absent,
            Some(region) => region
                .partition
                .admitted_head_view()
                .map_or(Probe::Cold, Probe::Resident),
        }
    }

    pub fn demand_region_head_cached(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::request_custody::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        if let Some(region) = self.get(id) {
            region.partition.demand_head_cached(execution, 1 << 30)?;
        }
        Ok(())
    }

    pub fn probe_head_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        match self.probe_head_region(id) {
            Probe::Resident(view) => {
                if self
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

    pub fn demand_region_head_with_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.project_selected_head(id)
            .demand_with_symbols(execution, symbols)
    }

    pub fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_regions_full(execution, symbols, self.partition_ids_iter())
    }

    pub fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        for partition in partitions {
            self.demand_region_head_with_symbols(execution, partition, symbols)?;
        }
        Ok(())
    }

    pub fn probe_full_region(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::PartitionProbe;
        match self.get(id) {
            None => PartitionProbe::Absent,
            Some(region) => match region.partition.admitted_partition() {
                Some(partition) => PartitionProbe::Resident(partition),
                None => PartitionProbe::Cold,
            },
        }
    }

    pub fn probe_full_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::PartitionProbe as Probe;
        match self.probe_full_region(id) {
            Probe::Resident(partition) => {
                let Some(region) = self.get(id) else {
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

    pub fn demand_full_region_cached(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if let Some(region) = self.get(id) {
            region.partition.demand_full(execution, 1 << 30)?;
        }
        Ok(())
    }

    pub fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::request_custody::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        for partition in partitions {
            if let Some(region) = self.get(partition) {
                region
                    .partition
                    .demand_full(execution, 1 << 30)?
                    .admit_symbols(execution, symbols)?;
            }
        }
        Ok(())
    }

    pub fn demand_region_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: PartitionId,
    ) -> Result<
        Option<
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>,
        >,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        self.project_selected_head(partition)
            .demand_lease(execution)
    }

    pub fn entity_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::EntityId> + '_ {
        let regions: Box<dyn Iterator<Item = PartitionId> + '_> = match partition {
            Some(id) => Box::new(std::iter::once(id)),
            None => Box::new(self.kinds.entity_regions(kind)),
        };
        regions.flat_map(move |id| {
            self.get(id)
                .into_iter()
                .flat_map(move |region| region.directory.entity_ids(kind))
        })
    }

    pub fn relation_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        let regions: Box<dyn Iterator<Item = PartitionId> + '_> = match partition {
            Some(id) => Box::new(std::iter::once(id)),
            None => Box::new(self.kinds.relation_regions(kind)),
        };
        regions.flat_map(move |id| {
            self.get(id)
                .into_iter()
                .flat_map(move |region| region.directory.relation_ids(kind))
        })
    }

    pub fn entity_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.kinds.entity_regions(kind)
    }

    pub fn relation_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.kinds.relation_regions(kind)
    }
}

impl PartitionAccess for RelationalPersistentRegionSet {
    fn probe_full_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        self.probe_full_region(id)
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_full_region_cached(execution, id)
    }
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_region(partition_id) {
            crate::storage::residency::PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for RelationalPersistentRegionSet {
    fn get_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        // Fresh admission belongs to probe_head_partition; a paid read consumes
        // the immutable counted custody already retained by this root.
        self.accepted_head_region_view(id)
    }
    fn probe_head_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.probe_head_region(id)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_region_head_cached(execution, id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partition_ids_iter().collect()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(RelationalPersistentRegionSet::partition_ids_iter(self))
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        use crate::storage::overlay::collect_partition_ids;
        collect_partition_ids(
            self.len(),
            RelationalPersistentRegionSet::partition_ids_iter(self),
            memory,
        )
    }
}
