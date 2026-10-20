//! Explicit action routes remain separate from automatic control and historical Back.
use super::{CompiledWorkflowConnectionKind, CompiledWorkflowDefinition};
use worth_relational::facade::identity::EntityId;
impl CompiledWorkflowDefinition {
    pub(in crate::domain_computation::primary_graph) fn has_back_action(
        &self,
        source: EntityId,
    ) -> bool {
        self.outgoing(source).any(|edge| {
            matches!(
                self.publication.connections[edge.connection].kind.as_ref(),
                CompiledWorkflowConnectionKind::BackAction { .. }
            )
        })
    }
    pub(in crate::domain_computation::primary_graph) fn back_action(&self, source: EntityId) -> Result<(&super::CompiledWorkflowNode, &CompiledWorkflowConnectionKind), crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial>{
        let mut actions = self.outgoing(source).filter_map(|edge| {
            let kind = self.publication.connections[edge.connection].kind.as_ref();
            matches!(kind, CompiledWorkflowConnectionKind::BackAction { .. })
                .then_some((&self.publication.nodes[edge.peer], kind))
        });
        let action = actions.next();
        match (action, actions.next()) {
            (Some(action), None) => Ok(action),
            _ => Err(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                "workflow head has no single declared business Back action")),
        }
    }
}
