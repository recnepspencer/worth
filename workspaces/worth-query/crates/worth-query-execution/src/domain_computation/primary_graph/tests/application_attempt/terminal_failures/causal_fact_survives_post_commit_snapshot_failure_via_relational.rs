//! causal fact survives post commit snapshot failure via relational owner read.
use super::*;

#[test]
fn causal_fact_survives_post_commit_snapshot_failure_via_relational_owner_read() {
    use crate::domain_computation::application_aftermath::{
        WorthQueryAftermathCausalRole, WorthQueryPendingAftermathCausality,
    };

    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "causal-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");
    let parent = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .relational_basis()
                .observation()
                .commit_receipt()
                .cloned()
                .expect("fixture has an authoritative branch head")
        })
        .expect("the fixture read owns its installed host call");
    let pending = WorthQueryPendingAftermathCausality::undo_of(parent.clone());

    world.faults.fail_next_post_commit_snapshot();
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            world
                .application
                .compare_and_commit_application_with_aftermath(
                    phase,
                    program,
                    idempotency(29, 29),
                    pending.clone(),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
        })
        .expect("fixture owner admits its advancement")
    else {
        panic!("owner reconstruction must recover the causal commit");
    };
    let carried = receipt
        .aftermath_causality()
        .expect("committed receipt carries recovered causal fact");
    assert_eq!(carried.role(), WorthQueryAftermathCausalRole::Undo);
    assert_eq!(carried.parent(), &parent);
    assert_eq!(carried.child(), receipt.commit_reference());

    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let reread = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .primary_provider
                .resolve_aftermath_causality_at_basis(
                    &host_custody.borrowed_request_for_test(),
                    selected.product().relational_basis(),
                    &pending,
                    None,
                )
                .expect("owner read succeeds")
                .expect("co-committed fact remains visible")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(reread, *carried);
}
