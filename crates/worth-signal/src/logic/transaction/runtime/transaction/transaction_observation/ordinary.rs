use super::super::super::state::{
    ObservationHandleId, ObservationPolicy, ObservedNodeSet, ObserverId,
};
use super::boundary::CommittedObservationEventSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationScratchSummary {
    pub staged_candidate_observer_count: usize,
    pub staged_candidate_match_count: usize,
    pub classified_event_count: usize,
    pub visited_event_count: usize,
    pub recomputed_event_count: usize,
    pub meaningful_change_event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedObservationEventSummary {
    pub observer_id: ObserverId,
    pub handle_id: ObservationHandleId,
    pub policy: ObservationPolicy,
    pub observed_nodes: ObservedNodeSet,
    pub matched_nodes: ObservedNodeSet,
    pub visited: bool,
    pub recomputed: bool,
    pub meaningful_change: bool,
    pub trigger_matched: bool,
}

#[derive(Debug, Clone)]
pub(in crate::logic::transaction::runtime) struct CommittedObservationEvent {
    summary: CommittedObservationEventSummary,
}

impl CommittedObservationEvent {
    pub(super) fn new(summary: CommittedObservationEventSummary) -> Self {
        Self { summary }
    }

    pub fn observer_id(&self) -> ObserverId {
        self.summary.observer_id
    }

    pub fn handle_id(&self) -> ObservationHandleId {
        self.summary.handle_id
    }

    pub fn policy(&self) -> ObservationPolicy {
        self.summary.policy
    }

    pub fn observed_nodes(&self) -> &ObservedNodeSet {
        &self.summary.observed_nodes
    }

    pub fn matched_nodes(&self) -> &ObservedNodeSet {
        &self.summary.matched_nodes
    }

    pub fn visited(&self) -> bool {
        self.summary.visited
    }

    pub fn recomputed(&self) -> bool {
        self.summary.recomputed
    }

    pub fn meaningful_change(&self) -> bool {
        self.summary.meaningful_change
    }

    pub fn trigger_matched(&self) -> bool {
        self.summary.trigger_matched
    }
}
