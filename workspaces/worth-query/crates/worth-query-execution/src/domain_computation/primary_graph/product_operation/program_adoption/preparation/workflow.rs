//! The workflow participant of branch adoption: its bounded inventory and the
//! admission of the caller's dispositions against a fresh read.

use super::WorthQueryBranchAdoptionPreparationDenial;
use crate::domain_computation::primary_graph::workflow::adoption::WorkflowAdoptionReadDenial;

pub(super) fn workflow_read_denial(
    denial: WorkflowAdoptionReadDenial,
    maximum_selection_work: usize,
) -> WorthQueryBranchAdoptionPreparationDenial {
    match denial {
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
