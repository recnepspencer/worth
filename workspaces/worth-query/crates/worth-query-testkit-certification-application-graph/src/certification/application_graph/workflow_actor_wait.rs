//! An actor wait is a typed result of denied authority, without live-head disclosure.

pub use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial, WorthQueryOrdinaryWorkflowRunStop,
    WorthQueryWorkflowAdvancePreparationDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    workflow::{
        advance_instance, condition_terminal_definition, propose_authoring_instance,
        publish_definition, run_instance, start_instance, terminal_definition,
        WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowApprovalIntent,
        WorkflowGrantStatusInput, WorkflowGrantStatusIntent,
    },
};

pub fn change_advance_grant(
    application: &super::document_retention_model::host::DocumentWorkflowRuntime,
    instance: &worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    status: &str,
    key: u64,
) {
    change_grant(application, instance, "workflow-advance-grant", status, key);
}

pub fn change_grant(
    application: &super::document_retention_model::host::DocumentWorkflowRuntime,
    instance: &worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    grant_identity: &str,
    status: &str,
    key: u64,
) {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .advanced()
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: grant_identity.to_owned(),
                status: status.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            application.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("ordinary grant change prepares");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "ordinary grant change must publish: {outcome:?}"
    );
}
