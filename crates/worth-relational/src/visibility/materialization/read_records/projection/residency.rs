//! Projection operation demands preserve the selected root and runtime dictionary.
use super::VisibilityProjectionView;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::PartitionId;
impl VisibilityProjectionView {
    pub(super) fn admitted_head_region(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        if let Some(resident) = &self.resident {
            return resident.head_region(id);
        }
        self.selected_root()?.admitted_head_region(id)
    }
    pub(crate) fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        if self.resident.is_some() {
            return Ok(());
        }
        if let Some(root) = self.selected_root() {
            root.demand_regions_head(execution, &self.symbols, partitions)?;
        }
        Ok(())
    }

    pub(crate) fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        if self.resident.is_some() {
            return Ok(());
        }
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
        crate::execution::check_request(execution)?;
        if self.resident.is_some() {
            return Ok(());
        }
        if let Some(root) = self.selected_root() {
            root.demand_regions_full(execution, &self.symbols, partitions)?;
        }
        Ok(())
    }
}
