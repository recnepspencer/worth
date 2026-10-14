//! Public-entry approval currentness, not general business-attestation supersession.
use super::super::approval_journey;
use crate::document_retention_model::{
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
use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowInstanceRef, RequiredWorkflowOperation, WorkflowApprovalDecision,
        WorkflowProgressOutcome, WorthQueryApplicationMutationOutcome,
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
        WorthQueryWorkflowOperationBindingDenial,
    },
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn signed_dependency_history_preserves_approval_after_an_unrelated_resource_edit() {
    let (application, instance) = signed_instance(30_000);
    assert_eq!(
        settle(set_resource(
            &application,
            &instance,
            RELATED_DOCUMENT_IDENTITY,
            8,
            30_020
        )),
        RetentionVerdict::Performed(8)
    );
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
    let required = fresh_operation(&application, &instance, 30_021);
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let effect = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&30_022_u64)
        .for_workflow_operation(&application, &required)
        .expect("unrelated resource progress must not revoke this approval")
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            effect,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{effect:?}"
    );
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
}

#[test]
fn signed_dependency_history_does_not_revive_approval_when_source_value_returns() {
    let (application, instance) = signed_instance(30_100);
    for (retention, key) in [(8, 30_120), (SEED_RETENTION, 30_121)] {
        assert_eq!(
            settle(set_resource(
                &application,
                &instance,
                DOCUMENT_IDENTITY,
                retention,
                key
            )),
            RetentionVerdict::Performed(retention)
        );
    }
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
    // Issue fresh authority after both edits. A consumed or stale in-process
    // slot cannot counterfeit the intended historical-dependency refusal.
    let required = fresh_operation(&application, &instance, 30_122);
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let effect = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&30_123_u64)
        .for_workflow_operation(&application, &required);
    assert!(
        matches!(
            effect,
            Err(WorthQueryWorkflowOperationBindingDenial::AuthorityUnavailable)
        ),
        "returning to the old value must not restore signed evidence"
    );
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
}

fn signed_instance(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
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

fn fresh_operation(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) -> RequiredWorkflowOperation {
    match advance_instance(application, instance.clone(), key).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected the original guarded operation, got {other:?}"),
    }
}

fn set_resource(
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
