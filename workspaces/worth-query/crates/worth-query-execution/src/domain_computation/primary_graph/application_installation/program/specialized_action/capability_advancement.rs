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
        self.runtime
            .runtime
            .compare_and_commit_mandatory_review_for_program(phase, program, idempotency, None)
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
        self.runtime
            .runtime
            .compare_and_commit_capability_delegation_for_program(phase, program, idempotency)
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
        self.runtime
            .runtime
            .compare_and_commit_capability_revocation_for_program(phase, program, idempotency)
    }
}
