use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::{
    BoundExecutableMergePlan, ExecutionReadyLoweredMergePlan, PreparedMergeExecution,
    PreparedMergeMutationPlan,
};
use crate::merge::data::execution::MergeExecutionAuthorityBinding;

impl PreparedMergeExecution {
    pub(crate) fn execution_ready_plan_mut_for_test(
        &mut self,
    ) -> &mut ExecutionReadyLoweredMergePlan {
        &mut self.compiled.execution_ready_plan
    }

    pub(crate) fn authority_binding_mut_for_test(&mut self) -> &mut MergeExecutionAuthorityBinding {
        &mut self.compiled.bound_executable_plan.authority_binding
    }

    pub(crate) fn bound_executable_plan_mut_for_test(&mut self) -> &mut BoundExecutableMergePlan {
        &mut self.compiled.bound_executable_plan
    }

    pub(crate) fn replace_bound_plan_and_mutation_plan_for_test(
        &mut self,
        bound_executable_plan: BoundExecutableMergePlan,
        mutation_plan: PreparedMergeMutationPlan,
    ) {
        self.compiled.bound_executable_plan = bound_executable_plan;
        self.mutation_plan = mutation_plan;
    }

    pub(crate) fn bind_mutation_plan_for_test(
        &self,
        transaction_id: crate::transactions::data::primitives::TransactionId,
    ) -> crate::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan {
        self.mutation_plan.bind_transaction(transaction_id)
    }
}
