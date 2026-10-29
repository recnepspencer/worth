//! Adoption carries concrete workflow facts, never content: equal-content
//! publications and their instances stay distinct through a carry, and
//! nothing prepared under the source program survives adoption as authority.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
    WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
};
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

pub use super::workflow_participant::{expect_started, live_instance_on_first_program};
pub use crate::facade::certification::document_retention_model::{
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    workflow::{
        advance_instance, prepare_second_program_adoption, publish_adoption, publish_definition,
        retire_definition, start_instance, terminal_definition, WorkflowAdvanceInput,
        WorkflowAdvanceIntent,
    },
};

pub fn republish(
    application: &crate::facade::certification::document_retention_model::host::DocumentWorkflowRuntime,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(
        application,
        terminal_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the equal-content publication prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the equal-content publication did not publish: {other:?}"),
    }
}
