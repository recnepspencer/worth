use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

impl<Schema, Program, Operation> WorthQueryAdmittedProgramOperation<'_, Schema, Program, Operation>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    #[doc(hidden)]
    pub fn compare_and_commit_elevation_request_in_advancement<Input, Scope>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let execution =
            match phase.execution_request_for(&self.runtime.backing.runtime.product_runtime) {
                Ok(execution) => execution,
                Err(cause) => {
                    return self
                        .runtime
                        .backing
                        .runtime
                        .refuse_elevation_request(program, cause.into())
                }
            };
        self.compare_and_commit_elevation_request_in_advancement_with_request(
            &execution,
            phase,
            program,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn compare_and_commit_elevation_request_in_advancement_with_request<
        Input,
        Scope,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .backing
            .runtime
            .compare_and_commit_elevation_request_for_program(
                execution,
                phase,
                program,
                idempotency,
            )
    }
    #[doc(hidden)]
    pub fn compare_and_commit_elevation_approval_in_advancement<Input, Scope>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let execution =
            match phase.execution_request_for(&self.runtime.backing.runtime.product_runtime) {
                Ok(execution) => execution,
                Err(cause) => {
                    return self
                        .runtime
                        .backing
                        .runtime
                        .refuse_elevation_approval(program, cause.into())
                }
            };
        self.compare_and_commit_elevation_approval_in_advancement_with_request(
            &execution,
            phase,
            program,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn compare_and_commit_elevation_approval_in_advancement_with_request<
        Input,
        Scope,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .backing
            .runtime
            .compare_and_commit_elevation_approval_for_program(
                execution,
                phase,
                program,
                idempotency,
                None,
            )
    }
    #[doc(hidden)]
    pub fn compare_and_commit_elevation_close_in_advancement<Input, Scope>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationCloseOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let execution =
            match phase.execution_request_for(&self.runtime.backing.runtime.product_runtime) {
                Ok(execution) => execution,
                Err(cause) => {
                    return self
                        .runtime
                        .backing
                        .runtime
                        .refuse_elevation_close(program, cause.into())
                }
            };
        self.compare_and_commit_elevation_close_in_advancement_with_request(
            &execution,
            phase,
            program,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn compare_and_commit_elevation_close_in_advancement_with_request<
        Input,
        Scope,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationCloseOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .backing
            .runtime
            .compare_and_commit_elevation_close_for_program(
                execution,
                phase,
                program,
                idempotency,
                None,
            )
    }
}
