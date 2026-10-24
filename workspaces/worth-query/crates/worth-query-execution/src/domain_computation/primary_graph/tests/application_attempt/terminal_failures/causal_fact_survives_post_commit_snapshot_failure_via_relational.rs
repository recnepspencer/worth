//! Actual typed correction causality survives a post-commit snapshot failure.
use crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole;
use crate::domain_computation::primary_graph::tests::fixture::correction_source;

#[test]
fn causal_fact_survives_post_commit_snapshot_failure_via_relational_owner_read() {
    let world = correction_source::install();
    let source = correction_source::edit(&world, "snapshot-source", "edited-label");
    let prepared = correction_source::prepare(&world, &source, "snapshot-correction");
    let pending = prepared.pending();
    let parent = pending.parent().clone();
    world.faults.fail_next_post_commit_snapshot();
    let receipt = correction_source::committed(correction_source::publish(&world, prepared));
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
            let execution = phase.borrowed_request_for_test();
            let provider = &world.application.primary_provider;
            let selected = world.selected_product(&phase.test_host_request());
            let basis = selected.product().relational_basis();
            provider.graph.with_runtime_mut(&execution, |runtime| {
                let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                    .expect("the published basis admits an exact snapshot");
                let resolution = provider.resolve_aftermath_causality_in_snapshot(
                    &execution, runtime, basis, &snapshot, &pending, None,
                );
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                resolution
            })
            .expect("the native canonical read succeeds")
            .expect("the co-committed fact exists")
        })
        .unwrap();
    assert_eq!(reread, *carried);
}
