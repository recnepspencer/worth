//! Moving a workflow host from the first program to `RetentionProgramP1`.
//!
//! Workflows on the adopted branch are then driven through the ordinary
//! helpers: each entry prepares against the vocabulary installed for the
//! program its branch runs.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::declaration::application_program::ApplicationProgramRevision;
use worth_query_host::facade::primary_graph::{
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryPreparedBranchAdoption,
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDispositions,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::super::{
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP1,
};

/// The caller's decision over the inventory the owner just read.
pub type WorkflowDecision<'decide> =
    &'decide dyn Fn(&WorthQueryWorkflowAdoptionInventory) -> WorthQueryWorkflowDispositions;

pub fn second_program_workflow_inventory(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
) -> WorthQueryWorkflowAdoptionInventory {
    let target = second_revision(application);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    programs
        .adopt(&requirements)
        .workflow_inventory(1_024)
        .expect("the owner inventories the branch's workflow facts")
}

/// Prepares the move to P1, deciding the fresh inventory when `decide` is set.
pub fn prepare_second_program_adoption(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    decide: Option<WorkflowDecision<'_>>,
) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryApplicationProgramAdoptionPreparationDenial>
{
    let target = second_revision(application);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let request = programs.adopt(&requirements);
    let request = match decide {
        Some(decide) => {
            let inventory = request
                .workflow_inventory(1_024)
                .expect("the owner inventories the branch's workflow facts");
            request.workflow(decide(&inventory))
        }
        None => request,
    };
    request.prepare(1_024)
}

pub fn publish_adoption(
    prepared: Result<
        WorthQueryPreparedBranchAdoption,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    >,
) {
    match prepared.expect("decided workflow facts adopt").publish() {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {}
        _ => panic!("the adoption did not publish"),
    }
}

pub fn second_revision(application: &DocumentWorkflowRuntime) -> ApplicationProgramRevision {
    *application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
}
