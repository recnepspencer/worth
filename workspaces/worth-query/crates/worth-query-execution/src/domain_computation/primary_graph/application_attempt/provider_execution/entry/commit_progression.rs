use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn compare_and_commit_application_inner_with_currentness_and_aftermath<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let running = match self.prepare_managed_application_commit(
            execution,
            phase,
            program,
            idempotency,
            elevation_currentness,
            aftermath_causality,
        ) {
            Ok(running) => running,
            Err(outcome) => return outcome,
        };
        finish_application_commit(
            execution,
            phase,
            self,
            progress_application_commit(execution, phase, self, running, allocation_policy),
        )
    }
}
