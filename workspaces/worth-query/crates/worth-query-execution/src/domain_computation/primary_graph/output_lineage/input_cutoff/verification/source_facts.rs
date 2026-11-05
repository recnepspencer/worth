//! Current source facts preserve the request stop through input cutoff.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::movement::Movement;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::source_currentness::WorthQuerySourceCurrentnessFailure;

pub fn marked_facts_permit_reuse<'fact>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<bool, InputCutoffVerificationStop> {
    match facts_are_current(execution, facts, runtime, snapshot) {
        Ok(current) => Ok(current),
        Err(InputCutoffVerificationStop::WorkExhausted) => {
            observation::reach(Gate::HandlerPrefixAllowance);
            Ok(false)
        }
        Err(stop) => Err(stop),
    }
}

/// Re-verifies marked facts, or the full-verification prefix, on the
/// host-bounded source-currentness allowance.
pub(super) fn facts_are_current<'fact>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<bool, InputCutoffVerificationStop> {
    for fact in facts {
        if !fact_is_current(execution, fact, runtime, snapshot)? {
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
) -> Result<bool, InputCutoffVerificationStop> {
    match fact.source_currentness_in(runtime, execution, snapshot)? {
        Err(WorthQuerySourceCurrentnessFailure::ManagedIndex(cause)) => Err(InputCutoffVerificationStop::OutputObservation(
            crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop::ManagedIndex(cause))),
        Err(WorthQuerySourceCurrentnessFailure::OutputObservation(stop)) => {
            Err(InputCutoffVerificationStop::OutputObservation(stop))
        }
        Ok(movement) => Ok(movement.movement() == Movement::Unmoved),
        Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
            Err(InputCutoffVerificationStop::Execution(cause))
        }
        Err(WorthQuerySourceCurrentnessFailure::Unavailable) => Ok(false),
        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
            Err(InputCutoffVerificationStop::WorkExhausted)
        }
    }
}
