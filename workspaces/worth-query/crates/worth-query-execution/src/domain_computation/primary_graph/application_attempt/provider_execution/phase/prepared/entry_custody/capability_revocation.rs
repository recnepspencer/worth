use super::*;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_capability_revocation_for_program(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;
        CurrentCommitProgram {
            application,
            program: program.into_inner(),
            phase,
        }
        .compare_and_commit_application_inner(
            execution,
            idempotency,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }
}
