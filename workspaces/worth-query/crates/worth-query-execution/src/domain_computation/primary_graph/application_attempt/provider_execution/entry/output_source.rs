use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn compare_and_commit_application_for_required_output_source<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        presented: &WorthQueryPresentedProgram<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) = self.require_program_incarnation(&program) {
            return outcome;
        }
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_application_for_required_output_source(
            execution,
            presented,
            idempotency,
            causality,
            allocation_policy,
        )
    }

    pub fn compare_and_commit_application_for_program_output_producer<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        selected_program: Option<(
            &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        )>,
        history: Option<HistoryProducerInvocation<'_>>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) = self.require_program_incarnation(&program) {
            return outcome;
        }
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_application_for_program_output_producer(
            execution,
            idempotency,
            selected_program,
            history,
        )
    }

    pub fn compare_and_commit_application_for_program_action<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        presented: &WorthQueryPresentedProgram,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) = self.require_program_incarnation(&program) {
            return outcome;
        }
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_application_for_program_action(
            execution,
            presented,
            idempotency,
            causality,
            allocation_policy,
        )
    }
}
