//! Prepare the native custody consumed by cancellation courts.
use super::*;
pub(crate) fn capture_result(
    mutation: &MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis), String> {
    mutation
        .capture_exact(basis, &SignalOwnerCancellationSource::new().token())
        .map(|outcome| outcome.into_parts())
        .map_err(|denial| format!("{denial:?}"))
}

pub(crate) fn retirement_plan(
    lifecycle: &LifecyclePort,
    basis: AdmittedSignalBranchBasis,
) -> worth_signal::facade::branch::PlannedSignalBranchRetirement {
    match lifecycle.plan_retirement_exact(basis, SignalBranchRetirementReason::Superseded) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the child retirement plan is owner-issued: {other:?}"),
    }
}
