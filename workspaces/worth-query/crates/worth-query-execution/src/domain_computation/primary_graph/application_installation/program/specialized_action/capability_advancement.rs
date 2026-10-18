use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

impl<Schema, Program, Operation> WorthQueryAdmittedProgramOperation<'_, Schema, Program, Operation>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Commits inside the publication call's existing runtime-owned request.
    #[doc(hidden)]
    pub fn compare_and_commit_mandatory_review_in_advancement<Input, Scope>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryMandatoryReviewOutcome
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
                        .refuse_mandatory_review(program, cause.into())
                }
            };
        self.compare_and_commit_mandatory_review_in_advancement_with_request(
            &execution,
            phase,
            program,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn compare_and_commit_mandatory_review_in_advancement_with_request<
        Input,
        Scope,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .backing
            .runtime
            .compare_and_commit_mandatory_review_for_program(
                execution,
                phase,
                program,
                idempotency,
                None,
            )
    }
    /// Commits inside the publication call's existing runtime-owned request.
    #[doc(hidden)]
    pub fn compare_and_commit_capability_delegation_in_advancement<Input, Scope>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let execution = match phase
            .execution_request_for(&self.runtime.backing.runtime.product_runtime)
        {
            Ok(execution) => execution,
            Err(cause) => {
                return crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
            }
        };
        self.compare_and_commit_capability_delegation_in_advancement_with_request(
            &execution,
            phase,
            program,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn compare_and_commit_capability_delegation_in_advancement_with_request<
        Input,
        Scope,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .backing
            .runtime
            .compare_and_commit_capability_delegation_for_program(
                execution,
                phase,
                program,
                idempotency,
            )
    }
    /// Commits inside the publication call's existing runtime-owned request.
    #[doc(hidden)]
    pub fn compare_and_commit_capability_revocation_in_advancement<Input, Scope>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let execution = match phase
            .execution_request_for(&self.runtime.backing.runtime.product_runtime)
        {
            Ok(execution) => execution,
            Err(cause) => {
                return crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
            }
        };
        self.compare_and_commit_capability_revocation_in_advancement_with_request(
            &execution,
            phase,
            program,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn compare_and_commit_capability_revocation_in_advancement_with_request<
        Input,
        Scope,
    >(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .backing
            .runtime
            .compare_and_commit_capability_revocation_for_program(
                execution,
                phase,
                program,
                idempotency,
            )
    }
}
