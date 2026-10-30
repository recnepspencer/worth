//! Public certification for definition retirement and lineage reopening.

pub use worth_query_host::facade::{
    application_entry::{
        PerformedWorkflowDefinitionRetirement, PublishedWorkflowDefinitionRef,
        PublishedWorkflowInstanceRef, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
        WorthQueryWorkflowInstancePreparationDenial,
    },
    declaration::application_program::ValidatedWorkflowDefinition,
    primary_graph::WorthQueryApplicationUncommitted,
};

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        advance_instance, authoring_intent, expect_retired_start, expect_superseded_start,
        publish_definition, retire_definition, start_instance, terminal_definition,
        ReviewedDocumentWorkflow,
    },
};

#[path = "workflow_retirement/contention.rs"]
pub mod contention;
#[path = "workflow_retirement/denial.rs"]
pub mod denial;
#[path = "workflow_retirement/discovery.rs"]
pub mod discovery;
#[path = "workflow_retirement/fork_definitions.rs"]
pub mod fork_definitions;

pub fn expect_published(
    context: &str,
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> PublishedWorkflowDefinitionRef {
    match result.expect("the workflow publication must prepare") {
        WorkflowDefinitionPublicationOutcome::Published(publication) => {
            publication.definition().clone()
        }
        unexpected => panic!("{context}: expected a publication, got {unexpected:?}"),
    }
}

pub fn expect_stale_publication(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result.expect("the workflow publication must reach commit comparison") {
        WorkflowDefinitionPublicationOutcome::Application(
            WorthQueryApplicationUncommitted::Stale(stale),
        ) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected a stale publication predecessor, got {unexpected:?}"),
    }
}

pub fn expect_retired(
    result: Result<
        WorkflowDefinitionRetirementOutcome,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
    >,
) -> PerformedWorkflowDefinitionRetirement {
    match result.expect("the workflow retirement must prepare") {
        WorkflowDefinitionRetirementOutcome::Retired(retired) => retired,
        unexpected => panic!("expected a retired definition, got {unexpected:?}"),
    }
}

pub fn expect_stale_retirement(
    result: Result<
        WorkflowDefinitionRetirementOutcome,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
    >,
) {
    match result.expect("the workflow retirement must reach commit comparison") {
        WorkflowDefinitionRetirementOutcome::Application(
            WorthQueryApplicationUncommitted::Stale(stale),
        ) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected a stale retirement, got {unexpected:?}"),
    }
}

pub fn expect_started(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) -> PublishedWorkflowInstanceRef {
    match result.expect("workflow instance-start preparation must succeed") {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("workflow instance start failed: {other:?}"),
    }
}
