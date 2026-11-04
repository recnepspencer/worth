use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryParameterSet,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};

use super::{WorthQueryProductQueryControls, WorthQuerySelectedProductOperation};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub(in crate::domain_computation::primary_graph) fn retained_query_controls(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
        query_name: &str,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        (
            &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationQueryControls<'runtime, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        if self.product().branch_identity() != retained.product().branch_identity() {
            return Err(foreign_basis(query_name));
        }
        let security = self
            .application()
            .runtime
            .product_runtime
            .security_observation_for(execution, self.product())
            .map_err(|stop| super::selection_denial(stop, query_name))?;
        let (application, _current_security, _) = self.into_parts();
        let producer = retained.original_settled_producer().cloned();
        let (retained_application, product, application_basis) = retained.into_parts();
        if !std::ptr::eq(application, retained_application) {
            return Err(foreign_basis(query_name));
        }
        Ok((
            application,
            WorthQueryApplicationQueryControls::retained_product_one_shot(
                security,
                product,
                application_basis,
                controls.maximum_results,
                controls.request,
            )
            .with_original_settled_producer(producer),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit_retained_governed_application_query<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        Operation,
        Input,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<'_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
        query: &'runtime WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    >
    where
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        let request = match phase.execution_request_for(&self.application().runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, controls) =
            self.retained_query_controls(execution, retained, query.name(), controls)?;
        application.admit_governed_application_query(
            execution, query, access, capability, parameters, controls,
        )
    }
}

fn foreign_basis(query_name: &str) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
        query_name,
    )
}

mod court_doors_for_test;
