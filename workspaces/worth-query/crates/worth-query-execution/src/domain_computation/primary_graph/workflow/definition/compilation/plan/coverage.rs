use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDataFlow, ApplicationWorkflowSubjectSelector,
};
use worth_relational::facade::identity::EntityId;

use super::{CompiledWorkflowConnectionKind, CompiledWorkflowDefinition, CompiledWorkflowNodeKind};

impl CompiledWorkflowDefinition {
    pub(in crate::domain_computation::primary_graph) fn proposal_coverage_selectors(
        &self,
        proposal: EntityId,
    ) -> Result<Vec<ApplicationWorkflowSubjectSelector>, ()> {
        let mut selectors = self
            .connections
            .iter()
            .filter_map(|connection| match connection.kind {
                CompiledWorkflowConnectionKind::Data(
                    ApplicationWorkflowDataFlow::AssessmentSubject,
                ) if connection.source == proposal => Some(connection.target),
                _ => None,
            })
            .map(|target| {
                self.nodes
                    .iter()
                    .find(|node| node.entity == target)
                    .and_then(|node| match &node.kind {
                        CompiledWorkflowNodeKind::Assessment { subject, .. } => {
                            Some(subject.clone())
                        }
                        _ => None,
                    })
                    .ok_or(())
            })
            .collect::<Result<Vec<_>, _>>()?;
        selectors.sort();
        selectors.dedup();
        Ok(selectors)
    }
}
