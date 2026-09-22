use super::*;
use worth_query_host::facade::application_contribution::WorthQueryWorkflowAssessmentPosture;

#[test]
fn changing_one_subject_invalidates_only_that_subjects_evidence() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        multi_subject_assessment_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        560,
    )
    .expect("multi-subject definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 561)
        .expect("multi-subject instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 562).expect("proposal must settle");
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            6,
            563,
        )),
        DimensionVerdict::Performed(6)
    );
    for (settlement_key, acceptance_key, subject, posture) in [
        (
            564,
            565,
            PART_IDENTITY,
            WorthQueryWorkflowAssessmentPosture::Failing,
        ),
        (
            566,
            567,
            RELATED_PART_IDENTITY,
            WorthQueryWorkflowAssessmentPosture::Passing,
        ),
    ] {
        let settled = settle_assessment_for(
            &application,
            started.instance().clone(),
            settlement_key,
            subject,
        );
        assert_eq!(settled.posture(), posture);
        assert!(matches!(
            accept_assessment(
                &application,
                started.instance().clone(),
                &settled,
                acceptance_key,
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 568),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            SEED_DIMENSION,
            569,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION)
    );

    match advance_instance(&application, started.instance().clone(), 570)
        .expect("the changed resource subject must require fresh evidence")
    {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "checks/first");
        }
        other => panic!("resource evidence was incorrectly reused: {other:?}"),
    }
    let replacement = settle_assessment(&application, started.instance().clone(), 571);
    assert_eq!(
        replacement.posture(),
        WorthQueryWorkflowAssessmentPosture::Passing
    );
    assert!(matches!(
        accept_assessment(&application, started.instance().clone(), &replacement, 572,),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match advance_instance(&application, started.instance().clone(), 573)
        .expect("the unchanged related subject must reuse its evidence")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/second");
            assert!(performed.assessment_evidence().is_none());
        }
        other => panic!("related evidence was not selectively reused: {other:?}"),
    }
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 574),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
}
