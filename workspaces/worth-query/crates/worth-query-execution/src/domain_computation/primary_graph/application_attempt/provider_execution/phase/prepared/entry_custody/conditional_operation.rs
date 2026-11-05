use super::*;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::ProviderExecutionEntryScopedOps;
use crate::domain_computation::primary_graph::application_attempt::EntryScopedOps;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_conditional_operation(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
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

        if !application.has_installed_application_program() {
            return Self {
                application,
                program,
                phase,
            }
            .compare_and_commit_application_in_advancement(
                idempotency,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        }
        if !application
            .application_runtime()
            .installed_conditionals
            .contains_operation::<Operation>()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        if let Err(outcome) = application
            .application_runtime()
            .require_occurrence_program_commit_binding(execution, &program, None)
        {
            return outcome;
        }
        Self {
            application,
            program,
            phase,
        }
        .compare_and_commit_application_with_output_observation(
            execution,
            idempotency,
            false,
            None,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }
}
