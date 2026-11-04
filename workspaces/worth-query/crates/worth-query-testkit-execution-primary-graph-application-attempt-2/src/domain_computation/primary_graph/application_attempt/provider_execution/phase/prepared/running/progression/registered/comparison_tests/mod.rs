use crate::domain_computation::primary_graph::provider::decision_facts::comparison::tests::TestsWorthQueryPrimaryGraphProviderForTest;
use crate::domain_computation::primary_graph::provider::decision_facts::comparison::tests::principal_staleness::PrincipalStalenessWorthQueryPrimaryGraphProviderForTest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryRegisteredProviderAttempt;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// Test doors on a library type, owned by the moved courts.
pub trait ComparisonTestsWorthQueryRegisteredProviderAttemptForTest: Sized {
    fn assert_scoped_comparison(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &AuthorizationWorld,
        foreign: &AuthorizationWorld,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
    );
    fn assert_principal_comparison(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &AuthorizationWorld,
        request: &WorthQueryRequestScope,
        relation: worth_relational::facade::identity::RelationId,
    );
}

impl ComparisonTestsWorthQueryRegisteredProviderAttemptForTest
    for WorthQueryRegisteredProviderAttempt<'_>
{
    fn assert_scoped_comparison(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &AuthorizationWorld,
        foreign: &AuthorizationWorld,
        request: &WorthQueryRequestScope,
        cancellation: &WorthQueryCancellationSource,
    ) {
        (*world.application.primary_provider_for_test()).assert_comparison_scope_lifecycle(
            execution,
            &(*self.staged_for_test()).read_authority(),
            request,
            cancellation,
            &(*foreign.application.primary_provider_for_test()),
        );
        let _ = self.staged_into_for_test().abort();
        assert_eq!(
            world.application.provider_session_resource_count_for_test(),
            0
        );
    }

    fn assert_principal_comparison(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &AuthorizationWorld,
        request: &WorthQueryRequestScope,
        relation: worth_relational::facade::identity::RelationId,
    ) {
        (*world.application.primary_provider_for_test()).assert_principal_stale_and_stopped(
            execution,
            &(*self.staged_for_test()).read_authority(),
            request,
            relation,
        );
        let _ = self.staged_into_for_test().abort();
    }
}
