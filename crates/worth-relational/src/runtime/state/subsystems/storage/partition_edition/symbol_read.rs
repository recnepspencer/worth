//! Bind mirror reads to the demanding runtime's dictionary authority.
use super::PartitionEdition;
use crate::identity::data::PartitionId;
use crate::runtime::RuntimeSymbolTable;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess, PartitionState};
use crate::storage::residency::{
    PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView, RelationalResidencyDemandFailure,
};

pub(super) struct SymbolAdmittedEdition<'read> {
    pub(super) edition: &'read PartitionEdition,
    pub(super) symbols: &'read RuntimeSymbolTable,
}

impl PartitionAccess for SymbolAdmittedEdition<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }

    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        match self.edition.partitions.get(&id) {
            None => PartitionProbe::Absent,
            Some(region) => match region.admitted_partition() {
                Some(partition)
                    if self
                        .symbols
                        .region_symbols_admitted(region.lease().symbol_admission_identity()) =>
                {
                    PartitionProbe::Resident(partition)
                }
                _ => PartitionProbe::Cold,
            },
        }
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.edition
            .demand_partition_with_symbols(execution, self.symbols, id)
    }
}

impl CurrentPartitionAccess for SymbolAdmittedEdition<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        match self.probe_head_partition(id) {
            PartitionHeadProbe::Resident(view) => Some(view),
            _ => None,
        }
    }

    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        match self.edition.partitions.get(&id) {
            None => PartitionHeadProbe::Absent,
            Some(region) => match region.admitted_head_view() {
                Some(view) if region.head_symbols_admitted(self.symbols) => {
                    PartitionHeadProbe::Resident(view)
                }
                _ => PartitionHeadProbe::Cold,
            },
        }
    }

    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        if let Some(lease) = self.edition.demand_partition_head(execution, id)? {
            lease.admit_symbols(execution, self.symbols)?;
        }
        Ok(())
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.edition.partition_ids()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        self.edition.partition_ids_iter()
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        self.edition.partition_ids_with_memory(memory)
    }
}
