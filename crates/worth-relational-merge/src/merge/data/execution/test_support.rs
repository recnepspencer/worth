use super::{
    BoundExecutableMergePlan, ExecutionReadyLoweredMergePlan, PreparedMergeExecution,
    PreparedMergeMutationPlan,
};
use crate::contracts::merge::data::execution::MergeExecutionAuthorityBinding;

#[doc(hidden)]
pub trait PreparedMergeExecutionFixturePort {
    fn execution_ready_plan_mut_for_test(&mut self) -> &mut ExecutionReadyLoweredMergePlan;

    fn authority_binding_mut_for_test(&mut self) -> &mut MergeExecutionAuthorityBinding;

    fn bound_executable_plan_mut_for_test(&mut self) -> &mut BoundExecutableMergePlan;

    fn replace_bound_plan_and_mutation_plan_for_test(
        &mut self,
        bound_executable_plan: BoundExecutableMergePlan,
        mutation_plan: PreparedMergeMutationPlan,
    );

    fn bind_mutation_plan_for_test(
        &self,
        transaction_id: crate::contracts::transactions::data::primitives::TransactionId,
    ) -> crate::contracts::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan;
}

impl PreparedMergeExecutionFixturePort for PreparedMergeExecution {
    fn execution_ready_plan_mut_for_test(&mut self) -> &mut ExecutionReadyLoweredMergePlan {
        &mut self.compiled.execution_ready_plan
    }

    fn authority_binding_mut_for_test(&mut self) -> &mut MergeExecutionAuthorityBinding {
        &mut self.compiled.bound_executable_plan.authority_binding
    }

    fn bound_executable_plan_mut_for_test(&mut self) -> &mut BoundExecutableMergePlan {
        &mut self.compiled.bound_executable_plan
    }

    fn replace_bound_plan_and_mutation_plan_for_test(
        &mut self,
        bound_executable_plan: BoundExecutableMergePlan,
        mutation_plan: PreparedMergeMutationPlan,
    ) {
        self.compiled.bound_executable_plan = bound_executable_plan;
        self.mutation_plan = mutation_plan;
    }

    fn bind_mutation_plan_for_test(
        &self,
        transaction_id: crate::contracts::transactions::data::primitives::TransactionId,
    ) -> crate::contracts::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan
    {
        self.mutation_plan.bind_transaction(transaction_id)
    }
}
