use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledApplicationQueryAuthorization,
};

use super::WorthQueryApplicationQueryAccessContext;
use crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact;
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph::application_query) fn observe_query_authorization<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
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
