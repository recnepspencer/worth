use super::*;

#[test]
fn evidence_join_rejects_completed_evidence_after_its_native_source_changes() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        480,
    )
    .expect("stale-join definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 481)
        .expect("stale-join instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 482).expect("proposal must settle");
    for (settlement_key, acceptance_key) in [(483, 484), (485, 486)] {
        let settled = settle_assessment(&application, started.instance().clone(), settlement_key);
        assert!(matches!(
            accept_assessment(
                &application,
                started.instance().clone(),
                &settled,
                acceptance_key
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            SEED_DIMENSION + 1,
            487,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 488),
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt)
        )) if attempt.kind()
            == WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    ));
}

#[test]
fn assessment_acceptance_rejects_output_after_its_native_source_changes() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition("currentness"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        451,
    )
    .expect("assessment definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected a published assessment definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 452)
        .expect("assessment instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected a started assessment instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 453)
        .expect("assessment proposal preparation must succeed");
    let settled = settle_assessment(&application, started.instance().clone(), 454);

    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            SEED_DIMENSION + 1,
            455,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );

    assert!(matches!(
        accept_assessment(&application, started.instance().clone(), &settled, 456),
        Err(WorthQueryWorkflowAssessmentAcceptanceDenial::Attempt(_))
    ));

    let replacement = settle_assessment(&application, started.instance().clone(), 457);
    assert!(matches!(
        accept_assessment(&application, started.instance().clone(), &replacement, 458),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match accept_assessment(&application, started.instance().clone(), &settled, 458)
        .expect("changed assessment meaning must resolve as idempotency drift")
    {
        WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        other => panic!("expected assessment intent drift, got {other:?}"),
    }
}
