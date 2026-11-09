//! World publication stop classification shared by the physical cutover.
pub(super) fn publication_request_stop(
    cause: worth_execution::WorkCeilingDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::WorthQueryProviderSessionCommitControlStopped;
    use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
    use crate::domain_computation::WorthQueryProviderSessionFailure;
    use crate::domain_computation::WorthQueryProviderSessionProtocolStage;
    match crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause) {
        Ok(kind) => Stop::PreEffectDenied(WorthQueryProviderSessionFailure::checked_read_stopped(Ok(kind), WorthQueryProviderSessionProtocolStage::Commit, "World publication request stopped before effect")),
        Err(control) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(control, "World publication request stopped before effect")),
    }
}

pub(super) fn world_no_effect(
    no_effect: worth_runtime_world::facade::NoEffectCompositePublication,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::WorthQueryProviderSessionCommitControlStopped;
    use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind;
    use worth_runtime_world::facade::NoEffectCause;
    match no_effect.cause() {
        NoEffectCause::StaleExpectedProductHead => Stop::ProductStale(
            crate::domain_computation::WorthQueryProductStaleApplication::new(no_effect),
        ),
        NoEffectCause::CancelledBeforeEffect
        | NoEffectCause::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled,
        )) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
            WorthQueryProviderSessionControlStopKind::Cancelled,
            "World publication cancelled before effect",
        )),
        NoEffectCause::DeadlineBeforeEffect
        | NoEffectCause::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::DeadlineElapsed,
        )) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
            WorthQueryProviderSessionControlStopKind::TimedOut,
            "World publication deadline elapsed before effect",
        )),
        NoEffectCause::ExecutionRequest(_)
        | NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CurrentReadDependency(_)
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::CapacityExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::RelationalDeferred(_)
        | NoEffectCause::PreEffectFailure => Stop::NoEffect(no_effect),
    }
}
