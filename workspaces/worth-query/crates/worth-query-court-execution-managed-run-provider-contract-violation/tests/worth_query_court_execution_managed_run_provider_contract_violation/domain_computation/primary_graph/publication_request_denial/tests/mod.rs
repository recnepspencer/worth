use worth_execution::MapKernelStop;
use worth_execution::WorkCeilingDenial as Stop;
use worth_query_execution::court_doors_for_test::domain_computation::vocabulary::primary_graph::publication_request_denial::publication;
use worth_runtime_world::facade::NoEffectCause;
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
            NoEffectCause::CurrentReadDependency(worth_runtime_world::facade::CurrentProductReadDependencyDenial::WorldHeadUnavailable),
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
