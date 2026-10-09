//! Callback rollback preserves canonical truth and sibling health.
use super::super::world::AdversarialWorld;
use std::panic::{catch_unwind, AssertUnwindSafe};
use worth_signal::facade::branch::SignalOwnerCancellationSource;

#[test]
fn transaction_callback_panic_rolls_back_and_keeps_source_and_sibling_healthy() {
    let world = AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.root_basis)
        .expect("the rollback observation uses owner-issued reference custody");
    let fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.mutation.advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::execution_custody::operational_serial_request(),
            ),
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| panic!("transaction callback failure"),
        );
    }));
    assert!(
        fault.is_err(),
        "the transaction callback panic must reach the caller"
    );
    // A caller panic whose rollback succeeds restores the canonical state: the branch is
    // quarantined only when that rollback itself fails.
    let observed = world
        .basis
        .observe_current(&reference)
        .expect("a successful rollback leaves the source observable");
    assert_eq!(
        observed.observation().canonical_encoding(),
        world.root_basis.observation().canonical_encoding(),
        "the rolled-back callback performed no canonical movement"
    );
    drop(observed);
    world
        .mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::execution_custody::operational_serial_request(),
            ),
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("a successful rollback keeps the source cell usable");
    world
        .mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::execution_custody::operational_serial_request(),
            ),
            &world.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("a transaction panic must not poison an unrelated branch");
}
