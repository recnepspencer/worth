use crate::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationDenial as PublicationDenial;
use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

use super::fixture::{installed_authorization_world, prepare_relational_mutation_on_application};

#[test]
fn cancellation_after_world_preparation_reaches_execution_before_owner_effects() {
    let host_request = crate::test_request_policy::serial_request();
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
    let outcome = crate::domain_computation::primary_graph::with_test_advancement(|phase| {
        prepared.execute(phase.borrowed_request_for_test())
    });
    let Err(PublicationDenial::Stopped(stop)) = outcome else {
        panic!("cancellation after preparation must prevent owner effects")
    };
    assert_eq!(
        stop,
        worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
    );
}
