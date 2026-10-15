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
    Ok(proposal_node(node))
}

pub(super) fn proposal_node(node: &CompiledWorkflowNode) -> Option<SelectedWorkflowProposal<'_>> {
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
