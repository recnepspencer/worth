#[allow(unused_imports)]
pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    worth_proof::TransitionOutcome, worth_signal::facade::branch::AdmittedSignalBranchSnapshot,
    worth_signal::facade::branch::SignalBranchAdvanceDenial,
    worth_signal::facade::branch::SignalBranchForkOperationDenial,
    worth_signal::facade::branch::SignalBranchRestoreDenial,
    worth_signal::facade::branch::SignalBranchRetirementDenial,
    worth_signal::facade::branch::SignalBranchRetirementReason,
    worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
};

#[test]
fn pre_movement_cancellation_denies_every_cancellable_public_operation() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let snapshot_and_basis = world
        .mutation
        .capture_exact(
            &world.child_basis,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .expect("the restore case begins with a real snapshot")
        .into_parts();
    let (snapshot, captured_basis): (AdmittedSignalBranchSnapshot, _) = snapshot_and_basis;
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open")
        .canonical_movements();

    let advance_cancel = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    advance_cancel.cancel();
    assert!(matches!(
        world.mutation.advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request()
            ),
            &world.root_basis,
            &mut (),
            &advance_cancel.token(),
            |_| panic!("cancellation must precede the caller callback"),
        ),
        Err(worth_signal::facade::branch::SignalBranchAdvanceDenial::CancelledNoMovement)
    ));

    let fork_cancel = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    fork_cancel.cancel();
    assert!(matches!(
        world.mutation.fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("cancelled-child")
                .expect("the cancellation fixture name is valid"),
            &world.root_basis,
            &fork_cancel.token(),
        ),
        Err(worth_signal::facade::branch::SignalBranchForkOperationDenial::CancelledNoMovement)
    ));

    let capture_cancel = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    capture_cancel.cancel();
    assert!(matches!(
        world
            .mutation
            .capture_exact(&world.root_basis, &capture_cancel.token()),
        Err(worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial::CancelledNoMovement)
    ));

    let restore_cancel = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    restore_cancel.cancel();
    assert!(matches!(
        world
            .mutation
            .restore_exact(&captured_basis, &snapshot, &restore_cancel.token(),),
        Err(worth_signal::facade::branch::SignalBranchRestoreDenial::CancelledNoMovement)
    ));

    let retirement_reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the cancellation retirement target has a managed reference");
    let retirement_basis = world
        .basis
        .observe_current(&retirement_reference)
        .expect("the cancellation retirement target has independent custody");
    drop(snapshot);
    drop(captured_basis);
    drop(world.child_basis);
    let plan = match world.lifecycle.plan_retirement_exact(
        retirement_basis,
        worth_signal::facade::branch::SignalBranchRetirementReason::Superseded,
    ) {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("the cancellation retirement plan is valid: {other:?}"),
    };
    let retire_cancel = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    retire_cancel.cancel();
    assert!(matches!(
        world.lifecycle.retire_exact(plan, &retire_cancel.token()),
        worth_proof::TransitionOutcome::Denied(
            worth_signal::facade::branch::SignalBranchRetirementDenial::CancelledNoMovement
        )
    ));

    assert_eq!(
        world
            .basis
            .owner_service_cost_snapshot()
            .expect("pre-movement cancellation leaves the owner open")
            .canonical_movements(),
        before,
        "all cancelled operations are effect-free"
    );
    world
        .mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request(),
            ),
            &world.root_basis,
            &mut (),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("an immediate healthy twin follows every cancellation");
}
