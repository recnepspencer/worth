use super::*;
use crate::identity::data::VersionId;

impl super::ProjectionRecordSource for VisibilityProjectionView {
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.demand_all_full(execution)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.demand_regions_full(execution, partitions)
    }

    fn selected_root(&self) -> Option<&std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.selected_root()
    }
    fn is_exact_basis(&self) -> bool {
        self.is_exact_basis()
    }
    fn version_id(&self) -> VersionId {
        self.version_id()
    }
}
