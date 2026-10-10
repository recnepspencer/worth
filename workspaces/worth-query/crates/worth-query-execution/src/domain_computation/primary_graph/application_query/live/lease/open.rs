mod governed;
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::{ApplicationQueryLiveCauseBinding, ApplicationQueryParameterSet},
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

mod managed_basis_admission;

use self::managed_basis_admission::admit_live_managed_basis;
use super::super::{
    controls::WorthQueryApplicationLiveControls,
    outcome::{WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind},
    scope_identity::read_scope_identity,
};
use super::validation::{
    open_admission_denial, open_denial, open_read_denial, validate_live_binding,
    validate_live_resource_controls,
};
use super::WorthQueryApplicationLiveLease;
use crate::domain_computation::primary_graph::{
    application_query::{
        admission::prepare_governed_access,
        authorized_read::{execute_authorized_read, refresh_governed_authorization},
        disclosure::WorthQueryPendingApplicationQueryGovernance,
        WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryControls,
    },
    live_delivery::WorthQueryLiveCauseQueue,
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};

struct WorthQueryApplicationLiveOpenRequest<
    'principal,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    principal: &'principal WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
    controls: WorthQueryApplicationLiveControls,
    product: crate::basis::WorthQueryProductBranchLease,
    application_basis: super::super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
    pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>,
}

struct WorthQueryApplicationLiveInitialRead {
    governance: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    scope_identity: worth_foundational::facade::AspectValue,
    graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    initial_read_work: crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork,
    basis_release: super::super::super::WorthQueryApplicationBasisReleaseReceipt,
}

impl<'runtime, Schema>
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub fn open_application_query_live<
        'principal,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        principal: &'principal WorthQueryAuthenticatedPrincipal<
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<
        WorthQueryApplicationLiveLease<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
        >,
        WorthQueryApplicationLiveOpenDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
    {
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorthQueryApplicationLiveOpenDenial::with_authorization(WorthQueryApplicationLiveOpenDenialKind::AuthorizationDenied(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::ForeignPhase), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::ForeignPhase, "phase-custody"))),
        };
        let execution = &request;

        let (application, product, application_basis) = self.into_parts();
        application.open_application_query_live_with_governance::<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
        >(execution, WorthQueryApplicationLiveOpenRequest {
            query,
            principal,
            scope,
            parameters,
            controls,
            product,
            application_basis,
            pending_governance: None,
        })
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    fn open_application_query_live_with_governance<
        'runtime,
        'principal,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >(
        &'runtime self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: WorthQueryApplicationLiveOpenRequest<
            'principal,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationLiveLease<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
        >,
        WorthQueryApplicationLiveOpenDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
    {
        let live = validate_live_binding::<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Target,
            Binding,
        >(&request.query)?;
        validate_live_resource_controls(live, &request.controls, request.query.name())?;
        let retained_product = request.product.retained_clone();
        let access =
            WorthQueryApplicationQueryAccessContext::new(request.principal, &request.scope);
        let query_controls = WorthQueryApplicationQueryControls::product_live(
            request.product.read_lease(),
            request.application_basis,
            request.controls.maximum_materialized_record_count(),
            request.controls.maximum_work_per_delivery(),
            request.controls.request(),
        );
        let (admitted_parameters, query_controls) = self
            .prepare_application_query_admission(
                &request.query,
                &access,
                request.parameters.clone(),
                query_controls,
            )
            .map_err(open_admission_denial)?;
        let plan = self
            .finish_application_query_admission(
                execution,
                &request.query,
                &access,
                admitted_parameters,
                query_controls,
                request.pending_governance,
            )
            .map_err(open_admission_denial)?;
        let initial_read = execute_live_initial_read(execution, self, plan, request.query.name())?;
        let basis =
            admit_live_managed_basis(self, live, &initial_read.graph_work, request.query.name())?;
        let buffer_capacity = request.controls.buffer_capacity();
        let queue = WorthQueryLiveCauseQueue::open(
            &self.primary_provider.live_delivery,
            retained_product.observation(),
            buffer_capacity,
        );
        Ok(WorthQueryApplicationLiveLease {
            runtime: self,
            query: request.query,
            scope: request.scope,
            parameters: request.parameters,
            controls: request.controls,
            _opening_product: retained_product,
            governance: initial_read.governance,
            scope_identity: initial_read.scope_identity,
            basis: Some(basis),
            graph_work: Some(initial_read.graph_work),
            read_proof: Some(initial_read.read_proof),
            initial_read_work: Some(initial_read.initial_read_work),
            basis_release: Some(initial_read.basis_release),
            read_completion: None,
            queue,
            _target: PhantomData,
            _thread_affinity: PhantomData,
        })
    }
}

fn execute_live_initial_read<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    mut plan: crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan<
        '_, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope,
    >,
    subject: &str,
) -> Result<WorthQueryApplicationLiveInitialRead, WorthQueryApplicationLiveOpenDenial>
where
    Schema: ApplicationSchema,
{
    refresh_governed_authorization(execution, application, &mut plan)
        .map_err(|denial| open_read_denial(denial, subject))?;
    application.runtime.primary_graph().ok_or_else(|| {
        open_denial(
            WorthQueryApplicationLiveOpenDenialKind::ScopeIdentityUnavailable,
            subject,
        )
    })?;
    let ((scope_identity, initial_read_work), _, read_proof) =
        execute_authorized_read(execution, application, &plan, |runtime, graph, plan| {
            read_scope_identity(execution, runtime, graph, plan)
        })
        .map_err(|denial| open_read_denial(denial, subject))?;
    let governance = plan.take_governance();
    let basis_release = plan.basis.release();
    if !basis_release.released() {
        return Err(open_denial(
            WorthQueryApplicationLiveOpenDenialKind::BasisReleaseFailed,
            subject,
        ));
    }
    Ok(WorthQueryApplicationLiveInitialRead {
        governance,
        scope_identity,
        graph_work: plan.graph_work,
        read_proof,
        initial_read_work,
        basis_release,
    })
}
