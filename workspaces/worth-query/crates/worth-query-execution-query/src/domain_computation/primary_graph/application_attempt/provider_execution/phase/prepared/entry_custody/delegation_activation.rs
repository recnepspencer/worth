use super::*;
use worth_query_execution_attempt::facade::ports::SpecializedEffectProgramInternalPort as _;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
    >
{
    pub fn compare_and_commit_capability_delegation_for_program(
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
