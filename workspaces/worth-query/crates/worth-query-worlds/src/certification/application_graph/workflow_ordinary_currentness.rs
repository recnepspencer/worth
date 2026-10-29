//! The ordinary workflow entry preserves native evidence currentness across ABA.

pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryOrdinaryWorkflowRunStop,
};

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, SEED_RETENTION},
    presented_request::set_retention,
    retention_entry::DOCUMENT_IDENTITY,
    settled_verdict::{settle, RetentionVerdict},
    workflow::{
        accept_assessment, accept_early_assessment, advance_instance,
        assessment_join_terminal_definition, propose_instance, publish_definition, run_instance,
        settle_assessment, settle_early_assessment_for, start_instance,
    },
};
