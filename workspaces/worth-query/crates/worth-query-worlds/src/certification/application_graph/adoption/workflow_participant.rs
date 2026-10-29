//! Program adoption decides every live workflow fact on the branch explicitly:
//! carry it to the target in place, retire a definition, or cancel an instance
//! that has performed nothing.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryWorkflowInstancePreparationDenial,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryBranchAdoptionPreparationDenial,
    WorthQueryWorkflowDefinitionDisposition as DefinitionDisposition,
    WorthQueryWorkflowInstanceDisposition as InstanceDisposition,
};

pub use crate::facade::certification::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    programs::RetentionProgramP1,
    workflow::{
        advance_instance, prepare_second_program_adoption, publish_adoption, publish_definition,
        start_instance, support_workflow_program, terminal_definition,
    },
};

pub fn live_instance_on_first_program(
    key: u64,
) -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);
    let definition = match publish_definition(
        &application,
        terminal_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the P0 definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the P0 definition did not publish: {other:?}"),
    };
    let instance = expect_started(start_instance(&application, definition.clone(), key + 1));
    (application, definition, instance)
}

pub fn expect_started(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) -> PublishedWorkflowInstanceRef {
    match outcome.expect("the instance prepares") {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    }
}
