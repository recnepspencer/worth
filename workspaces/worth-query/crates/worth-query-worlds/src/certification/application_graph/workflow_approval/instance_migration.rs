//! Explicit migration continues a live instance on a newer definition as a
//! successor. It carries performed effects as history the successor can never
//! repeat, and re-establishes every proposal, evidence and approval it needs.

pub use worth_query_host::facade::application_entry::{
    WorkflowInstancePreparationDenial, WorthQueryWorkflowInstancePreparationDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryWorkflowInstanceDisposition;

pub use super::super::document_retention_model::{
    programs::RetentionProgramP1,
    workflow::{migrate_instance, support_workflow_program},
};
pub use super::*;

pub fn supersede(
    application: &DocumentWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    completion: &str,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    replace_definition(
        application,
        definition,
        reviewed_document_definition(completion),
        key,
    )
}

pub fn replace_definition(
    application: &DocumentWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    replacement: worth_query_host::facade::declaration::application_program::ValidatedWorkflowDefinition<
        super::super::document_retention_model::workflow::ReviewedDocumentWorkflow,
    >,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(
        application,
        replacement,
        WorkflowDefinitionExpectedPredecessor::Published(definition),
        key,
    )
    .expect("the successor definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the successor definition did not publish: {other:?}"),
    }
}

pub fn perform_approved_effect(
    application: &DocumentWorkflowRuntime<
        crate::facade::certification::document_retention_model::programs::RetentionProgramP0,
        impl worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec<
            Schema = crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
        >,
    >,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) -> worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt {
    let required = match advance_instance(application, instance.clone(), key)
        .expect("the approved operation requirement prepares")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetRetentionIntent {
            input: super::super::document_retention_model::schema::SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&(key + 1))
        .for_workflow_operation(application, &required)
        .expect("the effect request matches the operation requirement")
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the approved effect executes");
    match effect {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("the approved native effect did not commit: {other:?}"),
    }
}

pub fn started(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) -> worth_query_host::facade::application_entry::PerformedWorkflowInstanceStart {
    match outcome.expect("the migration prepares") {
        WorkflowInstanceStartOutcome::Started(performed) => performed,
        other => panic!("the migration did not publish: {other:?}"),
    }
}

pub fn migration_denial(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        )) => attempt.kind(),
        other => panic!("expected a typed migration denial, got {other:?}"),
    }
}
