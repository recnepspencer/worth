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
    WorthQueryWorkflowDispositions,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::super::{
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP1,
};

/// Prepares the move to P1, deciding the fresh inventory when `decide` is set.
pub fn prepare_second_program_adoption(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            crate::document_retention_model::schema::DocumentRetentionSchema,
        >,
    >,
    branch: WorthQueryProductBranch,
    decide: Option<WorthQueryWorkflowDispositions>,
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
    let request = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits");
    let request = match decide {
        Some(choices) => request.workflow(choices),
        None => request,
    };
    request.prepare(1_024)
}

pub fn publish_adoption(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            crate::document_retention_model::schema::DocumentRetentionSchema,
        >,
    >,
    prepared: Result<
        WorthQueryPreparedBranchAdoption,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    >,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    match prepared.expect("decided workflow facts adopt").publish(
        application.runtime(),
        &principal,
        &scope,
    ) {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {}
        _ => panic!("the adoption did not publish"),
    }
}

pub fn second_revision(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            crate::document_retention_model::schema::DocumentRetentionSchema,
        >,
    >,
) -> ApplicationProgramRevision {
    *application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
}
