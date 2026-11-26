//! Opening a whole-result live subscription: resolved live limits and the
//! funded open shared by borrowed and owned typed subscriptions.
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationLiveOpenRequestDenial,
    WorthQueryApplicationWholeResultLiveSubscription,
};

use super::{QueryBinding, QueryMarker, QueryRow, WorthQueryHostSession};

impl<'home, Schema, Program> WorthQueryHostSession<'home, Schema, Program>
where
    Schema: ApplicationSchema,
{
    pub(super) fn resolved_live_limits<Intent>(&self) -> WorthQueryApplicationLiveLimits
    where
        Intent: ApplicationQueryIntent<Schema>,
    {
        let maximum_results = match self.live_maximum_results {
            Some(explicit) => explicit,
            None => {
                let installed_max = self
                    .home
                    .primary_graph()
                    .installed_schema()
                    .installed_query_binding::<QueryBinding<Schema, Intent>>()
                    .map(|binding| {
                        self.home
                            .primary_graph()
                            .resolve_application_query_limits(binding.limits())
                            .maximum_results()
                            .get()
                    })
                    .unwrap_or(super::DEFAULT_LIVE_MAXIMUM_RESULTS);
                match self.query_result_limit {
                    Some(narrowed) => narrowed.get().min(installed_max),
                    None => installed_max,
                }
            }
        };
        WorthQueryApplicationLiveLimits::bounded(self.live_buffer_capacity, maximum_results)
    }

    /// Opens the untyped whole-result subscription behind [`Self::subscribe`].
    pub(super) fn open_whole_result<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'home, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let limits = self.resolved_live_limits::<Intent>();
        self.with_funded_advancement(&scope, |phase| {
            query_req.subscribe_whole_result_in_advancement(&phase, limits)
        })
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
    }
}
