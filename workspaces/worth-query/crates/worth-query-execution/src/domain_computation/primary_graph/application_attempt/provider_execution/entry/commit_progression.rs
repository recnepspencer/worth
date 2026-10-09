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
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let prepared = match prepare_application_commit(
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
            WorthQueryApplicationCommitPreparation::Terminal(outcome) => return outcome,
        };
        let running = match start_managed_application_commit(phase, self, prepared) {
            Ok(running) => running,
            Err(outcome) => return outcome,
        };
        finish_application_commit(
            phase,
            self,
            progress_application_commit(phase, self, running),
        )
    }
}
