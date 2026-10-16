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
    maximum_selection_work: usize,
) -> WorthQueryBranchAdoptionPreparationDenial {
    match denial {
        WorkflowAdoptionReadDenial::Preparation(cause) => match cause {
            worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) =>
                workflow_read_denial(WorkflowAdoptionReadDenial::ReadExecution(cause), maximum_selection_work),
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
            | worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. } =>
                selection_limit(maximum_selection_work),
            cause => WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryPreparation(cause),
        },
        WorkflowAdoptionReadDenial::ReadExecution(cause) => {
            WorthQueryBranchAdoptionPreparationDenial::ReadExecution(
                crate::facade::primary_graph::relational_execution_kind(cause, None),
            )
        }
        WorkflowAdoptionReadDenial::WorkLimitExceeded { .. } => {
            selection_limit(maximum_selection_work)
        }
        WorkflowAdoptionReadDenial::IndexMaintenanceWorkLimit => {
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryWorkLimit
        }
        WorkflowAdoptionReadDenial::SelectedIndexUnavailable
        | WorkflowAdoptionReadDenial::UnreadableEntity { .. }
        | WorkflowAdoptionReadDenial::UnreadableRelationSlot { .. } => {
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryUnavailable
        }
    }
}

fn selection_limit(maximum_work_units: usize) -> WorthQueryBranchAdoptionPreparationDenial {
    // State selection and inventory spend one logical allowance. Exhaustion
    // identifies its stopping frontier, independently of native physical work.
    WorthQueryBranchAdoptionPreparationDenial::SelectionLimitExceeded {
        maximum_work_units,
        consumed_work_units: maximum_work_units,
    }
}
