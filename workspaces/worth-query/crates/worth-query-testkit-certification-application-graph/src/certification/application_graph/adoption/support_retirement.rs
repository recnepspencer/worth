//! Retirement courts for installed program support and its execution-owned users.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome,
};
pub use worth_query_host::facade::application_installation::{
    WorthQueryProgramOwner, WorthQueryProgramSupportRetirementFailure,
};
pub use worth_query_host::facade::domain::{
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramSupportRetirementDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::RetentionProgramP1;

pub fn adopt(
    host: &worth_query_testkit_document_retention::facade::document_retention_model::host::DocumentRetentionRuntime<
        worth_query_testkit_document_retention::facade::document_retention_model::programs::RetentionProgramP0,
    >,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    target: &worth_query_host::facade::declaration::application_program::ApplicationProgramRevision,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("the adoption prepares");
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}
