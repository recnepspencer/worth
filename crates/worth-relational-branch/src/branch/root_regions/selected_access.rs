//! Head, Full, and kind reads of the actual selected radix backing. Every
//! admitted lease lives in the caller's operation residency, never the root.
use super::backing::RelationalPersistentRegionSet;
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::overlay::PartitionState;
use crate::storage::residency::{
    Full, Head, OperationResidency, PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView,
    RelationalResidencyDemandFailure as Failure, RelationalResidencyLease,
};
use worth_execution::ExecutionRequest;

impl RelationalPersistentRegionSet {
    pub fn project_selected_head<'s>(
        &'s self,
        partition: PartitionId,
        residency: &'s OperationResidency,
    ) -> SelectedSnapshotHead<'s> {
        SelectedSnapshotHead::new(
            partition,
            self.get(partition).map(|region| &region.partition),
            residency,
        )
    }

    pub fn accepted_head_region_witness(
        &self,
        id: PartitionId,
        residency: &OperationResidency,
    ) -> Result<Option<RelationalResidencyLease<Head>>, PartitionId> {
        self.project_selected_head(id, residency).witness()
    }

    pub fn accepted_head_region_view<'s>(
        &'s self,
        id: PartitionId,
        residency: &'s OperationResidency,
    ) -> Option<RelationalRegionHeadView<'s>> {
        self.project_selected_head(id, residency).view()
    }

    pub fn accepted_full_region<'s>(
        &'s self,
        id: PartitionId,
        residency: &'s OperationResidency,
    ) -> Option<&'s PartitionState> {
        self.get(id)?.partition.accepted_full_partition(residency)
    }

    pub fn demand_region_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
        residency: &OperationResidency,
    ) -> Result<Option<RelationalResidencyLease<Full>>, Failure> {
        self.get(id)
            .map(|region| region.partition.demand_full(residency, execution, 1 << 30))
            .transpose()
    }

    pub fn probe_head_region<'s>(
        &'s self,
        id: PartitionId,
        residency: &'s OperationResidency,
    ) -> PartitionHeadProbe<'s> {
        match self.get(id) {
            None => PartitionHeadProbe::Absent,
            Some(region) => region
                .partition
                .admitted_head_view(residency)
                .map_or(PartitionHeadProbe::Cold, PartitionHeadProbe::Resident),
        }
    }

    pub fn demand_region_head_cached(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
        residency: &OperationResidency,
    ) -> Result<(), Failure> {
        crate::execution::request_custody::check_request(execution).map_err(Failure::Request)?;
        if let Some(region) = self.get(id) {
            region
                .partition
                .demand_head_cached(residency, execution, 1 << 30)?;
        }
        Ok(())
    }

    pub fn probe_head_region_with_symbols<'s>(
        &'s self,
        id: PartitionId,
        symbols: &RuntimeSymbolTable,
        residency: &'s OperationResidency,
    ) -> PartitionHeadProbe<'s> {
        match self.probe_head_region(id, residency) {
            PartitionHeadProbe::Resident(view) => {
                if self.get(id).is_some_and(|region| {
                    region.partition.head_symbols_admitted(residency, symbols)
                }) {
                    PartitionHeadProbe::Resident(view)
                } else {
                    PartitionHeadProbe::Cold
                }
            }
            other => other,
        }
    }

    pub fn demand_region_head_with_symbols(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
        symbols: &RuntimeSymbolTable,
        residency: &OperationResidency,
    ) -> Result<(), Failure> {
        self.project_selected_head(id, residency)
            .demand_with_symbols(execution, symbols)
    }

    pub fn demand_regions_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
        residency: &OperationResidency,
    ) -> Result<(), Failure> {
        for partition in partitions {
            self.demand_region_head_with_symbols(execution, partition, symbols, residency)?;
        }
        Ok(())
    }

    pub fn probe_full_region<'s>(
        &'s self,
        id: PartitionId,
        residency: &'s OperationResidency,
    ) -> PartitionProbe<'s> {
        match self.get(id) {
            None => PartitionProbe::Absent,
            Some(region) => match region.partition.admitted_partition(residency) {
                Some(partition) => PartitionProbe::Resident(partition),
                None => PartitionProbe::Cold,
            },
        }
    }

    pub fn probe_full_region_with_symbols<'s>(
        &'s self,
        id: PartitionId,
        symbols: &RuntimeSymbolTable,
        residency: &'s OperationResidency,
    ) -> PartitionProbe<'s> {
        match self.probe_full_region(id, residency) {
            PartitionProbe::Resident(partition) => {
                let Some(region) = self.get(id) else {
                    return PartitionProbe::Absent;
                };
                if region
                    .partition
                    .admitted_full_lease(residency)
                    .is_some_and(|lease| {
                        symbols.region_symbols_admitted(lease.symbol_admission_identity())
                    })
                {
                    PartitionProbe::Resident(partition)
                } else {
                    PartitionProbe::Cold
                }
            }
            other => other,
        }
    }

    pub fn demand_full_region_cached(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
        residency: &OperationResidency,
    ) -> Result<(), Failure> {
        if let Some(region) = self.get(id) {
            region
                .partition
                .demand_full(residency, execution, 1 << 30)?;
        }
        Ok(())
    }

    pub fn demand_regions_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
        residency: &OperationResidency,
    ) -> Result<(), Failure> {
        crate::execution::request_custody::check_request(execution).map_err(Failure::Request)?;
        for partition in partitions {
            if let Some(region) = self.get(partition) {
                region
                    .partition
                    .demand_full(residency, execution, 1 << 30)?
                    .admit_symbols(execution, symbols)?;
            }
        }
        Ok(())
    }

    pub fn demand_region_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        partition: PartitionId,
        residency: &OperationResidency,
    ) -> Result<Option<RelationalResidencyLease<Head>>, Failure> {
        self.project_selected_head(partition, residency)
            .demand_lease(execution)
    }

    pub fn entity_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::EntityId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + '_ {
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
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::RelationId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + '_ {
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

    pub fn entity_ids_of_kind_with_residency<'a>(
        &'a self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
        residency: &'a OperationResidency,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::EntityId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + 'a {
        let regions: Box<dyn Iterator<Item = PartitionId> + 'a> = match partition {
            Some(id) => Box::new(std::iter::once(id)),
            None => Box::new(self.kinds.entity_regions(kind)),
        };
        regions.flat_map(move |id| {
            self.get(id)
                .filter(|region| region.directory.has_entity_kind(kind))
                .into_iter()
                .flat_map(
                    move |region| -> Box<dyn Iterator<Item = Result<_, _>> + 'a> {
                        if let Some(head) = region.partition.held_head_view(residency) {
                            Box::new(head.entity_ids_of_kind(kind).map(Ok))
                        } else {
                            Box::new(region.directory.entity_ids(kind))
                        }
                    },
                )
        })
    }

    pub fn relation_ids_of_kind_with_residency<'a>(
        &'a self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
        residency: &'a OperationResidency,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::RelationId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + 'a {
        let regions: Box<dyn Iterator<Item = PartitionId> + 'a> = match partition {
            Some(id) => Box::new(std::iter::once(id)),
            None => Box::new(self.kinds.relation_regions(kind)),
        };
        regions.flat_map(move |id| {
            self.get(id)
                .filter(|region| region.directory.has_relation_kind(kind))
                .into_iter()
                .flat_map(
                    move |region| -> Box<dyn Iterator<Item = Result<_, _>> + 'a> {
                        if let Some(head) = region.partition.held_head_view(residency) {
                            Box::new(head.relation_ids_of_kind(kind).map(Ok))
                        } else {
                            Box::new(region.directory.relation_ids(kind))
                        }
                    },
                )
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
