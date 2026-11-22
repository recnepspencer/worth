use super::super::super::finish_application_commit;
use super::super::running::progression::progress_application_commit_with_outer;
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
    pub(crate) fn compare_and_commit_application_inner_with_currentness_and_aftermath(
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
        self.compare_and_commit_application_inner_under_coordination(
            execution,
            idempotency,
            elevation_currentness,
            aftermath_causality,
            history,
            allocation_policy,
            None,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compare_and_commit_application_inner_under_coordination(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
        history: Option<HistoryProducerInvocation<'_>>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        coordination: Option<&crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>>,
        outer_application: Option<&(dyn std::any::Any + Send + Sync)>,
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
            coordination,
        ) {
            Ok(running) => running,
            Err(outcome) => return outcome,
        };
        finish_application_commit(
            execution,
            phase,
            application,
            progress_application_commit_with_outer(
                execution,
                phase,
                application,
                outer_application,
                running.consume(),
                history,
                allocation_policy,
                coordination,
            ),
        )
    }
}
