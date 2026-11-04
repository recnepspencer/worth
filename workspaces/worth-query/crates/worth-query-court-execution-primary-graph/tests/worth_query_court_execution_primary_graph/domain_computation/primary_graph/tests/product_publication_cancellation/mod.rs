use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationDenial as PublicationDenial;
use std::time::Duration;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::prepare_relational_mutation_on_application;

#[test]
fn cancellation_after_world_preparation_reaches_execution_before_owner_effects() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let prepared = prepare_relational_mutation_on_application(
        execution,
        &world.application,
        WorkerIntentBatch::new("cancel-after-world-preparation"),
        &request,
    );

    cancellation.cancel();
    let outcome = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|phase| {
        prepared.execute_for_test(phase.borrowed_request_for_test())
    });
    let Err(PublicationDenial::Stopped(stop)) = outcome else {
        panic!("cancellation after preparation must prevent owner effects")
    };
    assert_eq!(
        stop,
        worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
    );
}
