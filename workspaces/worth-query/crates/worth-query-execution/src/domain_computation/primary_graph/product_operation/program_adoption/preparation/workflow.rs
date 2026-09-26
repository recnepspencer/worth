//! The workflow participant of branch adoption: its bounded inventory and the
//! admission of the caller's dispositions against a fresh read.

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::WorthQueryBranchAdoptionPreparationDenial;
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::workflow::adoption::{
    inventory_workflows, WorkflowAdoptionInventoryRequest, WorkflowAdoptionReadDenial,
};
use crate::domain_computation::primary_graph::workflow::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDispositions,
};

pub(super) fn inventory<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
    expected_requirements: &WorthQueryProgramAdoptionRequirements,
    maximum_work_units: usize,
) -> Result<WorthQueryWorkflowAdoptionInventory, WorthQueryBranchAdoptionPreparationDenial> {
    let (source, requirements) = super::state::requirements(selected, target)?;
    if &requirements != expected_requirements {
        return Err(WorthQueryBranchAdoptionPreparationDenial::RequirementsChanged);
    }
    let application = selected.application();
    let support = application
        .installed_program_support()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportUnavailable)?;
    // Installed coverage speaks for the target only while support holds it
    // active; a retiring target must not license a carry.
    let _custody = support
        .retain_custody(&source, target)
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportRetirementInProgress)?;
    let graph = &application.primary_provider.graph;
    let request = WorkflowAdoptionInventoryRequest {
        layout: graph.layout.workflow(),
        version: selected
            .product()
            .relational_basis()
            .observation()
            .version_id(),
        branch_occurrence: selected.product().product_branch().occurrence_ordinal(),
        coverage: &application.workflow_coverage,
        requirements: &requirements,
        source: &source,
        target,
        maximum_work_units,
    };
    graph
        .with_runtime(|runtime| inventory_workflows(runtime, &request))
        .map_err(|denial| workflow_read_denial(denial, maximum_work_units, 0))
}

/// Admits the caller's workflow choices against the fresh inventory. An empty
/// inventory needs none; any supplied choices must still match its digest.
pub(super) fn admit_workflow_dispositions<'choices>(
    inventory: &WorthQueryWorkflowAdoptionInventory,
    choices: Option<&'choices WorthQueryWorkflowDispositions>,
) -> Result<
    Option<&'choices WorthQueryWorkflowDispositions>,
    WorthQueryBranchAdoptionPreparationDenial,
> {
    let Some(choices) = choices else {
        if inventory.is_empty() {
            return Ok(None);
        }
        return Err(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired {
                inventory: Box::new(inventory.clone()),
            },
        );
    };
    if choices.inventory_digest() != inventory.digest() {
        return Err(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryChanged {
                inventory: Box::new(inventory.clone()),
            },
        );
    }
    inventory
        .admit(choices)
        .map_err(WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected)?;
    Ok(Some(choices))
}

pub(super) fn workflow_read_denial(
    denial: WorkflowAdoptionReadDenial,
    maximum_work_units: usize,
    selection_work_units: usize,
) -> WorthQueryBranchAdoptionPreparationDenial {
    match denial {
        WorkflowAdoptionReadDenial::WorkLimitExceeded {
            consumed_work_units,
        } => WorthQueryBranchAdoptionPreparationDenial::SelectionLimitExceeded {
            maximum_work_units,
            consumed_work_units: selection_work_units.saturating_add(consumed_work_units),
        },
        WorkflowAdoptionReadDenial::UnreadableEntity { entity } => {
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryUnreadable { entity }
        }
        WorkflowAdoptionReadDenial::UnreadableRelationSlot { partition_id, slot } => {
            WorthQueryBranchAdoptionPreparationDenial::WorkflowInventoryRelationUnreadable {
                partition_id,
                slot,
            }
        }
    }
}
