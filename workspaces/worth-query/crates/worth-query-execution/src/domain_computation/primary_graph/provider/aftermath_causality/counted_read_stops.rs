//! Counted owner reads retain request stops for a real co-committed correction fact.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::correction_source;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

/// Opens an exact snapshot of `basis` under the runtime's own custody, reads
/// through the production publication read, and releases the snapshot.
fn read_at_basis(
    provider: &WorthQueryPrimaryGraphProvider,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pending: &WorthQueryPendingAftermathCausality,
) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial> {
    provider.graph.with_runtime_mut(execution, |runtime| {
        let snapshot =
            crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, basis,
            )
            .expect("the published basis admits an exact snapshot");
        let resolution = provider.resolve_aftermath_causality_in_snapshot(
            execution, runtime, basis, &snapshot, pending, None,
        );
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        resolution
    })
}

#[test]
fn aftermath_resolution_preserves_stops_at_every_consult() {
    let world = correction_source::install();
    let source = correction_source::edit(&world, "counted-source", "edited-label");
    let prepared = correction_source::prepare(&world, &source, "counted-correction");
    let pending = prepared.pending();
    let receipt = correction_source::committed(correction_source::publish(&world, prepared));
    let expected = receipt.aftermath_causality().unwrap().clone();
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let provider = &world.application.primary_provider;
            let selected = world.selected_product(&phase.test_host_request());
            let basis = selected.product().relational_basis();
            assert_eq!(
                read_at_basis(provider, &execution, basis, &pending).unwrap(),
                Some(expected)
            );
            crate::test_read_stops::every_consult_under(
                &execution,
                |request| read_at_basis(provider, request, basis, &pending),
                aftermath_read_stop(Cause::Cancelled),
                aftermath_read_stop(Cause::WorkExhausted),
            );
        })
        .unwrap();
}
