//! The existing reviewed document graph with an explicitly captured target input.
use super::*;
use worth_query_decl::facade::application_program::*;
pub(super) use document::workflow::home_transition::reviewed;

pub(super) fn unsigned() -> (
    ValidatedWorkflowDefinition<GovernedDocumentWorkflow>,
    ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
) {
    let mut graph = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
        "reviewed-document",
        definition_limits(),
    )
    .unwrap();
    let proposal = graph
        .operation_binding::<WorkflowDefinitionAuthoringBinding>("propose")
        .unwrap();
    let effect = graph
        .operation_binding::<document::retention_entry::OccurrenceSetRetentionBinding>("apply")
        .unwrap();
    let done = graph.terminal("completed").unwrap();
    graph
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &effect,
        )
        .control(&effect, ApplicationWorkflowControlOutcome::Completed, &done)
        .captured_operation_input(&proposal, &effect);
    (graph.finish().unwrap().validate().unwrap(), effect)
}
