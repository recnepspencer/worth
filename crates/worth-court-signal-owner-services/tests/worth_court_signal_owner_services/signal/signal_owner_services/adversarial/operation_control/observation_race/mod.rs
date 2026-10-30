#[allow(unused_imports)] use crate::signal_owner_services::ordinary_context::adversarial::operation_control::observation_race::*;
pub(crate) use {std::sync::mpsc};
pub(crate) use {std::thread};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::history::RuntimeBranchId;
pub(crate) use {
    worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::ManagedSignalBranchReference,
    worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial,
    worth_signal::facade::branch::SignalBranchBasisObservationDenial,
    worth_signal::facade::branch::SignalBranchObservation,
    worth_signal::facade::branch::SignalBranchRetirementDenial,
    worth_signal::facade::branch::SignalBranchRetirementReceipt,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest,
    worth_signal::facade::branch::SignalOwnerOperationControlForTest,
};

pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::prove_retirement_is_effectful,
    crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::retire,
    crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::retirement_plan,
};
pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    crate::signal_owner_services::ordinary_context::adversarial::world::BasisPort,
    crate::signal_owner_services::ordinary_context::adversarial::world::LifecyclePort,
    crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND,
};

#[test]
fn same_branch_observe_retire_returns_a_complete_pre_or_post_state() {
    prove_retirement_is_effectful();
    run_order(true);
    run_order(false);
}
