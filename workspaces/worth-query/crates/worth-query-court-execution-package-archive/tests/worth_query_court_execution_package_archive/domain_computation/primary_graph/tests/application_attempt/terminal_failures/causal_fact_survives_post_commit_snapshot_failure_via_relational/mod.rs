//! Actual typed correction causality survives a post-commit snapshot failure.
use worth_query_execution::facade::domain_computation::WorthQueryAftermathCausalRole;

#[test]
fn causal_fact_survives_post_commit_snapshot_failure_via_relational_owner_read() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::install(worth_execution::MonotonicClock::system());
    let source = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(&world, "snapshot-source", "edited-label");
    let prepared = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::prepare(&world, &source, "snapshot-correction");
    let pending = prepared.pending();
    let parent = pending.parent_for_test().clone();
    world.faults.fail_next_post_commit_snapshot();
    let receipt = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::committed(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::publish(&world, prepared));
    assert_eq!(
        world.faults.failed_post_commit_snapshot_consumption_count(),
        1
    );
    let carried = receipt
        .aftermath_causality()
        .expect("receipt recovers the actual causal fact");
    assert_eq!(
        carried.role(),
        WorthQueryAftermathCausalRole::RecordedReverse
    );
    assert_eq!(carried.parent(), &parent);
    assert_eq!(carried.child(), receipt.commit_reference());
    assert_eq!(carried.recorded_source(), &source.correction_source());
    let reread = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let provider = (*world.application.runtime_for_test()).primary_provider_for_test();
            let request = phase.borrowed_request_for_test();
            let basis = selected.product().relational_basis_for_test();
            (*provider.graph_for_test()).with_runtime_for_test(&request, |runtime| {
                let snapshot = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis).unwrap();
                let resolution = provider.resolve_aftermath_causality_in_snapshot_for_test(
                    &request, runtime, basis, &snapshot, &pending, None,
                );
                worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                resolution.expect("the native canonical read succeeds")
                    .expect("the co-committed fact exists")
            })
        })
        .unwrap();
    assert_eq!(reread, *carried);
}
