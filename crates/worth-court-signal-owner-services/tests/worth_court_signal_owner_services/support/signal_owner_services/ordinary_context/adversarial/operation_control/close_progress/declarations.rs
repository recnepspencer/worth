pub(crate) use crate::signal_owner_services::ordinary_context::execution_custody;
pub(crate) use std::sync::mpsc;
pub(crate) use std::thread;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::AdmittedSignalBranchSnapshot,
    worth_signal::facade::branch::SignalBranchAdvanceDenial,
    worth_signal::facade::branch::SignalBranchBasisPort,
    worth_signal::facade::branch::SignalBranchLifecyclePort,
    worth_signal::facade::branch::SignalBranchMutationPort,
    worth_signal::facade::branch::SignalBranchRetirementReason,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerLifecycleObservation,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest,
};

pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND,
};

pub(crate) fn assert_late_admission_denied(
    mutation: &SignalBranchMutationPort<(), (), (), (), ()>,
    root_basis: &AdmittedSignalBranchBasis,
) {
    let late = mutation.advance_exact(
        worth_execution::ExecutionRequest::serial(&crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request()),
        root_basis,
        &mut (),
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        |_| panic!("close must deny late work before the caller callback"),
    );
    assert!(matches!(
        late,
        Err(worth_signal::facade::branch::SignalBranchAdvanceDenial::OwnerUnavailable(_))
    ));
}

pub(crate) fn assert_closed(
    lifecycle: &SignalBranchLifecyclePort<(), (), ()>,
    basis: &SignalBranchBasisPort<(), (), ()>,
) {
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    );
    assert!(basis.owner_service_cost_snapshot().is_err());
}
