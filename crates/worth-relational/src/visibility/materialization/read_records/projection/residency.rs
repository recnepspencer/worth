//! Projection operation demands preserve the selected root and runtime dictionary.
use super::VisibilityProjectionView;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::PartitionId;
impl VisibilityProjectionView {
    pub(crate) fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        if let Some(root) = self.selected_root() {
            root.demand_all_full(execution, &self.symbols)?;
        }
        Ok(())
    }
    pub(crate) fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        if let Some(root) = self.selected_root() {
            root.demand_regions_full(execution, &self.symbols, partitions)?;
        }
        Ok(())
    }
}
