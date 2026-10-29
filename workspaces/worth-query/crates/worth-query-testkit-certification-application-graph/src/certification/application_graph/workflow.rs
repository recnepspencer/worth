//! Public certification for authored workflow-definition publication.

pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowInstancePreparationDenial,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted},
};

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    workflow::{
        advance_instance, expect_capacity_refused, expect_superseded_start, publish_definition,
        reviewed_document_definition, start_instance, terminal_definition,
    },
};

pub fn expect_started(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) -> worth_query_host::facade::application_entry::PerformedWorkflowInstanceStart {
    match result.expect("workflow instance-start preparation must succeed") {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("workflow instance start failed: {other:?}"),
    }
}

pub fn expect_published(
    context: &str,
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> worth_query_host::facade::application_entry::PerformedWorkflowDefinitionPublication {
    match result.expect("the public workflow publication must prepare") {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        unexpected => {
            panic!("{context}: expected a published workflow definition, got {unexpected:?}")
        }
    }
}

pub fn expect_stale_predecessor(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result {
        Ok(WorkflowDefinitionPublicationOutcome::Application(
            WorthQueryApplicationUncommitted::Stale(stale),
        )) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected stale predecessor refusal, got {unexpected:?}"),
    }
}

pub fn expect_start_intent_drift(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) {
    match result.expect("the changed workflow start intent must reach idempotency") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        unexpected => {
            panic!("expected workflow start idempotency intent drift, got {unexpected:?}")
        }
    }
}

pub fn expect_intent_drift(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result.expect("the changed workflow intent must reach idempotency") {
        WorkflowDefinitionPublicationOutcome::Application(
            WorthQueryApplicationUncommitted::Denied(denial),
        ) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        unexpected => panic!("expected workflow idempotency intent drift, got {unexpected:?}"),
    }
}
