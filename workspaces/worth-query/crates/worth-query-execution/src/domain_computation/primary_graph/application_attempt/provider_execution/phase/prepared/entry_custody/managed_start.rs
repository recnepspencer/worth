//! Consume original entry preparation before managed progression enters.
use super::super::running::start_managed_application_commit;
use super::*;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >
{
    #[inline(never)]
    pub(super) fn prepare_managed_application_commit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> Result<
        super::super::running::completion_storage::PhaseStorage<
            super::super::running::WorthQueryRunningApplicationCommit<
                Schema,
                Operation,
                Input,
                Scope,
            >,
        >,
        WorthQueryApplicationCommitOutcome,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        let application = self.application;
        let phase = self.phase;
        let prepared = match prepare_application_commit(
            execution,
            WorthQueryApplicationCommitPreparationRequest::new(
                self,
                idempotency,
                elevation_currentness,
                aftermath_causality,
            ),
        ) {
            WorthQueryApplicationCommitPreparation::Ready(prepared) => prepared,
            WorthQueryApplicationCommitPreparation::Terminal(outcome) => return Err(outcome),
        };
        let storage = super::super::running::completion_storage::PhaseStorage::prepare(execution)?;
        let running = start_managed_application_commit(phase, application, prepared)?;
        Ok(storage.fill(running))
    }
}
