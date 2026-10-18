//! One compiled proposal-node meaning for publication and descriptive waiting.
use super::*;

pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowProposal<'compiled> {
    pub(in crate::domain_computation::primary_graph) node_path: &'compiled str,
    pub(in crate::domain_computation::primary_graph) operation: &'compiled str,
    pub(in crate::domain_computation::primary_graph) input_type: &'compiled str,
}

pub(in crate::domain_computation::primary_graph) fn select_current_proposal_need<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    progress_basis: &WorkflowTransitionProgressBasis,
) -> Result<Option<SelectedWorkflowProposal<'compiled>>, WorthQueryApplicationAttemptDenial> {
    let node = super::select_current_node(compiled, progress_basis.progress())?;
    Ok(SelectedWorkflowProposal::from_node(node))
}

impl<'compiled> SelectedWorkflowProposal<'compiled> {
    pub(in crate::domain_computation::primary_graph) fn matches_recorded(
        &self,
        operation: &str,
        input_type: &str,
        node_path: &str,
    ) -> bool {
        self.operation == operation && self.input_type == input_type && self.node_path == node_path
    }
    pub(in crate::domain_computation::primary_graph) fn from_node(
        node: &'compiled CompiledWorkflowNode,
    ) -> Option<Self> {
        match node.kind() {
            CompiledWorkflowNodeKind::Operation {
                operation,
                input_type,
                requires_workflow_authority: false,
                ..
            } => Some(SelectedWorkflowProposal {
                node_path: node.path(),
                operation,
                input_type,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectedWorkflowProposal;
    #[test]
    fn recorded_proposal_descriptor_must_match_selected_authored_meaning() {
        // Pure borrowed descriptor, not a grant, prepared transition or fabricated runtime owner.
        let selected = SelectedWorkflowProposal {
            node_path: "propose",
            operation: "author-document",
            input_type: "author-document-input",
        };
        assert!(selected.matches_recorded("author-document", "author-document-input", "propose"));
        assert!(!selected.matches_recorded("set-retention", "author-document-input", "propose"));
        assert!(!selected.matches_recorded("author-document", "set-retention-input", "propose"));
        assert!(!selected.matches_recorded(
            "author-document",
            "author-document-input",
            "other-node"
        ));
    }
}
