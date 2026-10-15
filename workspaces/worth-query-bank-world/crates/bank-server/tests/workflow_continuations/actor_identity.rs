//! Carried rejection input never substitutes the freshly admitted actor.
use super::*;
#[test]
fn owning_rejection_preserves_original_actor_and_payment_truth() {
    let fixture = ordinary_read_world("original-rejector", 0);
    let approver = fixture.authenticate(APPROVER);
    let pending = pending_continuation(&fixture, &approver);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::reject_payment(RejectPayment {
            payment: pending.payment_id(),
            rejecting_principal: principal_id(OWNER),
        }))
        .as_principal(&approver)
        .controls(controls("wrong-original-actor"))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared rejection: {cause:?}"));
    let scope = request_scope();
    let request = fixture.world.runtime.request(&approver, &scope);
    let Advance::Terminal(terminal) = operation.advance(&request) else {
        panic!("original wrong actor denied before effect")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            BankProposalDenial::AuthenticatedActorMismatch
        ))
    ));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("no-effect close: {cause:?}"));
    assert_eq!(
        pending_continuation(&fixture, &approver).payment_id(),
        pending.payment_id()
    );
}
