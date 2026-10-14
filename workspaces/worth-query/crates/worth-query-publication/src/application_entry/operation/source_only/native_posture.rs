//! Close classification follows native no-effect custody, never posture names alone.
use super::*;
use worth_query_execution::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
};

pub(super) fn pinned_source_moved(denial: &WorthQueryApplicationRequestMutationDenial) -> bool {
    matches!(denial,
        WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::Attempt(cause))
        if matches!(cause.kind(), WorthQueryApplicationAttemptDenialKind::SourceChanged
            | WorthQueryApplicationAttemptDenialKind::SourceRetired))
}

pub(super) fn can_close<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
    state: &SourceState<S, I>,
) -> bool {
    match state {
        SourceState::Initial => true,
        SourceState::AdmissionStopped(denial) => pinned_source_moved(denial),
        SourceState::RetryableNoEffect(outcome) => fresh_retry_eligible::<S, I>(outcome),
        SourceState::Stopped(outcome) => matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(_)
                | WorthQueryApplicationMutationOutcome::Cancelled
                | WorthQueryApplicationMutationOutcome::DeadlineExceeded
                | WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
                | WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::Denied(_)
                        | WorthQueryApplicationUncommitted::Cancelled
                        | WorthQueryApplicationUncommitted::TimedOut
                        | WorthQueryApplicationUncommitted::Stale(_)
                        // Both own no-effect observations, released by their native Drop owners.
                        | WorthQueryApplicationUncommitted::ProductStale(_)
                        | WorthQueryApplicationUncommitted::NoEffect(_)
                )
        ),
        _ => false,
    }
}

// Only final no-effect outcomes whose managed completion succeeded qualify.
// Failed Stale/Denied cleanup is returned natively as Indeterminate instead.
pub(super) fn fresh_retry_eligible<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
    outcome: &NativeOutcome<S, I>,
) -> bool {
    use worth_query_execution::facade::primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationCommitDenialStage as Stage,
    };
    match outcome {
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Stale(_)) => true,
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(cause)) =>
            cause.kind() == Kind::ProductBasisStale
                && cause.stage() == Stage::InvariantExecution
                && cause.invariant_execution_failure().is_some_and(|failure| {
                    failure.kind() == worth_query_execution::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::ProductBasisStale
                }),
        _ => false,
    }
}
