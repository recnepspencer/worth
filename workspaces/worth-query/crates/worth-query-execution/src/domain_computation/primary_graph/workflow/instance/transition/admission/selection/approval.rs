use super::denial;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowApproval {
    pub(in crate::domain_computation::primary_graph) capability: String,
    pub(in crate::domain_computation::primary_graph) capability_type: String,
    pub(in crate::domain_computation::primary_graph) operation: String,
    pub(in crate::domain_computation::primary_graph) installed_capability_identity: String,
    pub(in crate::domain_computation::primary_graph) target_operation: String,
}

pub(super) fn select_approval(
    compiled: &CompiledWorkflowDefinition,
    node: &CompiledWorkflowNode,
    capability: &str,
    capability_type: &str,
    operation: &str,
    installed_capability_identity: &str,
) -> Result<SelectedWorkflowApproval, WorthQueryApplicationAttemptDenial> {
    let mut targets = compiled.approval_authority_targets(node.entity());
    let target = match (targets.next(), targets.next()) {
        (Some(target), None) => target,
        _ => return Err(invalid_target()),
    };
    let CompiledWorkflowNodeKind::Operation {
        operation: target_operation,
        requires_workflow_authority: true,
        ..
    } = target.kind()
    else {
        return Err(invalid_target());
    };
    Ok(SelectedWorkflowApproval {
        capability: capability.to_owned(),
        capability_type: capability_type.to_owned(),
        operation: operation.to_owned(),
        installed_capability_identity: installed_capability_identity.to_owned(),
        target_operation: target_operation.clone(),
    })
}

fn invalid_target() -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        "approval authority target is not one exact guarded operation",
    )
}
