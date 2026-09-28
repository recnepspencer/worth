//! Authorization sources a conditional operation uses to admit its
//! reconstruction query; operation sources live in the `operation` submodule.

use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryControls,
    WorthQueryOperationAuthorizationDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryParameterSet, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationQuery,
};
mod operation;
pub use operation::{
    WorthQueryGovernedTemporalOperationAuthorization,
    WorthQueryPublicTemporalOperationAuthorization, WorthQueryTemporalOperationAuthorization,
};
#[cfg(test)]
#[path = "authorization_sources/tests.rs"]
mod tests;
/// Refusal to admit a conditional operation's reconstruction query.
///
/// Nothing was read.
#[derive(Debug)]
pub enum WorthQueryTemporalQueryAuthorizationDenial {
    /// Query admission refused the read.
    Query(WorthQueryApplicationQueryAdmissionDenial),
    /// Capability authorization refused the read.
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
/// How a conditional operation admits the query that reconstructs its pending
/// temporal intents.
///
/// Use [`WorthQueryPublicTemporalQueryAuthorization`] for a public query or
/// [`WorthQueryGovernedTemporalQueryAuthorization`] for one governed by a
/// capability. The query is admitted afresh each time it runs.
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

/// Admits a public reconstruction query with ordinary query admission.
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

/// Admits a governed reconstruction query under a fresh capability
/// authorization for the given capability and input.
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
        + Clone
        + Send
        + Sync
        + 'static,
    Capability: Send + Sync + 'static,
    CapabilityOperation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + Send
            + Sync
            + 'static,
    <CapabilityOperation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = CapabilityInput,
        >,
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
        let product = controls
            .publication_product_branch()
            .expect("temporal query authorization retains its selected product lease");
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
