use super::PreparedMergeExecution;

impl PreparedMergeExecution {
    pub(crate) fn bind_mutation_plan_for_test(
        &self,
        transaction_id: crate::transactions::data::TransactionId,
    ) -> crate::transactions::data::MergeCommitMutationPlan {
        self.mutation_plan.bind_transaction(transaction_id)
    }
}
