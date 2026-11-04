//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn latest_transition_for_test(&self, node: EntityId) -> Option<WorkflowTransitionLocator> {
        Self::latest_transition(self, node)
    }
}

impl WorkflowTransitionLocator {
    #[doc(hidden)]
    pub fn new_for_test(entity: EntityId, settlement: SettledWorkflowTransition) -> Self {
        Self::new(entity, settlement)
    }
}

impl WorkflowTransitionProgressObservation {
    #[doc(hidden)]
    pub fn new_for_test(
        transition: WorkflowTransitionLocator,
        assessment_evidence: Option<EntityId>,
    ) -> Self {
        Self::new(transition, assessment_evidence)
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn retain_observation_for_test(
        &mut self,
        observation: WorkflowTransitionProgressObservation,
    ) {
        Self::retain_observation(self, observation)
    }
}
