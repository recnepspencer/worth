//! Finish admission and managed start before provider progression enters.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[inline(never)]
    pub(super) fn prepare_managed_application_commit<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> Result<
        super::super::phase::WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>,
        WorthQueryApplicationCommitOutcome,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.require_program_incarnation(&program)?;
        let prepared = match prepare_application_commit(
            execution,
            phase,
            self,
            WorthQueryApplicationCommitPreparationRequest::new(
                program,
                idempotency,
                elevation_currentness,
                aftermath_causality,
            ),
        ) {
            WorthQueryApplicationCommitPreparation::Ready(prepared) => prepared,
            WorthQueryApplicationCommitPreparation::Terminal(outcome) => return Err(outcome),
        };
        start_managed_application_commit(phase, self, prepared)
    }
}
