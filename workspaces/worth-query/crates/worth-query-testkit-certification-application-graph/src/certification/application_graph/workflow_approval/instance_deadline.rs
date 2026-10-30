//! A definition's total deadline bounds its instance's whole lineage on the
//! installed trusted clock. Once it elapses every step is refused, warm or
//! cold, and a step admitted in time is refused again at commit; a replay
//! still answers and cancellation still ends the instance. A successor keeps
//! the earlier of its source's deadline and its own.

pub use std::time::Duration;
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowProposalPreparationDenial,
};

pub use super::super::document_retention_model::{
    host::{publish_on_first_program_with_trusted_time, CertificationTrustedTime},
    workflow::{
        cancel_instance, continue_on_fork, migrate_instance, retain_workflow,
        reviewed_document_definition_with_deadline,
    },
};
pub use super::fork_continuation::fork_of;
pub use super::instance_cancellation::{approve, cancelled};
pub use super::instance_migration::{migration_denial, replace_definition, started};
pub use super::journey::approval_requirement;
pub use super::lineage_step_budget::step_denial;
pub use super::*;

pub const DEADLINE: Duration = Duration::from_secs(60);
pub const ELAPSED: WorthQueryApplicationAttemptDenialKind =
    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceDeadlineElapsed;

/// A started instance of a definition declaring `DEADLINE`, on trusted time
/// the certification moves.
pub fn deadline_instance(
    key: u64,
) -> (
    CertificationTrustedTime,
    DocumentWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
    let time = CertificationTrustedTime::new();
    let application = retain_workflow(publish_on_first_program_with_trusted_time(time.clone()));
    let definition = match publish_definition(
        &application,
        reviewed_document_definition_with_deadline("applied", DEADLINE),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the deadline definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the deadline definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition.clone(), key + 1)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    (time, application, definition, instance)
}

pub fn advance_denial(
    outcome: Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt),
        )) => attempt.kind(),
        Ok(WorkflowProgressOutcome::PreparationDenied(denial)) => denial.kind(),
        other => panic!("expected a typed step denial, got {other:?}"),
    }
}

pub fn warm_and_cold(application: &DocumentWorkflowRuntime, mut check: impl FnMut(&str)) {
    check("warm");
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    check("cold");
}

/// Admits the approved operation in time, lets `lapse` act on trusted time,
/// then requests the operation: its commit refusal, or the currentness
/// refusal that stopped it before its handler.
pub fn overdue_operation(
    key: u64,
    lapse: impl FnOnce(&CertificationTrustedTime),
) -> Result<WorthQueryApplicationCommitDenialKind, WorthQueryApplicationAttemptDenialKind> {
    let (time, application, _, instance) = deadline_instance(key);
    let (proposal, required, _) = approval_requirement(&application, instance.clone(), key);
    approve(&application, &instance, &required, &proposal, key + 10);
    let operation = match advance_instance(&application, instance.clone(), key + 11) {
        Ok(WorkflowProgressOutcome::AwaitingOperation(operation)) => operation,
        other => panic!("expected an admitted operation, got {other:?}"),
    };
    lapse(&time);

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
        .idempotency(&(key + 12))
        .for_workflow_operation(&application, &operation)
        .expect("the request matches the requirement it was issued")
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
    match effect {
        Ok(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::Denied(denial),
        )) => Ok(denial.kind()),
        Err(WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(denial)) => {
            Err(denial.kind())
        }
        other => panic!("an operation past trusted time never performs: {other:?}"),
    }
}
