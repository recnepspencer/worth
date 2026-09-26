use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

use super::{
    denial, CompiledWorkflowDefinition, CompiledWorkflowNodeKind, SettledWorkflowTransition,
    WorkflowInstanceProgress, WorkflowTransitionProgressObservation,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorkflowInstanceProgress {
    pub(in crate::domain_computation::primary_graph) fn reconstruct(
        compiled: &CompiledWorkflowDefinition,
        observations: &mut [WorkflowTransitionProgressObservation],
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        observations
            .sort_unstable_by_key(|observation| observation.transition().settlement().occurrence());
        let mut progress = Self::start(compiled);
        for observation in observations {
            progress.apply_observation(compiled, *observation)?;
        }
        Ok(progress)
    }

    pub(in crate::domain_computation::primary_graph) fn apply_observation(
        &mut self,
        compiled: &CompiledWorkflowDefinition,
        observation: WorkflowTransitionProgressObservation,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let settled = observation.transition().settlement();
        if settled.node() == self.head {
            self.advance(compiled, settled)?;
        } else {
            if observation.assessment_evidence().is_none() {
                return Err(denial(
                    "off-head assessment collection has no linked evidence",
                ));
            }
            self.collect_assessment(compiled, settled)?;
        }
        self.retain_observation(observation);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn collect_assessment(
        &mut self,
        compiled: &CompiledWorkflowDefinition,
        transition: SettledWorkflowTransition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if transition.occurrence() != self.next_occurrence
            || transition.node() == self.head
            || transition.outcome() != ApplicationWorkflowControlOutcome::Completed
            || transition.operation_receipt_identity().is_some()
        {
            return Err(denial(
                "assessment collection is not a distinct next occurrence",
            ));
        }
        let Some(node) = compiled.node(transition.node()) else {
            return Err(denial(
                "assessment collection node is absent from the definition",
            ));
        };
        if !matches!(node.kind(), CompiledWorkflowNodeKind::Assessment { .. }) {
            return Err(denial(
                "off-head collection does not name an authored assessment",
            ));
        }
        let mut sources = compiled.assessment_subject_sources(transition.node());
        let Some(source) = sources.next() else {
            return Err(denial(
                "assessment collection has no declared proposal source",
            ));
        };
        if sources.next().is_some() || self.latest_transition(source.entity()).is_none() {
            return Err(denial("assessment collection proposal source is not ready"));
        }
        self.next_occurrence = self.next_occurrence.checked_add(1).ok_or_else(|| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
                "workflow collection history exceeds supported occurrence range",
            )
        })?;
        Ok(())
    }
}
