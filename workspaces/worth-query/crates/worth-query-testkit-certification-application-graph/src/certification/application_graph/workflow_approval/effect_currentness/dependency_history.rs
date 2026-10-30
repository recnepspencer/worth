//! Public-entry approval currentness, not general business-attestation supersession.
pub use super::super::approval_journey;
pub use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowInstanceRef, RequiredWorkflowOperation, WorkflowApprovalDecision,
        WorkflowProgressOutcome, WorthQueryApplicationMutationOutcome,
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
        WorthQueryWorkflowOperationBindingDenial,
    },
    runtime::ExecutionAllocationPolicy,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::{DocumentWorkflowRuntime, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::RetentionOutcome,
    readback::read_retention,
    retention_entry::{
        ReviewedSetRetentionIntent, SetRetentionIntent, DOCUMENT_IDENTITY,
        RELATED_DOCUMENT_IDENTITY,
    },
    schema::SetRetentionInput,
    settled_verdict::{settle, RetentionVerdict},
    workflow::{advance_instance, approve_instance},
};

pub fn signed_instance(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
    let (application, _, instance, proposal, approval, _) = approval_journey("applied", key);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            key + 10
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    (application, instance)
}

pub fn fresh_operation(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) -> RequiredWorkflowOperation {
    match advance_instance(application, instance.clone(), key).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected the original guarded operation, got {other:?}"),
    }
}

pub fn set_resource(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    identity: &str,
    retention_days: u64,
    key: u64,
) -> Result<RetentionOutcome, WorthQueryApplicationRequestMutationDenial> {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: identity.to_owned(),
                retention_days,
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
}
