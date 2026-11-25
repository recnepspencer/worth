//! Bind shared branch cells to the reading runtime's dictionary authority.
//! Admitted leases live in the enclosing operation's residency.
use crate::branch::root_regions::backing::RelationalPersistentRegionSet;
use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess, PartitionState};
use crate::storage::residency::{
    OperationResidency, PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView,
    RelationalResidencyDemandFailure,
};

pub struct SymbolAdmittedRootRead<'read> {
    regions: &'read RelationalPersistentRegionSet,
    symbols: &'read RuntimeSymbolTable,
    residency: &'read OperationResidency,
}

impl<'read> SymbolAdmittedRootRead<'read> {
    pub(super) fn new(
        regions: &'read RelationalPersistentRegionSet,
        symbols: &'read RuntimeSymbolTable,
        residency: &'read OperationResidency,
    ) -> Self {
        Self {
            regions,
            symbols,
            residency,
        }
    }
}

impl PartitionAccess for SymbolAdmittedRootRead<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.regions
            .probe_full_region_with_symbols(id, self.symbols, self.residency)
    }
    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions
            .demand_regions_full(execution, self.symbols, [id], self.residency)
    }
}

impl CurrentPartitionAccess for SymbolAdmittedRootRead<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        match self.probe_head_partition(id) {
            PartitionHeadProbe::Resident(view) => Some(view),
            _ => None,
        }
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.regions
            .probe_head_region_with_symbols(id, self.symbols, self.residency)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions
            .demand_region_head_with_symbols(execution, id, self.symbols, self.residency)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.regions.partition_ids_iter().collect()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.regions.partition_ids_iter())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        crate::storage::overlay::collect_partition_ids(
            self.regions.len(),
            self.regions.partition_ids_iter(),
            memory,
        )
    }
}
