//! Direct commit resolution is selected by owner-minted terminal evidence.

use super::{admitted_program, authenticated_principal, idempotency, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

#[test]
fn owner_terminal_evidence_resolves_the_direct_provider_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "typed-terminal-evidence",
    );
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
