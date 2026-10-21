use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[allow(clippy::too_many_arguments)]
    pub fn open_governed_application_query_live<
        'principal,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
        Capability,
        Operation,
        Input,
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
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
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
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorthQueryApplicationLiveOpenDenial::with_authorization(WorthQueryApplicationLiveOpenDenialKind::AuthorizationDenied(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody"))),
        };
        let execution = &request;

        let (application, product, application_basis) = self.into_parts();
        let access = WorthQueryApplicationQueryAccessContext::new(principal, &scope);
        let query_controls = WorthQueryApplicationQueryControls::product_live(
            product.read_lease(),
            application
                .retain_product_application_basis(execution, product.observation())
                .map_err(|_| {
                    open_denial(
                        WorthQueryApplicationLiveOpenDenialKind::ProviderVersionUnavailable,
                        query.name(),
                    )
                })?,
            controls.maximum_materialized_record_count(),
            controls.request(),
        );
        let pending =
            prepare_governed_access(application, &query, &access, capability, &query_controls)
                .map_err(open_admission_denial)?;
        drop(query_controls);
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
            pending_governance: Some(pending),
        })
    }
}
