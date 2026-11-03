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
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_in_advancement(
        self,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_in_advancement_with_history(
            idempotency,
            None,
            allocation_policy,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_in_advancement_with_history(
        self,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        history: Option<HistoryProducerInvocation<'_>>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;

        let execution = match phase.execution_request_for(&application.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
            }
        };
        if let Some(denial) = application.direct_operation_commit_denial::<Operation>() {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        Self {
            application,
            program,
            phase,
        }
        .compare_and_commit_application_with_output_observation(
            &execution,
            idempotency,
            false,
            history,
            allocation_policy,
        )
    }
}
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    ReadmittedCommitProgram<
        '_,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application(
        self,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let request = self.request_scope().clone();
        let application = self.application;
        application
            .with_funded_application_advancement(&request, allocation_policy, |phase| {
                let entry = match self.in_advancement(&phase) {
                    Ok(entry) => entry,
                    Err(outcome) => return outcome,
                };
                entry.compare_and_commit_application_in_advancement(idempotency, allocation_policy)
            })
            .unwrap_or_else(|denial| denial.into_commit_outcome())
    }
}
