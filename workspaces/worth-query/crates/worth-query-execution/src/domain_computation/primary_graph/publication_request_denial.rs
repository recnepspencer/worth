//! Request stops carried by World's no-effect publication outcomes.
use worth_runtime_world::facade::NoEffectCause;
pub(crate) fn publication(
    cause: NoEffectCause,
) -> Result<NoEffectCause, worth_execution::WorkCeilingDenial> {
    use worth_execution::{MapKernelStop, WorkCeilingDenial as Stop};
    match cause {
        NoEffectCause::CancelledBeforeEffect => Err(Stop::Stopped(MapKernelStop::Cancelled)),
        NoEffectCause::DeadlineBeforeEffect => Err(Stop::Stopped(MapKernelStop::DeadlineElapsed)),
        NoEffectCause::ExecutionRequest(cause) => Err(cause),
        ordinary @ (NoEffectCause::StaleExpectedProductHead
        | NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::CapacityExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::RelationalDeferred(_)
        | NoEffectCause::PreEffectFailure) => Ok(ordinary),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::{MapKernelStop, WorkCeilingDenial as Stop};
    #[test]
    fn publication_stop_classification_preserves_control_and_ordinary_causes() {
        for (cause, expected) in [
            (
                NoEffectCause::CancelledBeforeEffect,
                MapKernelStop::Cancelled,
            ),
            (
                NoEffectCause::DeadlineBeforeEffect,
                MapKernelStop::DeadlineElapsed,
            ),
        ] {
            assert_eq!(publication(cause), Err(Stop::Stopped(expected)));
        }
        for stop in [
            Stop::Stopped(MapKernelStop::Cancelled),
            Stop::Stopped(MapKernelStop::DeadlineElapsed),
            Stop::Stopped(MapKernelStop::WorkCeiling),
            Stop::Admission(worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity {
                    requested: 43,
                    admitted: 17,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 2 },
                },
            )),
        ] {
            assert_eq!(
                publication(NoEffectCause::ExecutionRequest(stop)),
                Err(stop)
            );
        }
        for cause in [
            NoEffectCause::StaleExpectedProductHead,
            NoEffectCause::OwnerDeniedBeforeEffect,
            NoEffectCause::CorrespondenceRebindRequired,
            NoEffectCause::ReferenceGenerationExhausted,
            NoEffectCause::CapacityExhausted,
            NoEffectCause::OwnerUnavailable,
            NoEffectCause::PreEffectFailure,
            NoEffectCause::RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred::PatchPositionReservationContended),
        ] {
            assert_eq!(publication(cause), Ok(cause));
        }
    }
}
