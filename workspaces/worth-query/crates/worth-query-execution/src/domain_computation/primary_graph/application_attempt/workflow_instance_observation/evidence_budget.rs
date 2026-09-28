//! One lineage retains one evidence budget: the assessment evidence bytes an
//! instance inherited from its migration sources and those its own history
//! retains. Evidence is never released while its history is retained, so
//! Back, a yield or a duplicate delivery neither spends nor restores it.

use super::{ObservedWorkflowInstance, ObservedWorkflowTransition};
use crate::domain_computation::primary_graph::workflow::instance::{
    WorkflowTransitionLocator, WorkflowTransitionProgressObservation,
};

impl ObservedWorkflowInstance {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn lineage_evidence_bytes(
        &self,
    ) -> u64 {
        self.inherited_evidence_bytes
            .saturating_add(self.progress_basis.progress().retained_evidence_bytes())
    }

    /// The evidence bytes a further assessment may still retain under the
    /// installed ceiling.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn evidence_allowance(
        &self,
        maximum_evidence_bytes: u64,
    ) -> u64 {
        maximum_evidence_bytes.saturating_sub(self.lineage_evidence_bytes())
    }
}

impl ObservedWorkflowTransition {
    pub(super) fn progress_observation(&self) -> WorkflowTransitionProgressObservation {
        let evidence = self.assessment_evidence.as_ref();
        WorkflowTransitionProgressObservation::new(
            WorkflowTransitionLocator::new(self.entity, self.settlement),
            evidence.map(|evidence| evidence.entity),
        )
        .retaining_evidence_bytes(evidence.map_or(0, |evidence| evidence.retained_bytes))
    }
}
