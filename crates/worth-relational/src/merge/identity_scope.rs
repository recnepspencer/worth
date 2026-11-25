use super::MergeAccess;
use crate::merge::data::{HistoryScopedMergePlan, IdentityScopedMergePlan, MergePlanningError};
impl<'runtime> MergeAccess<'runtime> {
    pub(in crate::merge) fn plan_identity_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &crate::merge::request_normalization::BoundMergeInputs,
    ) -> Result<IdentityScopedMergePlan, MergePlanningError> {
        let history_plan = self.plan_history_scope(execution, request)?;
        self.discover_identity_scope(execution, history_plan)
    }

    fn discover_identity_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history_plan: HistoryScopedMergePlan,
    ) -> Result<IdentityScopedMergePlan, MergePlanningError> {
        let target_view = self
            .runtime
            .read_truth()
            .read_version(execution, history_plan.basis.target_head().version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;
        let source_view = self
            .runtime
            .read_truth()
            .read_version(execution, history_plan.basis.source_head().version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;

        let (plan, target_rows, indexed_rows) = super::identity::discover_identity_scope(
            &self.runtime.config.schema.registry,
            &self.runtime.schema_contract_runtime.aspect_contract_plans,
            &target_view,
            &source_view,
            history_plan,
        );
        self.runtime
            .performance_access()
            .count_merge_identity_target_indexing(target_rows, indexed_rows);
        Ok(plan)
    }
}
