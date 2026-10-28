//! Classify native no-effect custody before managed cleanup chooses a terminal.
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome as Outcome;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationNoEffect, WorthQueryApplicationNoEffectCause as QueryCause,
};
use worth_runtime_world::facade::{NoEffectCause as Cause, NoEffectCompositePublication};

pub(super) fn classify(terminal: NoEffectCompositePublication) -> Outcome {
    let cause = match terminal.cause() {
        Cause::StaleExpectedProductHead => {
            return Outcome::ProductStale(
                crate::domain_computation::WorthQueryProductStaleApplication::new(terminal),
            )
        }
        Cause::CancelledBeforeEffect
        | Cause::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled,
        )) => return Outcome::Cancelled,
        Cause::DeadlineBeforeEffect
        | Cause::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::DeadlineElapsed,
        )) => return Outcome::TimedOut,
        Cause::ExecutionRequest(cause) => QueryCause::ExecutionRequest(cause),
        Cause::OwnerDeniedBeforeEffect => QueryCause::OwnerDeniedBeforeEffect,
        Cause::CorrespondenceRebindRequired => QueryCause::CorrespondenceRebindRequired,
        Cause::ReferenceGenerationExhausted => QueryCause::ReferenceGenerationExhausted,
        Cause::CapacityExhausted => QueryCause::CapacityExhausted,
        Cause::OwnerUnavailable => QueryCause::OwnerUnavailable,
        Cause::RelationalDeferred(reason) => QueryCause::RelationalDeferred(reason),
        Cause::PreEffectFailure => QueryCause::PreEffectFailure,
    };
    Outcome::NoEffect(WorthQueryApplicationNoEffect::from_world(terminal, cause))
}
