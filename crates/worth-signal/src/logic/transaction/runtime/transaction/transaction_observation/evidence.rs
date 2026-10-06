use std::collections::{BTreeMap, BTreeSet};

use crate::data::handle::NodeId;
use crate::logic::planner::ExecutionReport;

use super::super::super::state::{
    ObservationHandleId, ObservationPolicy, ObservationTrigger, ObservedNodeSet, ObserverId,
    RuntimeObservationRegistry,
};

use super::boundary::{
    CommittedObservationEventSummary, ObservationBoundaryOutcome, ObservationBoundarySummary,
};
use super::ordinary::{
    ClassifiedObservationEventSummary, CommittedObservationEvent, ObservationScratchSummary,
};

#[derive(Debug, Clone)]
struct StagedObservationCandidate {
    observer_id: ObserverId,
    handle_id: ObservationHandleId,
    policy: ObservationPolicy,
    observed_nodes: ObservedNodeSet,
    matched_nodes: BTreeSet<NodeId>,
}

impl StagedObservationCandidate {
    fn new(
        observer_id: ObserverId,
        handle_id: ObservationHandleId,
        policy: ObservationPolicy,
        observed_nodes: ObservedNodeSet,
    ) -> Self {
        Self {
            observer_id,
            handle_id,
            policy,
            observed_nodes,
            matched_nodes: BTreeSet::new(),
        }
    }

    fn stage_match(&mut self, node: NodeId) {
        self.matched_nodes.insert(node);
    }
}

#[derive(Debug, Clone)]
struct ClassifiedObservationEvent {
    observer_id: ObserverId,
    handle_id: ObservationHandleId,
    policy: ObservationPolicy,
    observed_nodes: ObservedNodeSet,
    matched_nodes: BTreeSet<NodeId>,
    visited: bool,
    recomputed: bool,
    meaningful_change: bool,
}

impl ClassifiedObservationEvent {
    fn from_candidate(candidate: &StagedObservationCandidate) -> Self {
        Self {
            observer_id: candidate.observer_id,
            handle_id: candidate.handle_id,
            policy: candidate.policy,
            observed_nodes: candidate.observed_nodes.clone(),
            matched_nodes: candidate.matched_nodes.clone(),
            visited: true,
            recomputed: false,
            meaningful_change: false,
        }
    }

    fn absorb_candidate(&mut self, candidate: &StagedObservationCandidate) {
        self.matched_nodes
            .extend(candidate.matched_nodes.iter().copied());
        self.visited = true;
    }

    fn mark_recomputed(&mut self) {
        self.recomputed = true;
    }

    fn mark_meaningful_change(&mut self) {
        self.meaningful_change = true;
    }

    fn mark_resource_lifecycle_change(&mut self) {
        self.visited = true;
        self.meaningful_change = true;
    }

    fn trigger_matched(&self) -> bool {
        match self.policy.trigger() {
            ObservationTrigger::Visited => self.visited,
            ObservationTrigger::Recomputed => self.recomputed,
            ObservationTrigger::MeaningfulChange => self.meaningful_change,
        }
    }

    fn summary(&self) -> ClassifiedObservationEventSummary {
        ClassifiedObservationEventSummary {
            observer_id: self.observer_id,
            handle_id: self.handle_id,
            policy: self.policy,
            observed_nodes: self.observed_nodes.clone(),
            matched_nodes: ObservedNodeSet::from_nodes(self.matched_nodes.iter().copied()),
            visited: self.visited,
            recomputed: self.recomputed,
            meaningful_change: self.meaningful_change,
            trigger_matched: self.trigger_matched(),
        }
    }

    fn committed_summary(
        &self,
        outcome: ObservationBoundaryOutcome,
    ) -> CommittedObservationEventSummary {
        CommittedObservationEventSummary {
            observer_id: self.observer_id,
            handle_id: self.handle_id,
            policy: self.policy,
            observed_nodes: self.observed_nodes.clone(),
            matched_nodes: ObservedNodeSet::from_nodes(self.matched_nodes.iter().copied()),
            visited: self.visited,
            recomputed: self.recomputed,
            meaningful_change: self.meaningful_change,
            trigger_matched: self.trigger_matched(),
            outcome,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::logic::transaction::runtime) struct TransactionObservationScratch {
    staged_candidates: BTreeMap<ObserverId, StagedObservationCandidate>,
    staged_observers_by_node: BTreeMap<NodeId, BTreeSet<ObserverId>>,
    classified_events: BTreeMap<ObserverId, ClassifiedObservationEvent>,
}

impl TransactionObservationScratch {
    pub fn stage_match<D, I, E, Ctx, T>(
        &mut self,
        observations: &RuntimeObservationRegistry<D, I, E, Ctx, T>,
        observer_id: ObserverId,
        node: NodeId,
    ) where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        T: Copy + Ord,
    {
        if let Some(candidate) = self.staged_candidates.get_mut(&observer_id) {
            self.staged_observers_by_node
                .entry(node)
                .or_default()
                .insert(observer_id);
            candidate.stage_match(node);
            return;
        }
        let Some((handle_id, policy, observed_nodes)) = observations.registration_for(observer_id)
        else {
            return;
        };
        self.staged_observers_by_node
            .entry(node)
            .or_default()
            .insert(observer_id);
        let candidate = self
            .staged_candidates
            .entry(observer_id)
            .or_insert_with(|| {
                StagedObservationCandidate::new(
                    observer_id,
                    handle_id,
                    policy,
                    observed_nodes.clone(),
                )
            });
        candidate.stage_match(node);
    }

    pub fn lower_classifications(
        &mut self,
        graph: &crate::data::graph::SignalGraph,
        report: &ExecutionReport,
    ) -> Result<(), crate::data::error::SignalError> {
        let mut runtime_facts = BTreeMap::<NodeId, (bool, bool)>::new();
        for record in report
            .stages
            .iter()
            .flat_map(|stage| stage.task_records.iter())
        {
            let entry = runtime_facts.entry(record.node).or_insert((false, false));
            entry.0 |= record.recomputed;
            if let Some(artifact) = graph.node_runtime_artifact_finalize_image(record.node)? {
                entry.1 |= artifact.output_change() != crate::data::output::OutputChange::Unchanged;
            }
        }

        for (&node, &(recomputed, meaningful_change)) in &runtime_facts {
            let Some(observer_ids) = self.staged_observers_by_node.get(&node) else {
                continue;
            };
            for &observer_id in observer_ids {
                let Some(candidate) = self.staged_candidates.get(&observer_id) else {
                    continue;
                };
                let classified = self
                    .classified_events
                    .entry(observer_id)
                    .or_insert_with(|| ClassifiedObservationEvent::from_candidate(candidate));
                classified.absorb_candidate(candidate);
                if recomputed {
                    classified.mark_recomputed();
                }
                if meaningful_change {
                    classified.mark_meaningful_change();
                }
            }
        }

        Ok(())
    }

    pub fn classify_resource_lifecycle_change<D, I, E, Ctx, T>(
        &mut self,
        observations: &RuntimeObservationRegistry<D, I, E, Ctx, T>,
        node: NodeId,
    ) -> usize
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        T: Copy + Ord,
    {
        let mut matched = 0;
        observations.for_each_matching_observer_for_node(node, |observer_id| {
            matched += 1;
            self.stage_match(observations, observer_id, node);
            let Some(candidate) = self.staged_candidates.get(&observer_id) else {
                return;
            };
            let classified = self
                .classified_events
                .entry(observer_id)
                .or_insert_with(|| ClassifiedObservationEvent::from_candidate(candidate));
            classified.absorb_candidate(candidate);
            classified.mark_resource_lifecycle_change();
        });
        matched
    }

    pub fn summary(&self) -> ObservationScratchSummary {
        ObservationScratchSummary {
            staged_candidate_observer_count: self.staged_candidates.len(),
            staged_candidate_match_count: self
                .staged_candidates
                .values()
                .map(|candidate| candidate.matched_nodes.len())
                .sum(),
            classified_event_count: self.classified_events.len(),
            visited_event_count: self
                .classified_events
                .values()
                .filter(|event| event.visited)
                .count(),
            recomputed_event_count: self
                .classified_events
                .values()
                .filter(|event| event.recomputed)
                .count(),
            meaningful_change_event_count: self
                .classified_events
                .values()
                .filter(|event| event.meaningful_change)
                .count(),
        }
    }

    pub fn classified_summaries(&self) -> Vec<ClassifiedObservationEventSummary> {
        self.classified_events
            .values()
            .map(ClassifiedObservationEvent::summary)
            .collect()
    }

    pub fn staged_candidate_count(&self) -> usize {
        self.staged_candidates.len()
    }

    pub fn classified_event_count(&self) -> usize {
        self.classified_events.len()
    }

    pub fn drain_delivery_boundary(
        &mut self,
        outcome: ObservationBoundaryOutcome,
    ) -> (Vec<CommittedObservationEvent>, ObservationBoundarySummary) {
        for (&observer_id, candidate) in &self.staged_candidates {
            self.classified_events
                .entry(observer_id)
                .or_insert_with(|| ClassifiedObservationEvent::from_candidate(candidate));
        }
        let classified_events = std::mem::take(&mut self.classified_events);
        let classified_event_count = classified_events.len() as u32;
        let trigger_matched_event_count = classified_events
            .values()
            .filter(|event| event.trigger_matched())
            .count() as u32;

        let boundary_events = classified_events
            .values()
            .filter(|event| event.trigger_matched())
            .map(|event| event.committed_summary(outcome))
            .collect::<Vec<_>>();

        let deliveries = boundary_events
            .iter()
            .cloned()
            .map(CommittedObservationEvent::new)
            .collect::<Vec<_>>();

        let delivered_event_count = if matches!(outcome, ObservationBoundaryOutcome::Delivered) {
            boundary_events.len() as u32
        } else {
            0
        };
        let rollback_suppressed_event_count =
            if matches!(outcome, ObservationBoundaryOutcome::RollbackSuppressed) {
                boundary_events.len() as u32
            } else {
                0
            };
        let branch_local_suppressed_event_count =
            if matches!(outcome, ObservationBoundaryOutcome::BranchLocalSuppressed) {
                boundary_events.len() as u32
            } else {
                0
            };

        self.staged_candidates.clear();
        self.staged_observers_by_node.clear();

        (
            deliveries,
            ObservationBoundarySummary {
                classified_event_count,
                trigger_matched_event_count,
                delivered_event_count,
                rollback_suppressed_event_count,
                branch_local_suppressed_event_count,
                boundary_events,
            },
        )
    }
}
