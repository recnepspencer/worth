use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcomeIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolutionDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::idempotency_read_stop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::resolve_at_snapshot;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn durable_idempotency_door_preserves_stops_at_every_consult() {
    // The selected original and source bytes are immutable. Reuse that truth,
    // but open a fresh real parent for EACH read rather than billing the whole
    // sweep to one ancestor or rebuilding installation for every cancellation.
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let provider = &world
        .application
        .runtime_for_test()
        .primary_provider_for_test();
    let layout = provider
        .graph_for_test()
        .layout_for_test()
        .provider_idempotency_for_test()
        .clone();
    let binding = WorthQueryApplicationIdempotencyBinding::new_for_test([9; 32], [7; 32]);
    world
        .application
        .with_host_advancement(|phase| {
            publish_relational_mutation(
                &phase.borrowed_request_for_test(),
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
                    None,
                    false,
                )),
            );
        })
        .unwrap();
    worth_query_execution::test_read_stops::every_consult_in_fresh_scope(
        16 << 20,
        |counted| {
            world
                .application
                .with_host_advancement(|phase| {
                    let execution = phase.borrowed_request_for_test();
                    let selected = world.selected_product(&phase.test_host_request_for_test());
                    let snapshot = selected.application_basis_for_test().snapshot_handle();
                    let product = selected.product().publication_binding_for_test();
                    provider
                        .graph_for_test()
                        .with_runtime(&execution, |runtime| {
                            counted(Some(&execution), &|request| {
                                resolve_at_snapshot(
                                    request, provider, runtime, snapshot, &layout, &product,
                                    binding,
                                )
                                .map(|answer| {
                                    matches!(
                                        answer,
                                        WorthQueryProviderIdempotencyResolution::Absent
                                    )
                                })
                                .or_else(|denial| {
                                    if denial
                                    == WorthQueryProviderIdempotencyResolutionDenial::Unavailable
                                {
                                    Ok(false)
                                } else {
                                    Err(denial)
                                }
                                })
                            })
                        })
                })
                .unwrap()
        },
        idempotency_read_stop(Cause::Cancelled),
        idempotency_read_stop(Cause::WorkExhausted),
    );
}
