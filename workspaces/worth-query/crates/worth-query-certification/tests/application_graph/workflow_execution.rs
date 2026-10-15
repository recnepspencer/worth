//! Native occurrence-bound business execution without inventing an approval.

#[path = "workflow_execution/review.rs"]
mod review;

use super::document_retention_model::{
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
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding, application_program::*,
};
use worth_query_host::facade::{
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

pub(super) fn definition<Binding: ApplicationMutationBinding<DocumentRetentionSchema>>() -> Result<
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

fn journey(
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
        .unwrap()
        .approval()
        .is_none());
    (application, instance, required)
}

fn execute(
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
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
}

#[test]
fn unsigned_execution_settles_atomically_and_same_intent_replays_without_signing() {
    let (application, instance, required) = journey(8);
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
    let outcome = execute(&application, &instance, &required, 8, 91005).unwrap();
    let receipt = match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result } => {
            assert_eq!(result.retention_days, 8);
            receipt
        }
        other => panic!("unsigned effect did not commit: {other:?}"),
    };
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
    // Lost-response retry uses the original requirement and key, not a new operation.
    match execute(&application, &instance, &required, 8, 91005).unwrap() {
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed) => {
            assert_eq!(replayed.outcome_identity(), receipt.outcome_identity());
        }
        other => panic!("same-intent execution did not replay: {other:?}"),
    }
    match advance_instance(&application, instance, 91006).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("the business commit did not settle its native operation: {other:?}"),
    }
}

#[test]
fn unsigned_invariant_rejection_publishes_neither_business_write_nor_settlement() {
    const REJECTED_RETENTION: u64 = 21;
    let (application, instance, required) = journey(REJECTED_RETENTION);
    reset_candidate_count(REJECTED_RETENTION);
    assert!(matches!(
        execute(&application, &instance, &required, REJECTED_RETENTION, 91030).unwrap(),
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(denial))
            if denial.kind() == WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    ));
    assert_eq!(candidate_count(REJECTED_RETENTION), 1);
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
    assert!(matches!(
        advance_instance(&application, instance, 91031).unwrap(),
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
}

#[test]
fn legacy_approval_floor_cannot_be_removed_by_authoring_an_unsigned_definition() {
    for denial in [
        definition::<ReviewedSetRetentionBinding>().err(),
        definition::<ExplicitReviewedSetRetentionBinding>().err(),
    ] {
        assert_eq!(
            denial.unwrap().kind(),
            ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority
        );
    }
    assert!(definition::<OccurrenceSetRetentionBinding>().is_ok());
}

#[test]
fn reconstructed_occurrence_only_contract_issues_fresh_execution_authority() {
    let (application, instance, original) = journey(8);
    drop(original);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    application
        .runtime()
        .release_workflow_compilation_for_test();
    let rebuilt = match advance_instance(&application, instance.clone(), 91020).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("retained unsigned contract did not reconstruct: {other:?}"),
    };
    assert!(rebuilt
        .workflow_execution_requirements()
        .unwrap()
        .approval()
        .is_none());
    assert!(matches!(
        execute(&application, &instance, &rebuilt, 8, 91021).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
    match advance_instance(&application, instance, 91022).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("rebuilt execution did not settle: {other:?}"),
    }
}

#[test]
fn direct_invocation_and_wrong_input_cannot_bypass_occurrence_binding() {
    let (application, instance, required) = journey(8);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let input = |retention_days| SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days,
    };
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .advanced()
            .mutate(input(8))
            .without_source()
            .idempotency(&91007_u64)
            .execute_in_program(
                application.program_runtime(),
                ExecutionAllocationPolicy::SystemAllocation
            ),
        Err(WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition)
    ));
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(input(9))
            .without_source()
            .idempotency(&91008_u64)
            .for_workflow_operation(&application, &required),
        Err(WorthQueryWorkflowOperationBindingDenial::RequirementMismatch)
    ));
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
}

#[test]
fn independently_issued_competing_requirement_cannot_repeat_a_performed_occurrence() {
    let (application, instance, first) = journey(8);
    let competing = match advance_instance(&application, instance.clone(), 91009).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("competing occurrence did not resolve: {other:?}"),
    };
    assert!(matches!(
        execute(&application, &instance, &competing, 8, 91010).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    // The occurrence owns one guarded idempotency identity. A different logical
    // request cannot claim it after the winner, even with a separately issued slot.
    assert!(matches!(
        execute(&application, &instance, &first, 8, 91011).unwrap(),
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
    assert!(matches!(
        advance_instance(&application, instance, 91012).unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
}
