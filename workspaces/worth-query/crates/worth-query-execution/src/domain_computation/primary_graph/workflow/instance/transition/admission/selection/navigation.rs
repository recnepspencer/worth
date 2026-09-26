use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

use super::denial;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode,
};
use crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceProgress;

/// Back cannot leave an operation while its approval authority is live.
///
/// Once the approval source is `Approved` and no transition at the operation
/// has settled since, a guarded effect may already be committed under that
/// authority. Leaving would orphan it, and a later re-approval would make an
/// unreconciled effect look consumed. Only receipted settlement leaves.
pub(super) fn deny_back_over_authorized_operation(
    compiled: &CompiledWorkflowDefinition,
    progress: &WorkflowInstanceProgress,
    node: &CompiledWorkflowNode,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    // Matches adoption custody: only a receipted settlement consumes approval.
    let settled = progress
        .latest_transition(node.entity())
        .map(|transition| transition.settlement())
        .filter(|settlement| settlement.operation_receipt_identity().is_some())
        .map(|settlement| settlement.occurrence());
    let authorized = compiled
        .approval_authority_sources(node.entity())
        .filter_map(|approval| progress.latest_transition(approval.entity()))
        .map(|transition| transition.settlement())
        .any(|approval| {
            approval.outcome() == ApplicationWorkflowControlOutcome::Approved
                && settled.is_none_or(|occurrence| occurrence < approval.occurrence())
        });
    if authorized {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled,
            "workflow Back cannot abandon an authorized operation",
        ));
    }
    Ok(())
}
