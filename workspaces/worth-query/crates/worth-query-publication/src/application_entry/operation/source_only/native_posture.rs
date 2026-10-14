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
        SourceState::AdmissionStopped(denial) => pinned_source_moved(denial),
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
