//! Registration recovery must preserve the original computation's own-write staleness.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{
        WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
        WorthQueryOutputReadinessDeliveryEvidence,
    },
    output_lineage::input_cutoff::RetainedInputCutoffCandidate,
    tests::{application_attempt::resolved_account, fixture::live_scope},
};

#[test]
fn accepted_current_cannot_bypass_original_own_write_evidence() {
    crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write(|world, receipt, cell, _| {
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let retained_product = world.application.with_host_advancement(|phase| { let phase_custody = &phase.test_host_request();    world.selected_product(phase_custody, ) }).expect("the fixture read owns its installed host call");
    let retained_snapshot = world.application.with_host_advancement(|phase| {   let borrowed_execution = phase.execution_request_for(&world.application.product_runtime).expect("fixture borrows its runtime request"); let execution = &borrowed_execution; handle.with_runtime(execution, |runtime| {
        let basis = runtime.admit_branch_basis(&runtime.main_branch_identity()).unwrap();
        runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap()
    }) }).expect("the fixture read owns its installed host call");
    let recorded = cell.get().unwrap();
    assert!(recorded.observed_source_facts().unwrap().for_comparison().is_none(),
        "accepted-current cannot obtain comparable postconditions of a stale computation");
    let selected = RetainedInputCutoffCandidate::from_exact_cell(cell);
    let ready = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),

    });
    let candidate = AcceptedCurrentCandidate { selected, ready };
    let (_, product, _) = retained_product.into_parts();
    world.application.with_host_advancement(|phase| {   let borrowed_execution = phase.execution_request_for(&world.application.product_runtime).expect("fixture borrows its runtime request"); let execution = &borrowed_execution; handle.with_runtime(execution, |runtime| {
        let snapshot = retained_snapshot;
        let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
        assert!(candidate
            .decision_input_changed_at(execution, owner, runtime, &snapshot, &selected)
            .unwrap(),
            "the performed own write is evidence of a changed decision input");

        let answer = candidate.certify_current(execution, owner, runtime, product.read_lease_ref(), &snapshot, &selected).unwrap();
        assert!(
            matches!(answer, CurrentAcceptedResult::NeedsDisclosure),
            "the output computed from x=1 cannot be Current after its own write of x=2"
        );
        drop(answer);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    }) }).expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let phase_custody = &phase.test_host_request();    resolved_account(&(phase_custody).test_host_request(), world, "2", &live_scope()) }).expect("the fixture read owns its installed host call");
    });
}

#[test]
fn checkpoint_excludes_facts_of_output_computed_before_its_own_write() {
    crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write(|_, _, cell, _| {
        assert!(cell.get().unwrap().checkpoint_source_facts().is_none());
    });
}
