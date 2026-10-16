//! Installed evidence floors and authored approval obligations use the same lane.

use super::super::document_retention_model::{
    retention_entry::ExplicitReviewedSetRetentionIntent,
    workflow::{
        accept_assessment, approve_instance, reviewed_document_definition_for_binding,
        settle_assessment, WorkflowGrantStatusInput, WorkflowGrantStatusIntent,
    },
};
use super::*;
use worth_query_host::facade::{
    application_entry::WorkflowApprovalDecision,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

fn approved<Binding: ApplicationMutationBinding<DocumentRetentionSchema>>() -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowInstanceRef,
    RequiredWorkflowOperation,
) {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        reviewed_document_definition_for_binding::<Binding>("done"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        92001,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("review definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, published, 92002).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("review instance did not start: {other:?}"),
    };
    let proposal =
        match propose_authoring_instance_with_retention(&application, instance.clone(), 92003, 8)
            .unwrap()
        {
            WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
            other => panic!("review proposal did not publish: {other:?}"),
        };
    for (settlement, acceptance) in [(92004, 92005), (92006, 92007)] {
        let evidence = settle_assessment(&application, instance.clone(), settlement);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &evidence, acceptance).unwrap(),
            WorkflowProgressOutcome::Completed(_)
        ));
    }
    assert!(matches!(
        advance_instance(&application, instance.clone(), 92008).unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
    let approval = match advance_instance(&application, instance.clone(), 92009).unwrap() {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("authored review was bypassed: {other:?}"),
    };
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            92010
        )
        .unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
    let required = match advance_instance(&application, instance.clone(), 92011).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("reviewed effect did not become executable: {other:?}"),
    };
    (application, instance, required)
}

#[test]
fn explicitly_required_approval_executes_with_its_exact_native_grant() {
    let (application, instance, required) = approved::<ExplicitReviewedSetRetentionBinding>();
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_some());
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ExplicitReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&92012_u64)
        .for_workflow_operation(&application, &required)
        .unwrap()
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
    match advance_instance(&application, instance, 92013).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("required approval effect did not settle: {other:?}"),
    }
}

#[test]
fn empty_installed_floor_does_not_ignore_an_authored_approval_or_its_revocation() {
    let (application, instance, required) = approved::<OccurrenceSetRetentionBinding>();
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_none());
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let changed = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .advanced()
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: "workflow-approval-grant".to_owned(),
                status: "revoked".to_owned(),
            },
        })
        .without_source()
        .idempotency(&92014_u64)
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(
        matches!(execute(&application, &instance, &required, 8, 92015),
        Err(WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(denial))
            if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable)
    );
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
}

#[test]
fn occurrence_only_floor_still_consumes_the_authored_approval() {
    // approved() first requires the real AwaitingApproval posture, signs it,
    // and obtains the resulting native operation requirement.
    let (application, instance, required) = approved::<OccurrenceSetRetentionBinding>();
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_none());
    let outcome = execute(&application, &instance, &required, 8, 92016).unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
}
