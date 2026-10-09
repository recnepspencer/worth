use super::WorthQueryRegisteredProviderAttempt;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

impl WorthQueryRegisteredProviderAttempt<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression) fn assert_scoped_comparison(
        self,
        world: &AuthorizationWorld,
        foreign: &AuthorizationWorld,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
    ) {
        world
            .application
            .primary_provider
            .assert_comparison_scope_lifecycle(
                &self.staged.read_authority(),
                self.requests,
                request,
                cancellation,
                &foreign.application.primary_provider,
            );
        let _ = self.staged.abort();
        assert_eq!(world.application.provider_session_resource_count(), 0);
    }
}
