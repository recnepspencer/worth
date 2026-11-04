//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn back_edge_iterations_for_test(&self) -> &OrdMap<(EntityId, EntityId), u64> {
        &self.back_edge_iterations
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn latest_assessment_evidence_for_test(
        &self,
    ) -> &OrdMap<EntityId, WorkflowAssessmentEvidenceLocator> {
        &self.latest_assessment_evidence
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn latest_transitions_for_test(&self) -> &OrdMap<EntityId, WorkflowTransitionLocator> {
        &self.latest_transitions
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn path_for_test(&self) -> &OrdMap<u64, WorkflowPathFrame> {
        &self.path
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn path_depth_for_test(&self) -> &u64 {
        &self.path_depth
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn retry_counts_for_test(
        &self,
    ) -> &OrdMap<(EntityId, ApplicationWorkflowControlOutcome), usize> {
        &self.retry_counts
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn advance_with_for_test(
        &mut self,
        transition: SettledWorkflowTransition,
        successor: &mut impl FnMut(
            EntityId,
            ApplicationWorkflowControlOutcome,
            usize,
        ) -> Result<EntityId, WorthQueryApplicationAttemptDenial>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::advance_with(self, transition, successor)
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn head_for_test(&self) -> EntityId {
        Self::head(self)
    }
}

impl SettledWorkflowTransition {
    #[doc(hidden)]
    pub fn new_for_test(
        node: EntityId,
        occurrence: u64,
        outcome: ApplicationWorkflowControlOutcome,
        operation_receipt_identity: Option<[u8; 32]>,
    ) -> Self {
        Self::new(node, occurrence, outcome, operation_receipt_identity)
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn next_occurrence_for_test(&self) -> u64 {
        Self::next_occurrence(self)
    }
}

impl WorkflowInstanceProgress {
    #[doc(hidden)]
    pub fn retained_charge_bytes_for_test(&self) -> usize {
        Self::retained_charge_bytes(self)
    }
}
