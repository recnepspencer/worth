//! unrelated graph drift preserves current idempotent recovery.
use crate::domain_computation::primary_graph::tests::authorization::capability_commit_revalidation::*;

#[test]
fn unrelated_graph_drift_preserves_current_idempotent_recovery() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| equivalent_programs(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");
    let committed = commit_first(&world, first, 55);
    world
        .application
        .with_host_advancement(|phase| {
            change_account_label(&phase, &world, "account-2", "independently-updated")
        })
        .expect("the fixture read owns its installed host call");

    let outcome = world.application.compare_and_commit_application(
        retry,
        idempotency(55, 55),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered) = outcome else {
        panic!("unrelated drift must preserve lawful idempotent recovery: {outcome:?}");
    };
    crate::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_original(
        &recovered, &committed,
    );
    assert!(recovered.new_product_head().memory().is_some());
}
