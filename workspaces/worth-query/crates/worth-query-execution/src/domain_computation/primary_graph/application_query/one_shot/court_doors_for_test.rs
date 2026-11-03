//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> where
    Schema: ApplicationSchema, {
    #[doc(hidden)]
    pub fn execute_application_query_one_shot_with_request_for_test<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >) -> Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>, {
        Self::execute_application_query_one_shot_with_request::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(self, execution, plan)
    }
}
