//! Retirement and reopening race other definition authors between prepare
//! and commit. An interleaved commit changes tracked facts, so the loser is
//! denied before effects; re-preparing observes the winner's currentness.

pub use super::super::document_retention_model::{
    retention_entry::DOCUMENT_IDENTITY,
    workflow::{WorkflowInstanceStartInput, WorkflowInstanceStartIntent},
};
pub use super::*;

pub fn retire_after(
    application: &DocumentWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
    between: impl FnOnce(),
) -> Result<
    WorkflowDefinitionRetirementOutcome,
    WorthQueryWorkflowDefinitionRetirementPreparationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_definition_retirement(application, definition)?;
    between();
    Ok(prepared.execute())
}

pub fn publish_after(
    application: &DocumentWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>,
    idempotency: u64,
    between: impl FnOnce(),
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let contract = application
        .workflow_spec()
        .bind_definition(definition)
        .expect("the workflow definition binds to installed vocabulary");
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)?;
    between();
    Ok(prepared.execute())
}

pub fn expect_changed_facts_stale<Denial: std::fmt::Debug>(
    result: Result<WorthQueryApplicationUncommitted, Denial>,
) {
    match result.expect("the losing request prepared before the winner committed") {
        WorthQueryApplicationUncommitted::Stale(stale) => assert!(stale.stale_fact_count() > 0),
        other => panic!("expected changed prepared facts to deny, got {other:?}"),
    }
}
