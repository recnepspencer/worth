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
        self.runtime
            .runtime
            .compare_and_commit_elevation_request_for_program(phase, program, idempotency)
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
        self.runtime
            .runtime
            .compare_and_commit_elevation_approval_for_program(phase, program, idempotency, None)
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
        self.runtime
            .runtime
            .compare_and_commit_elevation_close_for_program(phase, program, idempotency, None)
    }
}
