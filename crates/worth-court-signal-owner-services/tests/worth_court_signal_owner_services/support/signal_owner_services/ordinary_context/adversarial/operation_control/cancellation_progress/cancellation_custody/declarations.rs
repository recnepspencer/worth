//! Prepare the native custody consumed by cancellation courts.
pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::LifecyclePort,
    crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    worth_proof::TransitionOutcome, worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::AdmittedSignalBranchSnapshot,
    worth_signal::facade::branch::SignalBranchRetirementReason,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
};
pub(crate) fn capture_result(
    mutation: &MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis), String> {
    mutation
        .capture_exact(
            basis,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .map(|outcome| outcome.into_parts())
        .map_err(|denial| format!("{denial:?}"))
}

pub(crate) fn retirement_plan(
    lifecycle: &LifecyclePort,
    basis: AdmittedSignalBranchBasis,
) -> worth_signal::facade::branch::PlannedSignalBranchRetirement {
    match lifecycle.plan_retirement_exact(
        basis,
        worth_signal::facade::branch::SignalBranchRetirementReason::Superseded,
    ) {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("the child retirement plan is owner-issued: {other:?}"),
    }
}
