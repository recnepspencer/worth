use worth_relational::facade::identity::EntityId;

use super::{SettledWorkflowTransition, WorkflowInstanceProgress};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowTransitionLocator {
    entity: EntityId,
    settlement: SettledWorkflowTransition,
}

impl WorkflowTransitionLocator {
    pub(in crate::domain_computation::primary_graph) const fn new(
        entity: EntityId,
        settlement: SettledWorkflowTransition,
    ) -> Self {
        Self { entity, settlement }
    }

    pub(in crate::domain_computation::primary_graph) const fn entity(self) -> EntityId {
        self.entity
    }

    pub(in crate::domain_computation::primary_graph) const fn settlement(
        self,
    ) -> SettledWorkflowTransition {
        self.settlement
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowAssessmentEvidenceLocator {
    transition: WorkflowTransitionLocator,
    evidence: EntityId,
}

impl WorkflowAssessmentEvidenceLocator {
    pub(in crate::domain_computation::primary_graph) const fn new(
        transition: WorkflowTransitionLocator,
        evidence: EntityId,
    ) -> Self {
        Self {
            transition,
            evidence,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn transition(
        self,
    ) -> WorkflowTransitionLocator {
        self.transition
    }

    pub(in crate::domain_computation::primary_graph) const fn evidence(self) -> EntityId {
        self.evidence
    }
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct WorkflowTransitionProgressObservation {
    transition: WorkflowTransitionLocator,
    assessment_evidence: Option<EntityId>,
}

impl WorkflowTransitionProgressObservation {
    pub(in crate::domain_computation::primary_graph) const fn new(
        transition: WorkflowTransitionLocator,
        assessment_evidence: Option<EntityId>,
    ) -> Self {
        Self {
            transition,
            assessment_evidence,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn transition(
        self,
    ) -> WorkflowTransitionLocator {
        self.transition
    }
}

impl WorkflowInstanceProgress {
    pub(in crate::domain_computation::primary_graph) fn latest_transition(
        &self,
        node: EntityId,
    ) -> Option<WorkflowTransitionLocator> {
        self.latest_transitions.get(&node).copied()
    }

    pub(in crate::domain_computation::primary_graph) fn latest_assessment_evidence(
        &self,
        node: EntityId,
    ) -> Option<WorkflowAssessmentEvidenceLocator> {
        self.latest_assessment_evidence.get(&node).copied()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_observation(
        &mut self,
        observation: WorkflowTransitionProgressObservation,
    ) {
        let locator = observation.transition;
        let node = locator.settlement().node();
        retain_latest(&mut self.latest_transitions, node, locator, |value| {
            value.settlement().occurrence()
        });
        if let Some(evidence) = observation.assessment_evidence {
            retain_latest(
                &mut self.latest_assessment_evidence,
                node,
                WorkflowAssessmentEvidenceLocator::new(locator, evidence),
                |value| value.transition().settlement().occurrence(),
            );
        }
    }
}

fn retain_latest<Value: Copy>(
    retained: &mut im::OrdMap<EntityId, Value>,
    node: EntityId,
    candidate: Value,
    occurrence: impl Fn(Value) -> u64,
) {
    match retained.get(&node).copied() {
        Some(current) if occurrence(current) >= occurrence(candidate) => {}
        _ => {
            retained.insert(node, candidate);
        }
    }
}
