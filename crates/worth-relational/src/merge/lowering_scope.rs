use super::MergeAccess;
use crate::merge::data::{LoweredMergePlan, MergePlanningError};
impl<'runtime> MergeAccess<'runtime> {
    pub(in crate::merge) fn lower_planning_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &crate::merge::request_normalization::BoundMergeInputs,
    ) -> Result<LoweredMergePlan, MergePlanningError> {
        let policy_plan = self.plan_policy_scope(execution, request)?;
        super::lowering::lower_policy_plan(
            &self.runtime.schema_contract_runtime.aspect_contract_plans,
            policy_plan,
        )
    }
}
