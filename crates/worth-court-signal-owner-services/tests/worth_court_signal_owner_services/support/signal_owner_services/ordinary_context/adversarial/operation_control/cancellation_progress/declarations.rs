pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::operation_control::cancellation_progress::cancellation_custody::capture_result, crate::signal_owner_services::ordinary_context::adversarial::operation_control::cancellation_progress::cancellation_custody::retirement_plan};

pub(crate) use crate::signal_owner_services::ordinary_context::execution_custody;
pub(crate) use std::sync::mpsc;
pub(crate) use std::thread;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::AdmittedSignalBranchSnapshot,
    worth_signal::facade::branch::SignalBranchForkOperationDenial,
    worth_signal::facade::branch::SignalBranchRestoreDenial,
    worth_signal::facade::branch::SignalBranchRetirementDenial,
    worth_signal::facade::branch::SignalBranchRetirementReason,
    worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest as BoundaryForTest,
};

pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    crate::signal_owner_services::ordinary_context::adversarial::world::LifecyclePort,
    crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND,
};

pub(crate) fn assert_root_progress_with_basis(
    mutation: &crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    root_basis: &AdmittedSignalBranchBasis,
) {
    mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request(),
            ),
            root_basis,
            &mut (),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("a healthy sibling follows the cancelled operation");
}

pub(crate) fn assert_root_progress(world: &AdversarialWorld) {
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
        .expect("a healthy sibling follows the cancelled operation");
}
