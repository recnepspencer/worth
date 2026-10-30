//! Native occurrence-bound business execution without inventing an approval.

#[path = "workflow_execution/review.rs"]
pub mod review;

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::read_retention,
    retention_entry::{
        candidate_count, reset_candidate_count, ExplicitReviewedSetRetentionBinding,
        OccurrenceSetRetentionBinding, ReviewedSetRetentionBinding, DOCUMENT_IDENTITY,
    },
    schema::{DocumentRetentionSchema, SetRetentionInput},
    workflow::{
        advance_instance, definition_limits, propose_authoring_instance_with_retention,
        publish_definition, start_instance, ReviewedDocumentWorkflow,
        WorkflowDefinitionAuthoringOperation,
    },
};
pub use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding, application_program::*,
};
pub use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowInstanceRef, RequiredWorkflowOperation,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowOperationBindingDenial,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted},
    runtime::ExecutionAllocationPolicy,
};

pub fn definition<Binding: ApplicationMutationBinding<DocumentRetentionSchema>>() -> Result<
    ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>,
    ApplicationWorkflowValidationDenial,
> {
    let limits = definition_limits();
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "information-recording",
        limits,
    )
    .unwrap();
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let perform = builder.operation_binding::<Binding>("perform").unwrap();
    let done = builder.terminal("done").unwrap();
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &perform,
        )
        .control(
            &perform,
            ApplicationWorkflowControlOutcome::Completed,
            &done,
        )
        .operation_input(&propose, &perform);
    builder.finish().unwrap().validate()
}

pub fn journey(
    retention: u64,
) -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowInstanceRef,
    RequiredWorkflowOperation,
) {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        definition::<OccurrenceSetRetentionBinding>().unwrap(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        91001,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, published, 91002).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("instance did not start: {other:?}"),
    };
    assert!(matches!(
        propose_authoring_instance_with_retention(&application, instance.clone(), 91003, retention)
            .unwrap(),
        WorkflowProposalOutcome::Published(_)
    ));
    let required = match advance_instance(&application, instance.clone(), 91004).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("unsigned operation did not become executable: {other:?}"),
    };
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_none());
    (application, instance, required)
}

pub fn execute(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowOperation,
    retention: u64,
    key: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<
        super::document_retention_model::retention_entry::SetRetentionDenial,
        super::document_retention_model::retention_entry::DocumentRetentionWritten,
    >,
    WorthQueryApplicationRequestMutationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: retention,
        })
        .without_source()
        .idempotency(&key)
        .for_workflow_operation(application, required)
        .unwrap()
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
}
