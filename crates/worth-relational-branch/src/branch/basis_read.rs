//! An admitted basis reads its root within its own transaction residency, so
//! a preparation's declared demands and its later accepted reads agree while
//! the shared root retains nothing (Store M4 D1).
use super::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::storage::overlay::{
    CurrentPartitionAccess, PartitionAccess, PartitionIdMemory, PartitionIdsDenial, PartitionState,
};
use crate::storage::residency::{
    PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView,
    RelationalResidencyDemandFailure as Failure,
};
use worth_execution::ExecutionRequest;

impl AdmittedRelationalBranchBasis {
    fn regions(&self) -> &super::root_regions::backing::RelationalPersistentRegionSet {
        self.inner.root.regions_for_projection()
    }

    /// This basis' transaction residency.
    pub fn residency(&self) -> &crate::storage::residency::OperationResidency {
        &self.residency
    }

    pub fn shared_residency(
        &self,
    ) -> std::sync::Arc<crate::storage::residency::OperationResidency> {
        self.residency.clone()
    }

    /// A cold region refuses rather than reading as absent.
    pub fn partition_state(
        &self,
        id: PartitionId,
    ) -> Result<Option<&PartitionState>, RelationalStorageUnavailable> {
        let Some(region) = self.regions().get(id) else {
            return Ok(None);
        };
        region
            .partition
            .try_admitted_partition(&*self.residency)?
            .map(Some)
            .ok_or(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(id),
                cause: RelationalStorageUnavailableCause::SourceUnavailable,
            })
    }

    pub fn demand_regions_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), Failure> {
        self.regions()
            .demand_regions_full(execution, symbols, partitions, &self.residency)
    }

    pub fn symbol_admitted_projection<'a>(
        &'a self,
        symbols: &'a RuntimeSymbolTable,
    ) -> super::symbol_read::SymbolAdmittedRootRead<'a> {
        super::symbol_read::SymbolAdmittedRootRead::new(self.regions(), symbols, &self.residency)
    }

    pub fn demand_regions_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), Failure> {
        self.regions()
            .demand_regions_head(execution, symbols, partitions, &self.residency)
    }

    pub fn demand_all_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
    ) -> Result<(), Failure> {
        self.demand_regions_full(execution, symbols, self.regions().partition_ids_iter())
    }
}

impl PartitionAccess for AdmittedRelationalBranchBasis {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.regions().probe_full_region(id, &self.residency)
    }
    fn demand_full_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), Failure> {
        self.regions()
            .demand_full_region_cached(execution, id, &self.residency)
    }
}

impl CurrentPartitionAccess for AdmittedRelationalBranchBasis {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.regions()
            .accepted_head_region_view(id, &self.residency)
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.regions().probe_head_region(id, &self.residency)
    }
    fn demand_head_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), Failure> {
        self.regions()
            .demand_region_head_cached(execution, id, &self.residency)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.regions().partition_ids_iter().collect()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.regions().partition_ids_iter())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        crate::storage::overlay::collect_partition_ids(
            self.regions().len(),
            self.regions().partition_ids_iter(),
            memory,
        )
    }
}
