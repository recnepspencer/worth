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
    ) -> Self {
        // A court never names the attempt-owned history commit port: the door builds an
        // authority without a history producer.
        Self {
            application,
            provider,
            admission,
            authorization,
            idempotency,
            coordination,
            product,
            aftermath_causality,
            history: None,
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

/// Sealed test door: progress a running commit that carries no history producer, so a court
/// never names the attempt-owned history commit port.
#[doc(hidden)]
pub fn progress_application_commit_without_history_for_test<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    running_commit: WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProgressedApplicationCommit
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    super::progress_application_commit(
        execution,
        phase,
        application,
        running_commit,
        None,
        allocation_policy,
    )
}
