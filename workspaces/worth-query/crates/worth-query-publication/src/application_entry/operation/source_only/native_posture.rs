//! Close eligibility follows terminal custody; fresh retry requires separate no-effect proof.
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
        SourceState::Stopped(outcome) => super::super::close::terminal_outcome_can_close(outcome),
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
