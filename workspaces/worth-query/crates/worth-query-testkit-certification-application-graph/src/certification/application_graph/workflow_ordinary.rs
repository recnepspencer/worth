pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
        WorthQueryOrdinaryWorkflowPublicationDenial, WorthQueryOrdinaryWorkflowRunStop,
    },
    declaration::application_program::{
        ApplicationWorkflowDefinitionBuilder, AuthoredWorkflowDefinition,
    },
};

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    workflow::{
        accept_assessment, assessment_join_terminal_definition, propose_instance,
        publish_definition, reviewed_document_definition, settle_assessment, start_instance,
        ReviewedDocumentWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
        WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
    },
};

pub fn terminal_draft() -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "ordinary-terminal",
        super::document_retention_model::workflow::definition_limits(),
    )
    .expect("the ordinary workflow identity is valid");
    let done = builder.terminal("done").expect("the terminal is valid");
    builder.start(&done);
    builder.finish().expect("the draft is complete")
}
