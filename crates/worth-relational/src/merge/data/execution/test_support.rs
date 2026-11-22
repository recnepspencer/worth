use super::{ExecutionReadyLoweredMergePlan, PreparedMergeExecution};

impl PreparedMergeExecution {
    pub(crate) fn execution_ready_plan_mut_for_test(
        &mut self,
    ) -> &mut ExecutionReadyLoweredMergePlan {
        &mut self.compiled.execution_ready_plan
    }

    pub(crate) fn bind_mutation_plan_for_test(
        &self,
        transaction_id: crate::transactions::data::TransactionId,
    ) -> crate::transactions::data::MergeCommitMutationPlan {
        self.mutation_plan.bind_transaction(transaction_id)
    }
}
