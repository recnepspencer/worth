//! Moved whole from `domain_computation::primary_graph::publication_request_denial`: the items below and every impl they own.
use worth_runtime_world::facade::NoEffectCause;

pub fn publication(
    cause: NoEffectCause,
) -> Result<NoEffectCause, worth_execution::WorkCeilingDenial> {
    use worth_execution::{MapKernelStop, WorkCeilingDenial as Stop};
    match cause {
        NoEffectCause::CancelledBeforeEffect => Err(Stop::Stopped(MapKernelStop::Cancelled)),
        NoEffectCause::DeadlineBeforeEffect => Err(Stop::Stopped(MapKernelStop::DeadlineElapsed)),
        NoEffectCause::ExecutionRequest(cause) => Err(cause),
        ordinary @ (NoEffectCause::StaleExpectedProductHead
        | NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CurrentReadDependency(_)
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::CapacityExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::RelationalDeferred(_)
        | NoEffectCause::PreEffectFailure) => Ok(ordinary),
    }
}
