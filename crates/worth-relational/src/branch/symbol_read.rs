//! Bind shared branch cells to the reading runtime's dictionary authority.
use super::SelectedRelationalBranchState;
use crate::identity::data::PartitionId;
use crate::runtime::RuntimeSymbolTable;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess, PartitionState};
use crate::storage::residency::{
    PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView, RelationalResidencyDemandFailure,
};

pub(crate) struct SymbolAdmittedBranchRead<'read> {
    selected: &'read SelectedRelationalBranchState,
    symbols: &'read RuntimeSymbolTable,
}

impl<'read> SymbolAdmittedBranchRead<'read> {
    pub(super) fn new(
        selected: &'read SelectedRelationalBranchState,
        symbols: &'read RuntimeSymbolTable,
    ) -> Self {
        Self { selected, symbols }
    }
}

impl PartitionAccess for SymbolAdmittedBranchRead<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.selected
            .state()
            .root()
            .probe_full_region_with_symbols(id, self.symbols)
    }
    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.selected
            .state()
            .root()
            .demand_regions_full(execution, self.symbols, [id])
    }
}

impl CurrentPartitionAccess for SymbolAdmittedBranchRead<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        match self.probe_head_partition(id) {
            PartitionHeadProbe::Resident(view) => Some(view),
            _ => None,
        }
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.selected
            .state()
            .root()
            .probe_head_region_with_symbols(id, self.symbols)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.selected
            .state()
            .root()
            .demand_region_head_with_symbols(execution, id, self.symbols)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.selected.partition_ids()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        self.selected.partition_ids_iter()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        self.selected.partition_ids_with_memory(memory)
    }
    fn has_read_misses(&self) -> bool {
        self.selected.has_read_misses()
    }
}
