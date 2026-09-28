//! One lineage spends one step budget: the steps an instance inherited from
//! its migration sources and its own. Only a request that writes a transition
//! spends it; closing writes none.

use super::history::HistoryBasis;
use super::{
    ObservedWorkflowInstance, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};

/// What a request observes a live instance for. Advancing writes a new
/// transition and needs a free slot of the retained capacity; closing writes
/// none, so an instance that has used its whole capacity can still close.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt) enum WorkflowInstanceObservationPurpose
{
    Advance,
    Close,
}

/// Leave to write one step, granted only by
/// [`ObservedWorkflowInstance::ensure_step_left`]. Transition and migration
/// admission each take one, so no step reaches a commit without passing the
/// budget check. A settled or closing observation passes it without spending,
/// and the allowance names no instance, so each caller asks for it from the
/// observation of the instance it admits.
#[must_use = "a step is admitted only with the allowance its budget check granted"]
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowStepAllowance(());

impl WorkflowStepAllowance {
    /// Consumes the allowance for the one step being admitted.
    pub(in crate::domain_computation::primary_graph) fn spend(self) {
        let Self(()) = self;
    }
}

impl ObservedWorkflowInstance {
    /// A step spends one of the lineage's budget, which counts the steps the
    /// instance inherited from its sources as well as its own. Warm and cold
    /// reads refuse a spent budget alike, before any commit. The caller asks
    /// where a retry of a step already recorded can still replay, so the step
    /// that spent the last of the budget still answers.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn ensure_step_left(
        &self,
    ) -> Result<WorkflowStepAllowance, WorthQueryApplicationAttemptDenial> {
        let lineage_steps = usize::try_from(self.lineage_steps()).unwrap_or(usize::MAX);
        if self.live_membership.is_some()
            && self.purpose == WorkflowInstanceObservationPurpose::Advance
            && lineage_steps >= self.maximum_transitions
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
                "workflow instance transition capacity is exhausted",
            ));
        }
        Ok(WorkflowStepAllowance(()))
    }
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
