//! unrelated graph drift preserves current idempotent recovery.
use super::*;

#[test]
fn unrelated_graph_drift_preserves_current_idempotent_recovery() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            equivalent_programs(&phase, &world, &principal, &request)
        })
        .expect("the fixture read owns its installed host call");
    let committed = commit_first(&world, first, 55);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_account_label(&phase, &world, "account-2", "independently-updated")
        })
        .expect("the fixture read owns its installed host call");

    let outcome = world.application.compare_and_commit_application(
        retry,
        idempotency(55, 55),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = outcome else {
        panic!("unrelated drift must preserve lawful idempotent recovery: {outcome:?}");
    };
    assert!(recovered.is_same_authoritative_commit(&committed));
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
}
