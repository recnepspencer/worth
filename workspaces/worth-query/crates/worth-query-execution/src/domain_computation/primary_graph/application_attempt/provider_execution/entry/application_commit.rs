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
        if let Err(outcome) = self.validate_commit_program(None, &program) {
            return outcome;
        }
        let request = program.read_set.admission.publication_request().clone();
        self.with_funded_application_advancement(&request, allocation_policy, |phase| {
            self.compare_and_commit_application_in_advancement(
                &phase,
                program,
                idempotency,
                allocation_policy,
            )
        })
        .unwrap_or_else(|denial| denial.into_commit_outcome())
    }

    #[doc(hidden)]
    pub fn compare_and_commit_application_in_advancement<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) = self.validate_commit_program(Some(phase), &program) {
            return outcome;
        }
        self.compare_and_commit_application_in_advancement_with_history(
            phase,
            program,
            idempotency,
            None,
            allocation_policy,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_in_advancement_with_history<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        history: Option<HistoryProducerInvocation<'_>>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) = self.validate_commit_program(Some(phase), &program) {
            return outcome;
        }
        let execution = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
            }
        };
        if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        self.compare_and_commit_application_with_output_observation(
            &execution,
            phase,
            program,
            idempotency,
            false,
            history,
            allocation_policy,
        )
    }
}
