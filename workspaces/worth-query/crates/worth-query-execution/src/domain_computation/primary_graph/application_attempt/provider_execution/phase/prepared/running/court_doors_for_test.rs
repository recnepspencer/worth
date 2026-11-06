//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn lease_for_test(&self) -> &WorthQueryApplicationSnapshotLease {
        &self.lease
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn mutation_run_for_test(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryMutationRunBinding {
        &self.mutation_run
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn running_for_test(&self) -> &crate::domain_computation::WorthQueryRunningDirectRun {
        &self.running
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn running_mut_for_test(
        &mut self,
    ) -> &mut crate::domain_computation::WorthQueryRunningDirectRun {
        &mut self.running
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn mutation_run_into_for_test(
        self,
    ) -> crate::domain_computation::provider_session::WorthQueryMutationRunBinding {
        self.mutation_run
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn running_into_for_test(self) -> crate::domain_computation::WorthQueryRunningDirectRun {
        self.running
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn lease_into_for_test(self) -> WorthQueryApplicationSnapshotLease {
        self.lease
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn into_parts_for_test(self) -> (crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >, WorthQueryApplicationSnapshotLease, WorthQueryPreparedApplicationProviderAttempt, crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity, Option<LocalWorkflowSettlementPublication>, crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization, WorthQueryApplicationIdempotencyBinding, crate::domain_computation::WorthQueryRunningDirectRun, crate::domain_computation::provider_session::WorthQueryMutationRunBinding, WorthQueryApplicationAttemptBasis, Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,super::completion_storage::ApplicationCommitCompletionStorage,){
        (
            self.admission,
            self.lease,
            self.provider_attempt,
            self.outcome_identity,
            self.workflow_settlement_publication,
            self.authorization,
            self.idempotency,
            self.running,
            self.mutation_run,
            self.attempt_basis,
            self.aftermath_causality,
            self.completion_storage,
        )
    }
}
