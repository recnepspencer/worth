use super::MergeAccess;
use crate::merge::data::{CausallyAnnotatedMergePlan, MergePlanningError};
impl<'runtime> MergeAccess<'runtime> {
    pub(in crate::merge) fn plan_causal_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &crate::merge::request_normalization::BoundMergeInputs,
    ) -> Result<CausallyAnnotatedMergePlan, MergePlanningError> {
        let conflict_plan = self.plan_conflict_scope(execution, request)?;
        let history = self.runtime.history();
        Ok(super::causal::annotate_causal_scope(
            conflict_plan,
            &|commit| history.ancestor_closure_by_commit_id_order(commit),
        ))
    }
}
