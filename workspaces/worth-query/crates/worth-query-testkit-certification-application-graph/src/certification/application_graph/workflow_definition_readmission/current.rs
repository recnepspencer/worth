pub use super::*;

pub fn occurrence_definition() -> Result<
    worth_query_host::facade::declaration::application_program::ValidatedWorkflowDefinition<
        worth_query_testkit_document_retention::facade::document_retention_model::workflow::governed_read::GovernedDocumentWorkflow,
    >,
    worth_query_host::facade::declaration::application_program::ApplicationWorkflowValidationDenial,
>{
    use worth_query_testkit_document_retention::facade::document_retention_model::workflow::{
        definition_limits, governed_read::GovernedDocumentWorkflow,
        WorkflowDefinitionAuthoringOperation,
    };
    use worth_query_host::facade::declaration::application_program::{
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    };
    let mut builder = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
        "information-recording",
        definition_limits(),
    )
    .unwrap();
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let perform = builder
        .operation_binding::<OccurrenceSetRetentionBinding>("perform")
        .unwrap();
    let done = builder.terminal("done").unwrap();
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &perform,
        )
        .control(
            &perform,
            ApplicationWorkflowControlOutcome::Completed,
            &done,
        )
        .operation_input(&propose, &perform);
    builder.finish().unwrap().validate()
}
