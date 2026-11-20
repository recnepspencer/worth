use std::sync::Arc;

use super::{
    WorthQueryConvergenceComparison, WorthQueryConvergenceDomainDecision,
    WorthQueryConvergenceProgress, WorthQueryConvergenceRepeatedState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceDomainWorkEvidence {
    comparator_call_count: usize,
    progress_check_count: usize,
    repeated_state_probe_count: usize,
}

impl WorthQueryConvergenceDomainWorkEvidence {
    pub(super) const fn empty() -> Self {
        Self {
            comparator_call_count: 0,
            progress_check_count: 0,
            repeated_state_probe_count: 0,
        }
    }

    pub(super) fn called_comparator(&mut self) {
        self.comparator_call_count += 1;
    }

    pub(super) fn checked_progress(&mut self) {
        self.progress_check_count += 1;
    }

    pub(super) fn probed_repeated_state(&mut self) {
        self.repeated_state_probe_count += 1;
    }

    pub const fn comparator_call_count(&self) -> usize {
        self.comparator_call_count
    }

    pub const fn progress_check_count(&self) -> usize {
        self.progress_check_count
    }

    pub const fn repeated_state_probe_count(&self) -> usize {
        self.repeated_state_probe_count
    }
}

pub(super) struct WorthQueryConvergenceDomainAssessmentOutcome {
    decision: WorthQueryConvergenceDomainDecision,
    work: WorthQueryConvergenceDomainWorkEvidence,
}

impl WorthQueryConvergenceDomainAssessmentOutcome {
    pub(super) fn new(
        comparison: WorthQueryConvergenceComparison,
        progress: WorthQueryConvergenceProgress,
        repeated_state: WorthQueryConvergenceRepeatedState,
        work: WorthQueryConvergenceDomainWorkEvidence,
    ) -> Self {
        Self {
            decision: WorthQueryConvergenceDomainDecision::from_governed_assessment(
                comparison,
                progress,
                repeated_state,
            ),
            work,
        }
    }

    pub(super) fn decision(&self) -> &WorthQueryConvergenceDomainDecision {
        &self.decision
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryConvergenceDomainDecision,
        WorthQueryConvergenceDomainWorkEvidence,
    ) {
        (self.decision, self.work)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceDomainFailure {
    detail: Arc<str>,
}

impl WorthQueryConvergenceDomainFailure {
    pub fn new(detail: impl Into<Arc<str>>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
