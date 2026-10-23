//! A genuine approved effect is corrected without reopening its completed instance.
use super::*;
use crate::document_retention_model::{
    correctable_retention::{CorrectableRetentionBinding, CorrectableRetentionIntent},
    schema::SetRetentionInput,
    workflow::reviewed_document_definition_for_binding,
};
use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};
#[path = "correction/authored_divergence.rs"]
mod authored_divergence;

fn approved_source() -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowInstanceRef,
    RequiredWorkflowOperation,
    worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
) {
    let application = crate::document_retention_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_document_definition_for_binding::<CorrectableRetentionBinding>("applied"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        9_180,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("definition must publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, 9_181).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("instance must start: {other:?}"),
    };
    let (proposal, approval, _) =
        journey::approval_requirement(&application, instance.clone(), 9_180);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            9_189
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 9_190).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("approved operation must be required: {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
    };
    let source = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(CorrectableRetentionIntent {
            input: input.clone(),
        })
        .without_source()
        .idempotency(&9_191_u64)
        .for_workflow_operation(&application, &required)
        .unwrap()
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: source, ..
    } = source
    else {
        panic!("approved source must perform: {source:?}");
    };
    assert_eq!(read_retention(runtime, instance.branch()), 8);
    (application, instance, required, source)
}

#[test]
fn correction_preserves_completed_workflow_and_rejects_spent_approval() {
    let (application, instance, required, source) = approved_source();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    assert!(
        matches!(advance_instance(&application, instance.clone(), 9_192),
        Ok(WorkflowProgressOutcome::Completed(performed)) if performed.node_path() == "applied")
    );

    let request = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("9193")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("completed workflow source must prepare its ordinary inverse");
    };
    assert_eq!(
        read_retention(runtime, instance.branch()),
        8,
        "preparation cannot publish"
    );
    let inverse = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: inverse, ..
    } = inverse
    else {
        panic!("inverse must publish: {inverse:?}");
    };
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
    assert_eq!(
        inverse
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        source.commit_reference()
    );
    let replay = request
        .corrections()
        .reverse(&selected)
        .idempotency("9193")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replay),
    ) = replay
    else {
        panic!("exact workflow correction retry must return the original inverse");
    };
    assert_eq!(replay.commit_reference(), inverse.commit_reference());
    assert_eq!(
        instance_cancellation::cancellation_denial(
            crate::document_retention_model::workflow::cancel_instance(
                &application,
                instance.clone(),
                9_194
            )
        ),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCompleted
    );

    let stale = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(CorrectableRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&9_195_u64)
        .for_workflow_operation(&application, &required)
        .unwrap()
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        stale,
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
}

fn correct_source(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    source: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("9193")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("source must have an admitted ordinary inverse");
    };
    assert!(matches!(
        prepared.publish().unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
}

#[test]
fn correction_preserves_live_successor_instead_of_reexecuting_source() {
    let (application, instance, _, source) = approved_source();
    correct_source(&application, &instance, &source);
    assert!(matches!(advance_instance(&application, instance, 9_194),
        Ok(WorkflowProgressOutcome::Completed(performed)) if performed.node_path() == "applied"));
}

#[test]
fn correction_preserves_cancelled_instance() {
    let (application, instance, _, source) = approved_source();
    let cancelled = instance_cancellation::cancelled(
        crate::document_retention_model::workflow::cancel_instance(
            &application,
            instance.clone(),
            9_192,
        ),
    );
    assert_eq!(cancelled.performed_node_paths(), ["apply".to_owned()]);
    correct_source(&application, &instance, &source);
    assert_eq!(
        instance_cancellation::cancellation_denial(
            crate::document_retention_model::workflow::cancel_instance(
                &application,
                instance,
                9_194
            )
        ),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled
    );
}
