//! One named bound for every query a session reads or keeps index-ready, so
//! generic application code never restates projection and scope-resolution
//! where-clauses.
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexQueryIntent, ApplicationQueryBinding, ApplicationQueryIntent,
        ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationRootWindow, WorthQueryManagedIndex,
};
use worth_query_publication::facade::{
    application_entry::{
        WorthQueryApplicationReadObservation, WorthQueryApplicationRequestQueryDenial,
    },
    domain_computation::WorthQueryPublishedApplicationResult,
};

use super::{QueryBinding, QueryMarker, QueryRow, WorthQueryHostSession};

/// A query a [`WorthQueryHostSession`] reads: every
/// `ApplicationQueryIntent` whose scope resolves for its principal and whose
/// rows project implements it.
///
/// ```
/// use worth_query_host::facade::{WorthQueryHostSession, WorthQuerySessionQuery};
/// use worth_query_host::facade::declaration::{
///     application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
/// };
/// fn read<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, Q: WorthQuerySessionQuery<S>>(
///     session: &WorthQueryHostSession<'_, S, P>,
///     intent: Q,
/// ) -> Option<Vec<Q::Row>> {
///     session.query(intent).ok()
/// }
/// ```
pub trait WorthQuerySessionQuery<Schema: ApplicationSchema>:
    ApplicationQueryIntent<Schema>
{
    /// The projected row type.
    type Row;

    #[doc(hidden)]
    fn execute_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;

    #[doc(hidden)]
    fn execute_at_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;

    #[doc(hidden)]
    fn execute_window_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        at: Option<&WorthQueryApplicationReadObservation>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;
}

impl<Schema, Intent> WorthQuerySessionQuery<Schema> for Intent
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    type Row = QueryRow<Schema, Intent>;

    fn execute_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = request.query(self);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    fn execute_at_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = request.at(observation).query(self);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    fn execute_window_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        at: Option<&WorthQueryApplicationReadObservation>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = match at {
            Some(observation) => request.at(observation).query(self),
            None => request.query(self),
        }
        .window(window);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}

/// A query answered by a managed index the session can make ready.
pub trait WorthQuerySessionIndexedQuery<Schema: ApplicationSchema>:
    ApplicationManagedIndexQueryIntent<Schema>
{
    #[doc(hidden)]
    fn ensure_ready_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<WorthQueryManagedIndex<Schema, Self::Index>, WorthQueryApplicationRequestQueryDenial>;
}

impl<Schema, Intent> WorthQuerySessionIndexedQuery<Schema> for Intent
where
    Schema: ApplicationSchema,
    Intent: ApplicationManagedIndexQueryIntent<Schema>,
    <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    fn ensure_ready_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<WorthQueryManagedIndex<Schema, Self::Index>, WorthQueryApplicationRequestQueryDenial>
    {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        session
            .with_funded_advancement(&scope, |phase| {
                request
                    .query(self)
                    .ensure_index_ready_in_advancement::<Intent::Index>(&phase)
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}
