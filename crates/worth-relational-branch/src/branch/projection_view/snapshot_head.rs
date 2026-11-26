//! A point read borrows only the selected partition's Head custody, admitted
//! into the reading operation's residency.
use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::residency::{
    Head, OperationResidency, RelationalRegionHeadView, RelationalRegionRetention,
    RelationalResidencyDemandFailure, RelationalResidencyLease,
};
use worth_execution::ExecutionRequest;

pub struct SelectedSnapshotHead<'selected> {
    partition: PartitionId,
    retention: Option<&'selected RelationalRegionRetention>,
    residency: &'selected OperationResidency,
}

impl<'selected> SelectedSnapshotHead<'selected> {
    pub(in crate::internal_port::branch) fn new(
        partition: PartitionId,
        retention: Option<&'selected RelationalRegionRetention>,
        residency: &'selected OperationResidency,
    ) -> Self {
        Self {
            partition,
            retention,
            residency,
        }
    }

    pub fn view(&self) -> Option<RelationalRegionHeadView<'selected>> {
        self.retention?.accepted_head_view(self.residency)
    }

    pub fn witness(&self) -> Result<Option<RelationalResidencyLease<Head>>, PartitionId> {
        self.retention
            .map(|retention| {
                retention
                    .accepted_head_lease(self.residency)
                    .ok_or(self.partition)
            })
            .transpose()
    }

    pub(in crate::internal_port::branch) fn demand_lease(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<RelationalResidencyLease<Head>>, RelationalResidencyDemandFailure> {
        self.retention
            .map(|retention| retention.demand_head(self.residency, execution, 1 << 30))
            .transpose()
    }

    pub fn demand_with_symbols(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        if let Some(lease) = self.demand_lease(execution)? {
            lease.admit_symbols(execution, symbols)?;
        }
        Ok(())
    }
}
