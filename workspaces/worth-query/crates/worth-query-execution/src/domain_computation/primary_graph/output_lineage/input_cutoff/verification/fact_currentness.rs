use super::*;

/// Re-verifies marked facts, or the full-verification prefix, on the
/// host-bounded source-currentness allowance.
pub(super) fn facts_are_current<'fact>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    currentness: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    for fact in facts {
        currentness.charge_external_work(1)?;
        if !fact_is_current(execution, fact, runtime, snapshot, currentness)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn fact_is_current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    match fact.source_currentness_in(runtime, execution, snapshot, admission)? {
        Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => Err(
            InputCutoffVerificationStop::Admission(CompanionPreflightStop::Execution(cause)),
        ),
        Ok(movement) => Ok(movement.movement() == Movement::Unmoved),
        Err(WorthQuerySourceCurrentnessFailure::Unavailable) => Ok(false),
        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
            Err(InputCutoffVerificationStop::WorkExhausted)
        }
    }
}
