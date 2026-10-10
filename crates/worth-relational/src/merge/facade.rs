use crate::merge::MergeAccess;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::MergeExecutionOutcome;

impl RelationalRuntime {
    pub(crate) fn merge_access(&self) -> MergeAccess<'_> {
        MergeAccess::new(self)
    }

    #[cfg(test)]
    pub fn prepare_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: crate::merge::data::MergeExecutionRequest,
    ) -> Result<
        crate::merge::data::PreparedMergeExecution,
        crate::merge::data::MergeExecutionPreparationError,
    > {
        self.merge().prepare_merge_execution(execution, request)
    }

    #[cfg(not(test))]
    pub fn prepare_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: crate::merge::data::OwnerBoundMergeExecutionRequest,
    ) -> Result<
        crate::merge::data::PreparedMergeExecution,
        crate::merge::data::MergeExecutionPreparationError,
    > {
        self.merge().prepare_merge_execution(execution, request)
    }

    pub fn execute_prepared_merge(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: crate::merge::data::PreparedMergeExecution,
    ) -> Result<MergeExecutionOutcome, crate::merge::data::MergeExecutionError> {
        crate::merge::execution::execute_prepared_merge(execution, self, prepared)
    }
}
