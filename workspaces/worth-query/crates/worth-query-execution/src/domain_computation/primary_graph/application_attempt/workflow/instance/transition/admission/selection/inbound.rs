use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

use super::{
    denial, CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
    SelectedWorkflowInbound, WorkflowInstanceProgress, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};

pub(super) fn select_inbound(
    compiled: &CompiledWorkflowDefinition,
    node: &CompiledWorkflowNode,
    origin: &str,
    progress: &WorkflowInstanceProgress,
) -> Result<SelectedWorkflowInbound, WorthQueryApplicationAttemptDenial> {
    let [origin_node] = compiled.nodes_with_path(origin) else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            node.path(),
        ));
    };
    if !matches!(
        origin_node.kind(),
        CompiledWorkflowNodeKind::Operation { .. }
    ) {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            node.path(),
        ));
    }
    let Some(origin_receipt_identity) = progress
        .latest_transition(origin_node.entity())
        .map(|transition| transition.settlement())
        .filter(|settlement| {
            settlement.outcome() == ApplicationWorkflowControlOutcome::Completed
                && settlement.occurrence() < progress.next_occurrence()
        })
        .and_then(|settlement| settlement.operation_receipt_identity())
    else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled,
            node.path(),
        ));
    };
    Ok(SelectedWorkflowInbound {
        origin_receipt_identity,
    })
}
