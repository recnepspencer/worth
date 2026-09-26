//! One lineage spends one step budget: the steps an instance inherited from
//! its migration sources and its own. Only a request that writes a transition
//! spends it; closing writes none.

use super::history::HistoryBasis;
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
use crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionProgressBasis;

/// What a request observes a live instance for. Advancing writes a new
/// transition and needs a free slot of the retained capacity; closing writes
/// none, so an instance that has used its whole capacity can still close.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt) enum WorkflowInstanceObservationPurpose
{
    Advance,
    Close,
}

/// A step spends one of the lineage's budget, which counts the steps the
/// instance inherited from its sources as well as its own. Warm and cold
/// reads refuse a spent budget alike, before any commit.
pub(super) fn ensure_step_left(
    purpose: WorkflowInstanceObservationPurpose,
    inherited_steps: u64,
    progress_basis: &WorkflowTransitionProgressBasis,
    maximum_transitions: usize,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let lineage_steps = usize::try_from(
        inherited_steps.saturating_add(progress_basis.progress().next_occurrence()),
    )
    .unwrap_or(usize::MAX);
    if purpose == WorkflowInstanceObservationPurpose::Advance
        && lineage_steps >= maximum_transitions
    {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
            "workflow instance transition capacity is exhausted",
        ));
    }
    Ok(())
}

pub(super) fn history_basis(
    live: bool,
    purpose: WorkflowInstanceObservationPurpose,
    inherited_steps: u64,
) -> HistoryBasis {
    match (live, purpose) {
        (false, _) => HistoryBasis::Ended,
        (true, WorkflowInstanceObservationPurpose::Advance) => HistoryBasis::Advance {
            inherited_steps: usize::try_from(inherited_steps).unwrap_or(usize::MAX),
        },
        (true, WorkflowInstanceObservationPurpose::Close) => HistoryBasis::Close,
    }
}
