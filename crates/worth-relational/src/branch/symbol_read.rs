//! Bind shared branch cells to the reading runtime's dictionary authority.
use super::RelationalBranchRoot;
use crate::identity::data::PartitionId;
use crate::runtime::RuntimeSymbolTable;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess, PartitionState};
use crate::storage::residency::{
    PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView, RelationalResidencyDemandFailure,
};

pub(crate) struct SymbolAdmittedRootRead<'read> {
    root: &'read RelationalBranchRoot,
    symbols: &'read RuntimeSymbolTable,
}

impl<'read> SymbolAdmittedRootRead<'read> {
    pub(crate) fn new(
        root: &'read RelationalBranchRoot,
        symbols: &'read RuntimeSymbolTable,
    ) -> Self {
        Self { root, symbols }
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
        self.root.probe_full_region_with_symbols(id, self.symbols)
    }
    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.root.demand_regions_full(execution, self.symbols, [id])
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
        self.root.probe_head_region_with_symbols(id, self.symbols)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.root
            .demand_region_head_with_symbols(execution, id, self.symbols)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.root.partition_ids()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.root.partition_ids_iter())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        self.root.partition_ids_with_memory(memory)
    }
    fn has_read_misses(&self) -> bool {
        self.root.has_read_misses()
    }
}
