//! Actual unprepared-work refusal and same-key fresh effect, not Directory I/O.
use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFault;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome as Outcome;
use crate::domain_computation::WorthQueryProductPreparationRefusal;
use crate::facade::runtime::ExecutionAllocationPolicy;

#[test]
fn genuine_unprepared_refusal_preserves_winner_and_allows_fresh_same_key() {
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request(), &world, &scope)
        })
        .unwrap();
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&phase.test_host_request(), &world, "open", &scope)
        })
        .unwrap();
    let program = || {
        world
            .application
            .with_host_advancement(|phase| {
                admitted_program(&phase, &world, &principal, &account, &scope, "changed")
            })
            .unwrap()
    };
    world
        .faults
        .schedule(WorthQueryPrimaryGraphFault::DuplicateQueryPreparationSlot);
    let refused = world.application.compare_and_commit_application(
        program(),
        idempotency(171, 172),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let Outcome::ProductPreparationRefused(refused) = refused else {
        panic!("real owner-local Query reservation refusal: {refused:?}");
    };
    assert!(matches!(
        refused.reason(),
        WorthQueryProductPreparationRefusal::QueryPreparationReservationRefused
    ));
    // Both programs are genuinely admitted at their original product basis;
    // the second resolves the recorded key after that basis moves.
    let fresh_program = program();
    let replay_program = program();
    let fresh = world.application.compare_and_commit_application(
        fresh_program,
        idempotency(171, 172),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let Outcome::Committed(committed) = fresh else {
        panic!(
            "fresh key after genuine winning-slot disposal, while Refused remains owned: {fresh:?}"
        );
    };
    let replay = world.application.compare_and_commit_application(
        replay_program,
        idempotency(171, 172),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let Outcome::RecoveredCommitted(replayed) = replay else {
        panic!("one actual Native effect: {replay:?}");
    };
    assert_eq!(committed.commit_reference(), replayed.commit_reference());
    drop(refused);
}
