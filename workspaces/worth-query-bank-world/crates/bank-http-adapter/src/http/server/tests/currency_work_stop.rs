//! A real currency refusal reaches the execution twin's HTTP remedy.
use bank_domain::model::AccountId;
use bank_server::BankReadControls;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use worth_foundational::ExecutionBudget;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::{
    bound_advancement_requests_on_this_thread_for_test,
    place_managed_computations_on_this_thread_for_test, WorthQueryExecutionPlacementForTest,
};
#[tokio::test]
async fn one_unit_short_at_currency_has_the_execution_http_status() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let account = AccountId::new(100).unwrap();
    let application = super::fixture::application(account);
    let source = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        source.token(),
    );
    let principal = application.authenticate(&scope).await;
    let indexes = application
        .runtime
        .release_primary_index_generations_for_test();
    assert!(indexes > 1);
    struct Restore(WorthQueryExecutionPlacementForTest, Option<ExecutionBudget>);
    impl Drop for Restore {
        fn drop(&mut self) {
            place_managed_computations_on_this_thread_for_test(self.0);
            bound_advancement_requests_on_this_thread_for_test(self.1);
        }
    }
    let old_place = place_managed_computations_on_this_thread_for_test(
        WorthQueryExecutionPlacementForTest::Serial,
    );
    let old_budget =
        bound_advancement_requests_on_this_thread_for_test(Some(ExecutionBudget::new(
            NonZeroUsize::MIN,
            Some(64 * 1024 * 1024),
            Some((indexes - 1) as u64),
        )));
    let _restore = Restore(old_place, old_budget);
    let denied = application
        .runtime
        .account_activity(account)
        .as_principal(&principal)
        .execute(BankReadControls::current(scope, 16, 20_000).unwrap())
        .err()
        .expect("one short cannot rebuild currency");
    match &denied {
        bank_server::BankApplicationQueryDenial::PrincipalResolution(
            worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(kind)),
        ) => assert!(matches!(kind, worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted, ..
        })),
        other => panic!("expected the currency request stop: {other:?}"),
    }
    let http = super::super::query_denial::query_denial(denied);
    use super::super::super::protocol::{BankHttpDenialKind as Kind, BankHttpNextAction as Next};
    assert_eq!(http.kind, Kind::MalformedRequest);
    assert_eq!(http.next_action, Next::CorrectRequest);
}
