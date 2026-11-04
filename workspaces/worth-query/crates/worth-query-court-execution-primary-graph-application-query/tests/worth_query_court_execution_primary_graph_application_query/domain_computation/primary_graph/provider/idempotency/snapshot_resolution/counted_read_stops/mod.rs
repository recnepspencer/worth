use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcomeIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::product_affinity::WorthQueryProductIdempotencyAffinity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolutionDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::idempotency_read_stop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::resolve_at_snapshot;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn durable_idempotency_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let provider = &(*(*world.application.runtime_for_test()).primary_provider_for_test());
            let layout = (*(*provider.graph_for_test()).layout_for_test()).provider_idempotency_for_test().clone();
            let binding = WorthQueryApplicationIdempotencyBinding::new_for_test([9; 32], [7; 32]);
            publish_relational_mutation(
                &execution,
                &world,
                worth_relational::facade::transactions::WorkerIntentBatch::new(
                    "paid-idempotency-proof",
                )
                .push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::idempotency_create_intent(
                    &layout,
                    binding,
                    WorthQueryApplicationCommitOutcomeIdentity::restore_for_test(1).unwrap(),
                    0,
                    worth_relational::facade::identity::PartitionId(1),
                )),
            );
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let snapshot = selected.application_basis_for_test().snapshot_handle();
            let affinity = WorthQueryProductIdempotencyAffinity::from_observation(
                selected.product().observation_for_test(),
            );
            (*provider.graph_for_test()).with_runtime(&execution, |runtime| {
                worth_query_execution::test_read_stops::every_consult_under(
                    &execution,
                    |request| {
                        resolve_at_snapshot(
                            request, provider, runtime, snapshot, &layout, &affinity, binding,
                        )
                        .map(|answer| {
                            matches!(answer, WorthQueryProviderIdempotencyResolution::Absent)
                        })
                        .or_else(|denial| {
                            if matches!(denial, WorthQueryProviderIdempotencyResolutionDenial::CommittedReceiptNotRetained { .. }) {
                                Ok(false)
                            } else {
                                Err(denial)
                            }
                        })
                    },
                    idempotency_read_stop(Cause::Cancelled),
                    idempotency_read_stop(Cause::WorkExhausted),
                );
            });
        })
        .unwrap();
}
