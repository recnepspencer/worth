use super::*;
use worth_query_host::facade::{
    application_contribution::WorthQueryWorkflowAssessmentPosture,
    application_entry::PublishedWorkflowInstanceRef,
    declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
};

use crate::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime;

fn failing_join(
    policy: ApplicationWorkflowEvidenceJoinPolicy,
    key: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowInstanceRef,
) {
    const FAILING_DIMENSION: u64 = 6;
    let application = publish_workflow_on_first_program();
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            FAILING_DIMENSION,
            key
        )),
        DimensionVerdict::Performed(FAILING_DIMENSION)
    );
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition_with_policy(policy),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key + 1,
    )
    .expect("join definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), key + 2)
        .expect("join instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), key + 3)
        .expect("proposal must settle");
    for (settlement_offset, acceptance_offset) in [(4, 5), (6, 7)] {
        let settled = settle_assessment(
            &application,
            started.instance().clone(),
            key + settlement_offset,
        );
        assert_eq!(
            settled.posture(),
            WorthQueryWorkflowAssessmentPosture::Failing
        );
        assert!(matches!(
            accept_assessment(
                &application,
                started.instance().clone(),
                &settled,
                key + acceptance_offset
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    (application, started.instance().clone())
}

#[test]
fn completed_failing_inventory_settles_the_declared_join_failure_branch() {
    let (application, instance) = failing_join(
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        490,
    );
    match advance_instance(&application, instance.clone(), 498)
        .expect("complete failing evidence must settle the join")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join")
        }
        other => panic!("expected a settled failing join, got {other:?}"),
    }
    match advance_instance(&application, instance, 499)
        .expect("the failed join must select its declared successor")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "rejected")
        }
        other => panic!("expected rejection terminal completion, got {other:?}"),
    }
}

#[test]
fn all_completed_policy_accepts_a_complete_failing_inventory() {
    let (application, instance) = failing_join(
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredCompleted,
        500,
    );
    assert!(matches!(
        advance_instance(&application, instance.clone(), 508),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match advance_instance(&application, instance, 509)
        .expect("the satisfied all-completed join must select completion")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "completed")
        }
        other => panic!("expected completion terminal, got {other:?}"),
    }
}
