use self::session_admission::admit_provider_session;
use super::WorthQueryRunningApplicationCommit;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryApplicationAttemptBasis;
use {crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt, crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease, crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding};
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::provider::{
    WorthQueryApplicationBranchCommitCoordination, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;

mod authorized;
mod commit_resolution;
mod fresh;
mod invariant;
pub(crate) mod mutation_cleanup;
pub(crate) mod progressed;
pub(crate) mod registered;
pub(crate) mod session_admission;

pub(in crate::domain_computation::primary_graph::application_attempt) use authorized::WorthQueryManagedEquivalentCommitReceiptPermit;
pub(in crate::domain_computation::primary_graph::application_attempt) use commit_resolution::WorthQueryFreshCommitReceiptPermit;
pub(in crate::domain_computation::primary_graph::application_attempt) use fresh::WorthQueryRegisteredEquivalentCommitReceiptPermit;
use mutation_cleanup::WorthQueryApplicationMutationCleanupOwner;
pub(in crate::domain_computation) use registered::{
    WorthQueryProviderAttemptRegistrationContext, WorthQueryRegisteredProviderAttempt,
};

pub struct WorthQueryProgressedApplicationCommit {
    outcome: WorthQueryProviderProgressionOutcome,
    workflow_settlement_publication: Option<super::LocalWorkflowSettlementPublication>,
    lease: WorthQueryApplicationSnapshotLease,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    cleanup: WorthQueryApplicationMutationCleanupOwner,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

struct WorthQueryProviderProgression<'a, Schema, Operation, Input, Scope> {
    application: &'a dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    running: &'a mut crate::domain_computation::WorthQueryRunningDirectRun,
    graph: &'a worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    prepared: WorthQueryPreparedApplicationProviderAttempt,
    outcome_identity: crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome_identity::WorthQueryApplicationCommitOutcomeIdentity,
    attempt_basis: WorthQueryApplicationAttemptBasis,
    authorization: crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

pub struct WorthQueryProviderProgressionCompletion {
    outcome: WorthQueryProviderProgressionOutcome,
    cleanup: WorthQueryApplicationMutationCleanupOwner,
}

impl WorthQueryProviderProgressionCompletion {
    fn finish(
        self,
        lease: WorthQueryApplicationSnapshotLease,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        workflow_settlement_publication: Option<super::LocalWorkflowSettlementPublication>,
        request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> WorthQueryProgressedApplicationCommit {
        WorthQueryProgressedApplicationCommit {
            outcome: self.outcome,
            workflow_settlement_publication,
            lease,
            running,
            cleanup: self.cleanup,
            request,
        }
    }
}

pub struct WorthQueryApplicationCommitProgressionAuthority<
    'a,
    'provider,
    Schema,
    Operation,
    Input,
    Scope,
> {
    application: &'a dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    authorization:
        crate::domain_computation::authorization::WorthQueryRegisteredCommitAuthorization,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    coordination: &'a WorthQueryApplicationBranchCommitCoordination<'provider>,
    product: &'a crate::basis::WorthQueryProductBranchLease,
    history: Option<HistoryProducerInvocation<'a>>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

impl<'a, 'provider, Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitProgressionAuthority<'a, 'provider, Schema, Operation, Input, Scope>
{
    pub(super) fn application(
        &self,
    ) -> &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    > {
        self.application.application_runtime()
    }

    pub(super) fn provider(&self) -> &'a std::sync::Arc<WorthQueryPrimaryGraphProvider> {
        self.provider
    }

    pub(super) fn admission(
        &self,
    ) -> &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    > {
        self.admission
    }

    pub(super) fn idempotency(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    pub(super) fn coordination(
        &self,
    ) -> &'a WorthQueryApplicationBranchCommitCoordination<'provider> {
        self.coordination
    }

    pub(super) fn aftermath_causality(
        &self,
    ) -> Option<
        &'_ crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.aftermath_causality.as_ref()
    }
}

pub fn progress_application_commit<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    application: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    running_commit: WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>,
    history: Option<HistoryProducerInvocation<'_>>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProgressedApplicationCommit
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let WorthQueryRunningApplicationCommit {
        admission,
        lease,
        provider_attempt,
        outcome_identity,
        workflow_settlement_publication,
        authorization,
        idempotency,
        mut running,
        mutation_run,
        attempt_basis,
        aftermath_causality,
    } = running_commit;
    progress_provider_application(
        execution,
        phase,
        WorthQueryProviderProgression {
            application,
            running: &mut running,
            graph: application.primary_graph_authority(),
            provider: application.primary_provider(),
            admission: &admission,
            prepared: provider_attempt,
            outcome_identity,
            attempt_basis,
            authorization,
            idempotency,
            aftermath_causality,
        },
        mutation_run,
        history,
        allocation_policy,
    )
    .finish(
        lease,
        running,
        workflow_settlement_publication,
        admission.publication_request().clone(),
    )
}

fn progress_provider_application<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    progression: WorthQueryProviderProgression<'_, Schema, Operation, Input, Scope>,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
    history: Option<HistoryProducerInvocation<'_>>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProviderProgressionCompletion
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let WorthQueryProviderProgression {
        application,
        running,
        graph,
        provider,
        admission,
        prepared,
        outcome_identity,
        attempt_basis,
        mut authorization,
        idempotency,
        aftermath_causality,
    } = progression;
    let product = attempt_basis.retained_product();
    let admitted_session = match admit_provider_session(
        phase,
        running,
        graph,
        product.retained_clone(),
        mutation_run,
    ) {
        Ok(admitted) => admitted,
        Err(failure) => return failure.into_completion(),
    };
    let registered_session = match admitted_session.register(
        &mut authorization,
        prepared,
        attempt_basis,
        WorthQueryProviderAttemptRegistrationContext::new(
            provider,
            admission,
            idempotency,
            outcome_identity,
            aftermath_causality.as_ref(),
        ),
        allocation_policy,
    ) {
        Ok(registered) => registered,
        Err(failure) => return failure.into_completion(),
    };
    let Some(authorization) = authorization.finish_registration() else {
        return registered_session.deny(
            crate::domain_computation::primary_graph::application_attempt::provider_execution::progression_denied(
                crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            ),
        );
    };
    provider.after_application_attempt_registration_for_test();
    let command_lane = application
        .primary_provider()
        .correction_commit_lane(&idempotency);
    let _command = command_lane.as_ref().map(|lane| lane.enter());
    let commit_lane = application
        .primary_provider()
        .application_branch_commit_lane(product.observation());

    let coordination = commit_lane.enter();
    let authority = WorthQueryApplicationCommitProgressionAuthority {
        application,
        provider,
        admission,
        authorization,
        idempotency,
        coordination: &coordination,
        product: &product,
        history,
        aftermath_causality,
    };
    registered_session.progress(execution, &authority, allocation_policy)
}

mod court_doors_for_test;
