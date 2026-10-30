//! Current input cardinality through ordinary publication and archive import.

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::OccurrenceSetRetentionBinding,
    workflow::{
        authoring_intent, definition_limits, ReviewedDocumentWorkflow,
        WorkflowDefinitionAuthoringOperation,
    },
};
pub use worth_query_decl::facade::application_program::*;
pub use worth_query_host::facade::application_entry::*;
pub use worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind;
pub use worth_query_package_archive::facade::{
    decode_workflow_definition_draft, WorthQueryPackageArchiveLimits,
};

pub fn draft(inputs: usize) -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "input-cardinality",
        definition_limits(),
    )
    .unwrap();
    let first = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("first", false)
        .unwrap();
    let second = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("second", false)
        .unwrap();
    let perform = builder
        .operation_binding::<OccurrenceSetRetentionBinding>("perform")
        .unwrap();
    let done = builder.terminal("done").unwrap();
    use ApplicationWorkflowControlOutcome::Completed;
    builder
        .start(&first)
        .control(&first, Completed, &second)
        .control(&second, Completed, &perform)
        .control(&perform, Completed, &done);
    if inputs >= 1 {
        builder.operation_input(&first, &perform);
    }
    if inputs >= 2 {
        builder.operation_input(&second, &perform);
    }
    builder.finish().unwrap()
}
