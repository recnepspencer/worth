use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDataFlow, ApplicationWorkflowSubjectSelector,
};
use worth_relational::facade::identity::EntityId;

use super::{CompiledWorkflowDefinition, CompiledWorkflowNodeKind};

impl CompiledWorkflowDefinition {
    pub(in crate::domain_computation::primary_graph) fn proposal_coverage_selectors(
        &self,
        proposal: EntityId,
    ) -> Result<Vec<ApplicationWorkflowSubjectSelector>, ()> {
        let mut selectors = self
            .data_targets(proposal, ApplicationWorkflowDataFlow::AssessmentSubject)
            .map(|node| {
                match node.kind() {
                    CompiledWorkflowNodeKind::Assessment { subject, .. } => Some(subject.clone()),
                    _ => None,
                }
                .ok_or(())
            })
            .collect::<Result<Vec<_>, _>>()?;
        selectors.sort();
        selectors.dedup();
        Ok(selectors)
    }
}
