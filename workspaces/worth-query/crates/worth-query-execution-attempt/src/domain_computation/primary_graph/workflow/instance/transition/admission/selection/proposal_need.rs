//! One compiled proposal-node meaning for publication and descriptive waiting.
use super::*;

pub struct SelectedWorkflowProposal<'compiled> {
    pub node_path: &'compiled str,
    pub operation: &'compiled str,
    pub input_type: &'compiled str,
}

pub fn select_current_proposal_need<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    progress_basis: &WorkflowTransitionProgressBasis,
) -> Result<Option<SelectedWorkflowProposal<'compiled>>, WorthQueryApplicationAttemptDenial> {
    let node = super::select_current_node(compiled, progress_basis.progress())?;
    Ok(SelectedWorkflowProposal::from_node(node))
}

impl<'compiled> SelectedWorkflowProposal<'compiled> {
    pub fn matches_recorded(
        &self,
        operation: &str,
        input_type: &str,
        node_path: &str,
    ) -> bool {
        self.operation == operation && self.input_type == input_type && self.node_path == node_path
    }
    pub fn from_node(
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
