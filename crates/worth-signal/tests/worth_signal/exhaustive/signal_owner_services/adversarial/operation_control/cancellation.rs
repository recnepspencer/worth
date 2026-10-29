#[allow(unused_imports)]
pub(crate) use crate::fast::signal_owner_services::adversarial::operation_control::cancellation::*;
use crate::fast::signal_owner_services::execution_custody;

#[test]
fn pre_movement_cancellation_denies_every_cancellable_public_operation() {
    let world = AdversarialWorld::new();
    let snapshot_and_basis = world
        .mutation
        .capture_exact(
            &world.child_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the restore case begins with a real snapshot")
        .into_parts();
    let (snapshot, captured_basis): (AdmittedSignalBranchSnapshot, _) = snapshot_and_basis;
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open")
        .canonical_movements();

    let advance_cancel = SignalOwnerCancellationSource::new();
    advance_cancel.cancel();
    assert!(matches!(
        world.mutation.advance_exact(
            worth_execution::ExecutionRequest::serial(
                &execution_custody::operational_serial_request()
            ),
            &world.root_basis,
            &mut (),
            &advance_cancel.token(),
            |_| panic!("cancellation must precede the caller callback"),
        ),
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));

    let fork_cancel = SignalOwnerCancellationSource::new();
    fork_cancel.cancel();
    assert!(matches!(
        world.mutation.fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("cancelled-child")
                .expect("the cancellation fixture name is valid"),
            &world.root_basis,
            &fork_cancel.token(),
        ),
        Err(SignalBranchForkOperationDenial::CancelledNoMovement)
    ));

    let capture_cancel = SignalOwnerCancellationSource::new();
    capture_cancel.cancel();
    assert!(matches!(
        world
            .mutation
            .capture_exact(&world.root_basis, &capture_cancel.token()),
        Err(SignalBranchSnapshotCaptureDenial::CancelledNoMovement)
    ));

    let restore_cancel = SignalOwnerCancellationSource::new();
    restore_cancel.cancel();
    assert!(matches!(
        world
            .mutation
            .restore_exact(&captured_basis, &snapshot, &restore_cancel.token(),),
        Err(SignalBranchRestoreDenial::CancelledNoMovement)
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
    let plan = match world
        .lifecycle
        .plan_retirement_exact(retirement_basis, SignalBranchRetirementReason::Superseded)
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the cancellation retirement plan is valid: {other:?}"),
    };
    let retire_cancel = SignalOwnerCancellationSource::new();
    retire_cancel.cancel();
    assert!(matches!(
        world.lifecycle.retire_exact(plan, &retire_cancel.token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CancelledNoMovement)
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
                &execution_custody::operational_serial_request(),
            ),
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("an immediate healthy twin follows every cancellation");
}
