//! Counted owner reads retain request stops for a real co-committed correction fact.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::aftermath_causality::aftermath_read_stop;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[test]
fn aftermath_resolution_preserves_stops_at_every_consult() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::install();
    let source = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(&world, "counted-source", "edited-label");
    let prepared = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::prepare(&world, &source, "counted-correction");
    let pending = prepared.pending();
    let receipt = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::committed(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::publish(&world, prepared));
    let expected = receipt.aftermath_causality().unwrap().clone();
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let provider = &world.application.primary_provider;
            let selected = world.selected_product(&phase.test_host_request());
            let basis = selected.product().relational_basis();
            let snapshot = provider.graph.with_runtime_mut(&execution, |runtime| {
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis).unwrap()
            });
            let read = |request: &worth_execution::ExecutionRequest<'_, '_>| {
                provider.graph.with_runtime_mut(request, |runtime| {
                    provider.resolve_aftermath_causality_in_snapshot(
                        request, runtime, basis, &snapshot, &pending, None,
                    )
                })
            };
            assert_eq!(read(&execution).unwrap(), Some(expected));
            worth_query_execution::test_read_stops::every_consult_under(
                &execution,
                read,
                aftermath_read_stop(Cause::Cancelled),
                aftermath_read_stop(Cause::WorkExhausted),
            );
            provider.graph.with_runtime_mut(&execution, |runtime| {
                worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
        })
        .unwrap();
}
