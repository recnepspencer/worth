//! Certification of immutable workflow proposals and retained replay.

pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted},
};

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    workflow::{
        advance_instance, proposal_terminal_definition, propose_authoring_instance,
        propose_instance, propose_instance_on_branch, publish_definition,
        repeated_proposal_definition, reviewed_document_definition, start_instance,
    },
};

pub fn expect_published(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        worth_query_host::facade::application_entry::WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowDefinitionPublication {
    match result.expect("workflow definition publication must prepare") {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected a published workflow definition, got {other:?}"),
    }
}

pub fn expect_started(
    result: Result<
        WorkflowInstanceStartOutcome,
        worth_query_host::facade::application_entry::WorthQueryWorkflowInstancePreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowInstanceStart {
    match result.expect("workflow instance start must prepare") {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected a started workflow instance, got {other:?}"),
    }
}

pub fn expect_proposal(
    result: Result<
        WorkflowProposalOutcome,
        worth_query_host::facade::application_entry::WorthQueryWorkflowProposalPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowProposal {
    match result.expect("workflow proposal preparation must succeed") {
        WorkflowProposalOutcome::Published(proposal) => proposal,
        other => panic!("expected a published workflow proposal, got {other:?}"),
    }
}
