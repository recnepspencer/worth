//! The workflow participant of branch adoption: its bounded inventory and the
//! admission of the caller's dispositions against a fresh read.

use super::WorthQueryBranchAdoptionPreparationDenial;
use crate::domain_computation::primary_graph::workflow::adoption::WorkflowAdoptionReadDenial;
use crate::domain_computation::primary_graph::workflow::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDispositions,
};

/// Resolve explicit intent against the complete fresh PRIVATE inventory.
pub(super) fn admit_workflow_dispositions(
    inventory: &WorthQueryWorkflowAdoptionInventory,
    choices: Option<&WorthQueryWorkflowDispositions>,
    branch: crate::basis::WorthQueryProductBranch,
) -> Result<
    Option<
        crate::domain_computation::primary_graph::workflow::adoption::AdmittedWorkflowDispositions,
    >,
    WorthQueryBranchAdoptionPreparationDenial,
> {
    let Some(choices) = choices else {
        return if inventory.is_empty() {
            Ok(None)
        } else {
            Err(WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired)
        };
    };
    choices
        .resolve(inventory, branch)
        .map(Some)
        .map_err(|_| WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected)
}

pub(super) fn workflow_read_denial(
    denial: WorkflowAdoptionReadDenial,
) -> WorthQueryBranchAdoptionPreparationDenial {
    match denial {
        WorkflowAdoptionReadDenial::ReadExecution(cause) => {
            WorthQueryBranchAdoptionPreparationDenial::ReadExecution(
                crate::facade::primary_graph::relational_execution_kind(cause, None),
            )
        }
        WorkflowAdoptionReadDenial::WorkLimitExceeded { .. } => {
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryWorkLimit
        }
        WorkflowAdoptionReadDenial::UnreadableEntity { .. }
        | WorkflowAdoptionReadDenial::UnreadableRelationSlot { .. } => {
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryUnavailable
        }
    }
}
