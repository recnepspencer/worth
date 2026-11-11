//! Advance the same actual World phases under the original request ceiling.
use super::super::durable_custody::{QueryDurableOwnerCall as Call, QueryNativeCustody as Native};
use super::*;
mod native_binding;
mod native_completion;
mod native_preparation;
mod opening;
mod publication;
use worth_runtime_world::facade::{WorldDurablePublicationOutcome, WorldOpenAttemptOutcome};

pub(super) enum Progress {
    Terminal,
    Pending(Stage),
}
fn take_world(guard: &mut QueryDurablePreparationGuard, call: Call) -> World {
    std::mem::replace(
        &mut guard.preparation_mut().world,
        World::OwnerCallInProgress(call),
    )
}
pub(super) fn run(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    guard: &mut QueryDurablePreparationGuard,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<Progress, worth_execution::WorkCeilingDenial> {
    // The concrete guard lives outside both scopes, including unwind containment.
    execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    advance(execution, provider, guard, allocation, context)
                })
        })?
        .map(|(outcome, _)| outcome)?
}
// The retained guard, not this driver's stack, owns every actual phase.
// Each owner call enters only its own stage frame; checkpoint order is unchanged.
fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    guard: &mut QueryDurablePreparationGuard,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Progress, worth_execution::WorkCeilingDenial> {
    if let Some(outcome) = opening::advance(execution, provider, guard, context)? {
        return Ok(outcome);
    }
    if let Some(outcome) = native_binding::advance(execution, guard, context)? {
        return Ok(outcome);
    }
    if let Some(outcome) =
        native_preparation::advance(execution, provider, guard, allocation, context)?
    {
        return Ok(outcome);
    }
    if let Some(outcome) = native_completion::advance(execution, provider, guard, context)? {
        return Ok(outcome);
    }
    publication::advance(execution, guard, context)
}
