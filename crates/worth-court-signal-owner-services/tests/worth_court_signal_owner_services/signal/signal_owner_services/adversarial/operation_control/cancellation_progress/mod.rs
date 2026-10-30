#[allow(unused_imports)] use crate::signal_owner_services::ordinary_context::adversarial::operation_control::cancellation_progress::*;
use {crate::signal_owner_services::ordinary_context::adversarial::operation_control::cancellation_progress::cancellation_custody::capture_result, crate::signal_owner_services::ordinary_context::adversarial::operation_control::cancellation_progress::cancellation_custody::retirement_plan};

pub(crate) use crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND;

#[test]
fn pre_movement_fork_cancellation_releases_destination_reservation() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let root_reference = world
        .basis
        .issue_managed_branch_reference(&world.root_basis)
        .expect("the fork retry uses an owner-issued root reference");
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let mutation = world.mutation.clone();
    let basis = world.root_basis.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let name = "cancelled-fork-boundary";

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .fork_exact(
                    worth_signal::facade::branch::validate_signal_branch_name(name)
                        .expect("the cancellation identity is valid"),
                    &basis,
                    &worker_cancellation.token(),
                )
                .map(|_| ());
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert!(matches!(
            rx.recv_timeout(PROGRESS_BOUND),
            Ok(Err(
                worth_signal::facade::branch::SignalBranchForkOperationDenial::CancelledNoMovement
            ))
        ));
    });

    assert_root_progress(&world);
    let retry_root = world
        .basis
        .observe_current(&root_reference)
        .expect("the retry reacquires the current root basis after progress");
    world
        .mutation
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name(name)
                .expect("the retry identity is valid"),
            &retry_root,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .expect("pre-movement cancellation returns destination custody");
}

#[test]
fn pre_movement_snapshot_cancellation_returns_output_custody() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open")
        .canonical_movements();

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .capture_exact(&basis, &worker_cancellation.token())
                .map(|_| ());
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert!(matches!(
            rx.recv_timeout(PROGRESS_BOUND),
            Ok(Err(worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial::CancelledNoMovement))
        ));
    });

    assert_eq!(
        world
            .basis
            .owner_service_cost_snapshot()
            .expect("the owner remains open")
            .canonical_movements(),
        before,
        "a cancelled snapshot does not publish a movement"
    );
    assert_root_progress(&world);
    let (snapshot, _) = capture_result(&world.mutation, &world.child_basis)
        .expect("snapshot output capacity is reusable after cancellation");
    drop(snapshot);
}

#[test]
fn pre_movement_restore_cancellation_returns_output_custody() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let (snapshot, captured_basis) = capture_result(&world.mutation, &world.child_basis)
        .expect("restore starts from an owner-issued snapshot");
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let mutation = world.mutation.clone();
    let basis = captured_basis.clone();
    let worker_snapshot = snapshot.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .restore_exact(&basis, &worker_snapshot, &worker_cancellation.token())
                .map(|_| ());
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert!(matches!(
            rx.recv_timeout(PROGRESS_BOUND),
            Ok(Err(
                worth_signal::facade::branch::SignalBranchRestoreDenial::CancelledNoMovement
            ))
        ));
    });

    assert_root_progress(&world);
    world
        .mutation
        .restore_exact(
            &captured_basis,
            &snapshot,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .expect("pre-movement cancellation returns restore custody");
}

#[test]
fn pre_movement_retirement_cancellation_preserves_a_retryable_branch() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the retirement target has managed reference custody");
    let target_basis = world
        .basis
        .observe_current(&reference)
        .expect("the retirement target has an exact current basis");
    drop(world.child_basis);
    let plan = retirement_plan(&world.lifecycle, target_basis);
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let lifecycle = world.lifecycle.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = lifecycle.retire_exact(plan, &worker_cancellation.token());
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert!(matches!(
            rx.recv_timeout(PROGRESS_BOUND),
            Ok(worth_proof::TransitionOutcome::Denied(
                worth_signal::facade::branch::SignalBranchRetirementDenial::CancelledNoMovement
            ))
        ));
    });

    assert_root_progress_with_basis(&world.mutation, &world.root_basis);
    let retry_basis = world
        .basis
        .observe_current(&reference)
        .expect("the cancelled retirement leaves the branch current");
    let retry = retirement_plan(&world.lifecycle, retry_basis);
    assert!(matches!(
        world.lifecycle.retire_exact(
            retry,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token()
        ),
        worth_proof::TransitionOutcome::Success(_)
    ));
}

#[test]
fn post_movement_snapshot_cancellation_keeps_the_performed_capture() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let expected = world.child_basis.clone();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .capture_exact(&basis, &worker_cancellation.token())
                .map(|outcome| outcome.into_parts())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        let (snapshot, captured) = rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("capture reports after release")
            .expect("post-movement cancellation cannot erase capture");
        assert_eq!(captured.branch_id(), expected.branch_id());
        assert_eq!(
            captured.observation().generation().get(),
            expected.observation().generation().get() + 1
        );
        drop(snapshot);
    });
    assert_root_progress(&world);
}

#[test]
fn post_movement_restore_cancellation_keeps_the_performed_restore() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let (snapshot, captured_basis) = capture_result(&world.mutation, &world.child_basis)
        .expect("restore starts from an owner-issued snapshot");
    let expected = captured_basis.clone();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let mutation = world.mutation.clone();
    let basis = captured_basis;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .restore_exact(&basis, &snapshot, &worker_cancellation.token())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        let restored = rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("restore reports after release")
            .expect("post-movement cancellation cannot erase restore");
        assert_eq!(restored.branch_id(), expected.branch_id());
        assert_eq!(
            restored.observation().generation().get(),
            expected.observation().generation().get() + 1
        );
    });
    assert_root_progress(&world);
}

#[test]
fn post_movement_retirement_cancellation_keeps_the_performed_receipt() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the retirement target has managed reference custody");
    let target_basis = world
        .basis
        .observe_current(&reference)
        .expect("the retirement target has an exact current basis");
    drop(world.child_basis);
    let plan = retirement_plan(&world.lifecycle, target_basis);
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("operation control is issued after sealing");
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    );
    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let lifecycle = world.lifecycle.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = lifecycle
                .retire_exact(plan, &worker_cancellation.token())
                .into_result()
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert!(matches!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(_))));
    });
    assert_root_progress_with_basis(&world.mutation, &world.root_basis);
}
