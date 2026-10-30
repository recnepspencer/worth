//! Explicit cancellation ends a live instance where it stands. It is not
//! rollback: a performed effect remains and is reported, a step admitted
//! before the cancellation is refused as cancelled before its effect, and a
//! cancellation prepared before a step settles goes stale in turn.

pub use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceCancellation, WorkflowDefinitionRetirementOutcome,
    WorkflowInstanceCancellationOutcome, WorkflowInstancePreparationDenial,
    WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowInstancePreparationDenial,
};

pub use super::super::document_retention_model::{
    programs::RetentionProgramP1,
    workflow::{
        cancel_instance, prepare_cancellation, prepare_second_program_adoption, publish_adoption,
        retire_definition, support_workflow_program,
    },
};
pub use super::instance_migration::perform_approved_effect;
pub use super::*;

pub fn approve(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    key: u64,
) {
    match approve_instance(
        application,
        instance.clone(),
        required,
        proposal,
        WorkflowApprovalDecision::Approve,
        key,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected approval to complete, got {other:?}"),
    }
}

pub fn cancelled(
    outcome: Result<
        WorkflowInstanceCancellationOutcome,
        WorthQueryWorkflowInstancePreparationDenial,
    >,
) -> PerformedWorkflowInstanceCancellation {
    match outcome.expect("the cancellation prepares") {
        WorkflowInstanceCancellationOutcome::Cancelled(performed) => performed,
        other => panic!("the cancellation did not commit: {other:?}"),
    }
}

pub fn cancellation_denial(
    outcome: Result<
        WorkflowInstanceCancellationOutcome,
        WorthQueryWorkflowInstancePreparationDenial,
    >,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        )) => attempt.kind(),
        other => panic!("expected a typed cancellation denial, got {other:?}"),
    }
}
