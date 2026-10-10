//! Typed outcomes for refused comparison execution.
use super::*;

pub(super) fn invalid_execution_resource() -> WorthQueryComparisonOutcome {
    stopped(
        WorthQueryComparisonStopSource::InvalidBasisPair,
        WorthQueryComparisonNextAction::RefreshBasisPair,
        "execution resources do not match the structurally bound comparison family",
        WorthQueryComparisonJourneyCounters::validate_pair(),
    )
}

pub(super) fn stale_pair(
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    stopped(
        WorthQueryComparisonStopSource::StaleBasisPair,
        WorthQueryComparisonNextAction::RefreshBasisPair,
        "one or both structurally bound comparison bases are stale or mismatched",
        counters,
    )
}

pub(super) fn stopped(
    source: WorthQueryComparisonStopSource,
    next_action: WorthQueryComparisonNextAction,
    reason: impl Into<String>,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    WorthQueryComparisonOutcome::Stopped(WorthQueryComparisonStop::new(
        source,
        next_action,
        reason,
        counters,
    ))
}
