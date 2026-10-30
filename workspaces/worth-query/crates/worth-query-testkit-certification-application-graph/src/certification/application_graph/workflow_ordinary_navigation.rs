//! Back's published transition and retained evidence agree with ordinary run.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryApplicationRequestExt, WorthQueryOrdinaryWorkflowRunStop,
};

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    workflow::{
        accept_assessment, advance_instance, assessment_join_terminal_definition,
        propose_authoring_instance, publish_definition, run_instance, settle_assessment,
        start_instance, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};
