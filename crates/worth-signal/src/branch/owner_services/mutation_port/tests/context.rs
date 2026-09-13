use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::branch::{
    validate_signal_branch_name, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchSnapshotCaptureDenial,
};

use super::super::super::SignalOwnerCancellationSource;
use super::world::{set_dependency, MutationWorld};

struct NonCloneRequestContext {
    caller_marker: u32,
    dropped: Arc<AtomicBool>,
}

impl Drop for NonCloneRequestContext {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

#[test]
fn non_clone_context_and_local_callback_borrows_return_on_every_advance_posture() {
    let world = MutationWorld::<NonCloneRequestContext>::new();
    let dropped = Arc::new(AtomicBool::new(false));
    let mut context = NonCloneRequestContext {
        caller_marker: 7,
        dropped: Arc::clone(&dropped),
    };
    let mut local_callback_state = String::from("caller-owned");
    let current = world
        .port
        .advance_exact(
            &world.source_basis,
            &mut context,
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                local_callback_state.push_str("-success");
                set_dependency(transaction, world.derived, world.input_b)
            },
        )
        .expect("success synchronously returns the non-Clone context");
    context.caller_marker += 1;
    assert_eq!(context.caller_marker, 8);
    assert_eq!(local_callback_state, "caller-owned-success");

    let stale_callback_ran = Arc::new(AtomicBool::new(false));
    let stale_probe = Arc::clone(&stale_callback_ran);
    assert!(matches!(
        world.port.advance_exact(
            &world.source_basis,
            &mut context,
            &SignalOwnerCancellationSource::new().token(),
            move |_| {
                stale_probe.store(true, Ordering::Release);
                Ok(())
            },
        ),
        Err(SignalBranchAdvanceDenial::BasisMismatch { .. })
    ));
    assert!(!stale_callback_ran.load(Ordering::Acquire));
    context.caller_marker += 1;

    let cancelled = SignalOwnerCancellationSource::new();
    cancelled.cancel();
    let cancelled_callback_ran = Arc::new(AtomicBool::new(false));
    let cancelled_probe = Arc::clone(&cancelled_callback_ran);
    assert!(matches!(
        world.port.advance_exact(
            current.advanced_basis(),
            &mut context,
            &cancelled.token(),
            move |_| {
                cancelled_probe.store(true, Ordering::Release);
                Ok(())
            },
        ),
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));
    assert!(!cancelled_callback_ran.load(Ordering::Acquire));
    context.caller_marker += 1;

    let ledger_before_panic = world.owner.retention_ledger_observation();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.port.advance_exact(
            current.advanced_basis(),
            &mut context,
            &SignalOwnerCancellationSource::new().token(),
            |_| -> Result<(), crate::data::error::SignalError> {
                panic!("injected real transaction callback panic")
            },
        );
    }));
    assert!(panic.is_err());
    let ledger_after_panic = world.owner.retention_ledger_observation();
    assert_eq!(
        ledger_after_panic.used_capacity, ledger_before_panic.used_capacity,
        "callback unwind returns its pending output reservation"
    );
    assert_eq!(ledger_after_panic.reserved_admitted_lease_count, 0);
    context.caller_marker += 1;
    assert_eq!(context.caller_marker, 11);
    assert!(!dropped.load(Ordering::Acquire));
    assert!(matches!(
        world.port.advance_exact(
            current.advanced_basis(),
            &mut context,
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        ),
        Err(SignalBranchAdvanceDenial::QuarantinedBranch { branch_id })
            if branch_id == world.source_branch.id
    ));

    let sibling_dropped = Arc::new(AtomicBool::new(false));
    let mut sibling_context = NonCloneRequestContext {
        caller_marker: 100,
        dropped: Arc::clone(&sibling_dropped),
    };
    world
        .port
        .advance_exact(
            &world.sibling_basis,
            &mut sibling_context,
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("callback panic on one cell cannot block an unrelated context and cell");
    sibling_context.caller_marker += 1;
    assert_eq!(sibling_context.caller_marker, 101);
    drop(sibling_context);
    assert!(sibling_dropped.load(Ordering::Acquire));
    drop(context);
    assert!(dropped.load(Ordering::Acquire));
}

#[test]
fn same_thread_owner_reentry_denies_all_methods_before_nested_cell_work() {
    let world = MutationWorld::<()>::new();
    let captured = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("snapshot setup performs");
    let current = captured.captured_basis();
    let admission = world.owner.admit().expect("outer owner work admits");
    let cell = world
        .owner
        .lookup_cell(&admission, world.source_branch.id)
        .expect("outer work selects the source cell");
    let before = world.owner.cost_snapshot();
    cell.with_state(&admission, |_, _| {
        assert!(matches!(
            world.port.fork_exact(
                validate_signal_branch_name("reentrant-fork").expect("name validates"),
                current,
                &SignalOwnerCancellationSource::new().token(),
            ),
            Err(SignalBranchForkOperationDenial::OwnerReentry)
        ));
        assert!(matches!(
            world.port.advance_exact(
                current,
                &mut (),
                &SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            ),
            Err(SignalBranchAdvanceDenial::OwnerReentry)
        ));
        assert!(matches!(
            world
                .port
                .capture_exact(current, &SignalOwnerCancellationSource::new().token()),
            Err(SignalBranchSnapshotCaptureDenial::OwnerReentry)
        ));
        assert!(matches!(
            world.port.restore_exact(
                current,
                captured.admitted_snapshot(),
                &SignalOwnerCancellationSource::new().token(),
            ),
            Err(SignalBranchRestoreDenial::OwnerReentry)
        ));
    })
    .expect("the outer canonical cell remains healthy");
    let after = world.owner.cost_snapshot();
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups()
    );
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    drop(admission);
    world
        .port
        .advance_exact(
            current,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("dropping the outer hold permits an immediate healthy call");
}
