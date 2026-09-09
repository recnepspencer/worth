use self::session_admission::admit_provider_session;
use super::WorthQueryRunningApplicationCommit;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryApplicationAttemptBasis;
use crate::domain_computation::primary_graph::application_attempt::{
    provider_binding::WorthQueryPreparedApplicationProviderAttempt,
    snapshot_lease::WorthQueryApplicationSnapshotLease, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryApplicationCommitSerialization, WorthQueryPrimaryGraphProvider,
};

mod authorized;
mod commit_resolution;
mod fresh;
mod invariant;
mod mutation_cleanup;
pub(in crate::domain_computation::primary_graph::application_attempt) mod progressed;
mod registered;
mod session_admission;

pub(in crate::domain_computation::primary_graph::application_attempt) use authorized::WorthQueryManagedEquivalentCommitReceiptPermit;
pub(in crate::domain_computation::primary_graph::application_attempt) use commit_resolution::WorthQueryFreshCommitReceiptPermit;
pub(in crate::domain_computation::primary_graph::application_attempt) use fresh::WorthQueryRegisteredEquivalentCommitReceiptPermit;
use mutation_cleanup::WorthQueryApplicationMutationCleanupOwner;
pub(in crate::domain_computation) use registered::{
    WorthQueryProviderAttemptRegistrationContext, WorthQueryRegisteredProviderAttempt,
};

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) struct WorthQueryProgressedApplicationCommit
{
    outcome: WorthQueryProviderProgressionOutcome,
    lease: WorthQueryApplicationSnapshotLease,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    cleanup: WorthQueryApplicationMutationCleanupOwner,
}

struct WorthQueryProviderProgression<'a, 'provider, Schema, Operation, Input, Scope> {
    application:
        &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
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
    attempt_basis: WorthQueryApplicationAttemptBasis,
    authorization: crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    serialization: &'a WorthQueryApplicationCommitSerialization<'provider>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

struct WorthQueryProviderProgressionCompletion {
    outcome: WorthQueryProviderProgressionOutcome,
    cleanup: WorthQueryApplicationMutationCleanupOwner,
}

impl WorthQueryProviderProgressionCompletion {
    fn finish(
        self,
        lease: WorthQueryApplicationSnapshotLease,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
    ) -> WorthQueryProgressedApplicationCommit {
        WorthQueryProgressedApplicationCommit {
            outcome: self.outcome,
            lease,
            running,
            cleanup: self.cleanup,
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution::phase)
struct WorthQueryApplicationCommitProgressionAuthority<
    'a,
    'provider,
    Schema,
    Operation,
    Input,
    Scope,
> {
    application:
        &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
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
    serialization: &'a WorthQueryApplicationCommitSerialization<'provider>,
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
        self.application
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

    pub(super) fn serialization(&self) -> &'a WorthQueryApplicationCommitSerialization<'provider> {
        self.serialization
    }

    pub(super) fn aftermath_causality(
        &self,
    ) -> Option<
        &'_ crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.aftermath_causality.as_ref()
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn progress_application_commit<
    Schema,
    Operation,
    Input,
    Scope,
>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    running_commit: WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>,
) -> WorthQueryProgressedApplicationCommit
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let WorthQueryRunningApplicationCommit {
        admission,
        lease,
        provider_attempt,
        authorization,
        idempotency,
        mut running,
        mutation_run,
        attempt_basis,
        aftermath_causality,
    } = running_commit;
    let serialization = application.primary_provider.serialize_application_commit();
    progress_provider_application(
        WorthQueryProviderProgression {
            application,
            running: &mut running,
            graph: &application.primary_graph_authority,
            provider: &application.primary_provider,
            admission: &admission,
            prepared: provider_attempt,
            attempt_basis,
            authorization,
            idempotency,
            serialization: &serialization,
            aftermath_causality,
        },
        mutation_run,
    )
    .finish(lease, running)
}

fn progress_provider_application<Schema, Operation, Input, Scope>(
    progression: WorthQueryProviderProgression<'_, '_, Schema, Operation, Input, Scope>,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
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
        attempt_basis,
        mut authorization,
        idempotency,
        serialization,
        aftermath_causality,
    } = progression;
    let product = attempt_basis.retained_product();
    let admitted_session = match admit_provider_session(running, graph, product, mutation_run) {
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
            aftermath_causality.as_ref(),
        ),
    ) {
        Ok(registered) => registered,
        Err(failure) => return failure.into_completion(),
    };
    let Some(authorization) = authorization.finish_registration() else {
        return registered_session.deny(
            crate::domain_computation::primary_graph::application_attempt::provider_execution::progression_denied(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            ),
        );
    };
    let authority = WorthQueryApplicationCommitProgressionAuthority {
        application,
        provider,
        admission,
        authorization,
        idempotency,
        serialization,
        aftermath_causality,
    };
    registered_session.progress(&authority)
}
