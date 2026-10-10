//! Direct commit resolution is selected by owner-minted terminal evidence.

use super::{admitted_program, authenticated_principal, idempotency, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

#[test]
fn owner_terminal_evidence_resolves_the_direct_provider_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "typed-terminal-evidence",
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(93, 94),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("typed terminal fixture commits");
    };

    assert_eq!(receipt.changed_record_count(), 2);
    assert!(receipt.outcome_identity().is_some());
}
