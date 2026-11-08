//! One fresh disposition admission and realization owner.
use super::*;
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor;
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::transactions::MutationIntent;

pub(in crate::domain_computation::primary_graph) enum WorkflowDispositionPreparationDenial {
    Required,
    Rejected,
}
pub(in crate::domain_computation::primary_graph) fn prepare_workflow_dispositions(
    layout: &WorthQueryWorkflowLayout,
    inventory: &WorthQueryWorkflowAdoptionInventory,
    choices: Option<&WorthQueryWorkflowDispositions>,
    branch: WorthQueryProductBranch,
    target: &ApplicationProgramRevision,
    actor: worth_proof::AdmittedWorkflowEndingActor<WorkflowAdmittedActor>,
) -> Result<Vec<MutationIntent>, WorkflowDispositionPreparationDenial> {
    let Some(choices) = choices else {
        return if inventory.is_empty() {
            Ok(Vec::new())
        } else {
            Err(WorkflowDispositionPreparationDenial::Required)
        };
    };
    let admitted = choices
        .resolve(inventory, branch)
        .map_err(|_| WorkflowDispositionPreparationDenial::Rejected)?;
    Ok(super::staging::stage_workflow_dispositions(
        layout, inventory, &admitted, target, actor,
    ))
}
