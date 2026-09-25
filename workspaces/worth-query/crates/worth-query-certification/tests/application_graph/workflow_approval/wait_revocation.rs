//! Published approval authority is read back and rechecked at the effect wait.

use super::super::bounded_dimension_model::workflow::{
    WorkflowGrantStatusInput, WorkflowGrantStatusIntent,
};
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestMutationDenial,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

#[test]
fn revoked_approval_grant_cannot_authorize_the_waiting_effect() {
    assert_grant_change_denies_wait(
        &["revoked"],
        2_100,
        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable,
    );
}

#[test]
fn reactivated_approval_grant_does_not_revive_the_published_approval() {
    assert_grant_change_denies_wait(
        &["revoked", "active"],
        2_200,
        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied,
    );
}

fn assert_grant_change_denies_wait(
    statuses: &[&str],
    key: u64,
    expected: WorthQueryApplicationAttemptDenialKind,
) {
    let (application, _, instance, proposal, approval, _) = approval_journey("applied", key);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            key + 10,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let required = match advance_instance(&application, instance.clone(), key + 11)
        .expect("the published approval reaches its gated operation")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected a waiting operation, got {other:?}"),
    };
    for (index, status) in statuses.iter().enumerate() {
        change_approval_grant(&application, &instance, status, key + 12 + index as u64);
    }

    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&(key + 20))
        .for_workflow_operation(&application, &required)
        .expect("the operation binding remains exact")
        .execute_in_program(application.program_runtime());
    assert!(
        matches!(
            &effect,
            Err(WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(denial))
                if denial.kind() == expected
        ),
        "the published approval must lose authority at the wait: {effect:?}"
    );
    assert_eq!(read_dimension(runtime, instance.branch()), SEED_DIMENSION);
    assert!(matches!(
        advance_instance(&application, instance, key + 21),
        Ok(WorkflowProgressOutcome::AwaitingOperation(_))
    ));
}

fn change_approval_grant(
    application: &BoundedDimensionWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    status: &str,
    key: u64,
) {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: "workflow-approval-grant".to_owned(),
                status: status.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application.program_runtime())
        .expect("grant change must prepare through its installed ordinary operation");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the grant change must publish before effect admission: {outcome:?}"
    );
}
