use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn compare_and_commit_application<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program(program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_application(idempotency, allocation_policy)
    }

    #[doc(hidden)]
    pub fn compare_and_commit_application_in_advancement<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_application_in_advancement(idempotency, allocation_policy)
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_in_advancement_with_history<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        history: Option<HistoryProducerInvocation<'_>>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_application_in_advancement_with_history(
            idempotency,
            history,
            allocation_policy,
        )
    }
}
