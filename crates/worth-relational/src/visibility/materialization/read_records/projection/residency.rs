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
        self.selected_root()?.accepted_head_region_view(id)
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

    pub(super) fn demand_selected_scan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        if self.is_exact_basis() {
            if let Some(root) = self.selected_root() {
                self.demand_regions_head(execution, root.partition_ids_iter())?;
            } else {
                crate::execution::check_request(execution)?;
            }
            Ok(())
        } else {
            self.demand_all_full(execution)
        }
    }
    pub(super) fn demand_selected_regions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        if self.is_exact_basis() {
            self.demand_regions_head(execution, partitions)
        } else {
            self.demand_regions_full(execution, partitions)
        }
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
