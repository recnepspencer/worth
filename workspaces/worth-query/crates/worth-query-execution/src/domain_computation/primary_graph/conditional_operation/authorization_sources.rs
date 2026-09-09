use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryParameterSet,
    application_schema::{ApplicationSchema, TypedMutationPreconditions},
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationOperation,
    WorthQueryInstalledApplicationQuery,
};

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryControls,
    WorthQueryAuthenticatedPrincipal, WorthQueryOperationAuthorizationDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};

#[cfg(test)]
#[path = "authorization_sources/tests.rs"]
mod tests;

#[derive(Debug)]
pub enum WorthQueryTemporalQueryAuthorizationDenial {
    Query(WorthQueryApplicationQueryAdmissionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
}

impl std::fmt::Display for WorthQueryTemporalQueryAuthorizationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(denial) => denial.fmt(formatter),
            Self::Authorization(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorthQueryTemporalQueryAuthorizationDenial {}

pub trait WorthQueryTemporalQueryAuthorization<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>: Send + Sync + 'static where
    Schema: ApplicationSchema,
{
    fn admit<'a>(
        &self,
        runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryTemporalQueryAuthorizationDenial,
    >;
}

#[derive(Default)]
pub struct WorthQueryPublicTemporalQueryAuthorization;

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryTemporalQueryAuthorization<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    > for WorthQueryPublicTemporalQueryAuthorization
where
    Schema: ApplicationSchema,
{
    fn admit<'a>(
        &self,
        runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryTemporalQueryAuthorizationDenial,
    > {
        runtime
            .admit_application_query(query, access, parameters, controls)
            .map_err(WorthQueryTemporalQueryAuthorizationDenial::Query)
    }
}

pub struct WorthQueryGovernedTemporalQueryAuthorization<
    Schema,
    Capability,
    CapabilityOperation,
    CapabilityInput,
> {
    capability: WorthQueryInstalledApplicationCapability<
        Schema,
        Capability,
        CapabilityOperation,
        CapabilityInput,
    >,
    input: CapabilityInput,
}

impl<Schema, Capability, CapabilityOperation, CapabilityInput>
    WorthQueryGovernedTemporalQueryAuthorization<
        Schema,
        Capability,
        CapabilityOperation,
        CapabilityInput,
    >
{
    pub fn new(
        capability: WorthQueryInstalledApplicationCapability<
            Schema,
            Capability,
            CapabilityOperation,
            CapabilityInput,
        >,
        input: CapabilityInput,
    ) -> Self {
        Self { capability, input }
    }
}

impl<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        CapabilityOperation,
        CapabilityInput,
    >
    WorthQueryTemporalQueryAuthorization<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
    for WorthQueryGovernedTemporalQueryAuthorization<
        Schema,
        Capability,
        CapabilityOperation,
        CapabilityInput,
    >
where
    Schema: ApplicationSchema,
    CapabilityInput: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>
        + worth_query_declaration::facade::portable_identity::WorthQueryPortableType
        + Clone
        + Send
        + Sync
        + 'static,
    Capability: Send + Sync + 'static,
    CapabilityOperation: Send + Sync + 'static,
{
    fn admit<'a>(
        &self,
        runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryTemporalQueryAuthorizationDenial,
    > {
        let product = controls.product_branch();
        let capability = crate::domain_computation::authorization::admit_capability_access(
            runtime,
            product,
            access.principal(),
            &self.capability,
            self.input.clone(),
            controls.request_scope(),
            None,
        )
        .map_err(WorthQueryTemporalQueryAuthorizationDenial::Authorization)?;
        runtime
            .admit_governed_application_query(query, access, capability, parameters, controls)
            .map_err(WorthQueryTemporalQueryAuthorizationDenial::Query)
    }
}

pub trait WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>:
    Send + Sync + 'static
where
    Schema: ApplicationSchema,
{
    fn authorize<Principal, PrincipalIdentity>(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        input: &Input,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    >;
}

#[derive(Default)]
pub struct WorthQueryPublicTemporalOperationAuthorization;

impl<Schema, Operation, Input, Scope>
    WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>
    for WorthQueryPublicTemporalOperationAuthorization
where
    Schema: ApplicationSchema,
{
    fn authorize<Principal, PrincipalIdentity>(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        _input: &Input,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        product.authorize_operation(principal, scope, operation, preconditions, request)
    }
}

pub struct WorthQueryGovernedTemporalOperationAuthorization<Schema, Capability, Operation, Input> {
    capability: WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
}

impl<Schema, Capability, Operation, Input>
    WorthQueryGovernedTemporalOperationAuthorization<Schema, Capability, Operation, Input>
{
    pub fn new(
        capability: WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    ) -> Self {
        Self { capability }
    }
}

impl<Schema, Capability, Operation, Input, Scope>
    WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>
    for WorthQueryGovernedTemporalOperationAuthorization<Schema, Capability, Operation, Input>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>
        + worth_query_declaration::facade::portable_identity::WorthQueryPortableType
        + Clone
        + Send
        + Sync
        + 'static,
    Capability: Send + Sync + 'static,
    Operation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity
            + Send
            + Sync
            + 'static,
{
    fn authorize<Principal, PrincipalIdentity>(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        _scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        input: &Input,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        let capability =
            product.admit_capability_access(principal, &self.capability, input.clone(), request)?;
        product
            .application()
            .authorize_capability_operation(capability, operation, preconditions)
    }
}
