use crate::domain_computation::authorization::RequirementObservationScopedOps;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledApplicationQueryAuthorization,
};

use super::WorthQueryApplicationQueryAccessContext;
use crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact;
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial;

pub(in crate::domain_computation::primary_graph::application_query) trait AuthorizationObservationScopedOps<
    Schema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema> where
    Schema: ApplicationSchema,
{
    fn observe_query_authorization<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session_identity: crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: worth_relational::facade::snapshots::SnapshotHandle,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    {
        let WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) =
            query.authorization()
        else {
            return Ok(Vec::new());
        };
        self.observe_query_authorization_requirement(
            execution,
            session_identity,
            runtime,
            snapshot,
            query,
            access,
            requirement,
        )
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > AuthorizationObservationScopedOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}
