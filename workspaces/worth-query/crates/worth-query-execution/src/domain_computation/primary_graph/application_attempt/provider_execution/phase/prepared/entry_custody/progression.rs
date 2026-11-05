use super::super::super::{finish_application_commit, progress_application_commit};
use super::*;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(super) fn compare_and_commit_application_inner_with_currentness_and_aftermath(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
        history: Option<HistoryProducerInvocation<'_>>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let application = self.application;
        let phase = self.phase;
        let running = match self.prepare_managed_application_commit(
            execution,
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
            application,
            progress_application_commit(
                execution,
                phase,
                application,
                running,
                history,
                allocation_policy,
            ),
        )
    }
}
