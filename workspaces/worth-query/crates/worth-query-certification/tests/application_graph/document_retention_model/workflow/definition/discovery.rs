//! Actual principal-bearing retained metadata request used by fixture consumers.
use super::*;
use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowDefinitionDiscovery,
    application_entry::WorthQueryWorkflowObservationDenial, product::WorthQueryProductBranch,
};
/// Explicit Disabled policy must reject present and missing targets before lookup.
pub fn discover_definition_metadata(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            crate::document_retention_model::schema::DocumentRetentionSchema,
        >,
    >,
    branch: WorthQueryProductBranch,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> Result<WorthQueryWorkflowDefinitionDiscovery, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let basis = request.retain_current_read().unwrap();
    let target=worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionReadTarget::<ReviewedDocumentWorkflow>::new(identity.clone());
    match request.at(&basis).discover_workflow_definition::<ReviewedDocumentWorkflow,super::super::governed_read::MetadataRead>(
        application,&target,super::super::governed_read::WorkflowReadInput {document_identity:DOCUMENT_IDENTITY.to_owned()}) {
        Ok(view)=>Ok(view.definition().clone()),
        Err(WorthQueryWorkflowObservationDenial::Read(worth_query_host::facade::application_discovery::WorthQueryWorkflowReadDenial::Installation(cause)))=>{
            assert_eq!(cause.kind(),worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy);
            Err(())
        },
        Err(other)=>panic!("Disabled metadata must refuse at its installed policy, not another gate: {other:?}"),
    }
}
