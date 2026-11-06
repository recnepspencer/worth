//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderProgressionCompletion {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        lease: WorthQueryApplicationSnapshotLease,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        workflow_settlement_publication: Option<super::super::LocalWorkflowSettlementPublication>,
        request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> WorthQueryProgressedApplicationCommit {
        Self::finish(
            self,
            lease,
            running,
            workflow_settlement_publication,
            request,
        )
    }
}

impl<'a, 'provider, Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitProgressionAuthority<'a, 'provider, Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn new_for_test(
        application: &'a dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
        authorization: crate::domain_computation::authorization::WorthQueryRegisteredCommitAuthorization,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        coordination: &'a WorthQueryApplicationBranchCommitCoordination<'provider>,
        product: &'a crate::basis::WorthQueryProductBranchLease,
        aftermath_causality: Option<
            crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
        history: Option<&'a dyn HistoryProducerCommit>,
    ) -> Self {
        Self {
            application,
            provider,
            admission,
            authorization,
            idempotency,
            coordination,
            product,
            aftermath_causality,
            history,
        }
    }
}

impl WorthQueryProgressedApplicationCommit {
    #[doc(hidden)]
    pub fn new_for_test(
        workflow_settlement_publication: Option<super::super::LocalWorkflowSettlementPublication>,
        outcome: WorthQueryProviderProgressionOutcome,
        lease: WorthQueryApplicationSnapshotLease,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        cleanup: WorthQueryApplicationMutationCleanupOwner,
        request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Self {
        Self {
            workflow_settlement_publication,
            outcome,
            lease,
            running,
            cleanup,
            request,
        }
    }
}

impl WorthQueryProviderProgressionCompletion {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        WorthQueryProviderProgressionOutcome,
        WorthQueryApplicationMutationCleanupOwner,
    ) {
        (self.outcome, self.cleanup)
    }
}
