use super::super::super::{
    finish_application_commit, progress_application_commit, start_managed_application_commit,
};
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
            WorthQueryApplicationCommitPreparation::Terminal(outcome) => return outcome,
        };
        let running = match start_managed_application_commit(phase, application, prepared) {
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
