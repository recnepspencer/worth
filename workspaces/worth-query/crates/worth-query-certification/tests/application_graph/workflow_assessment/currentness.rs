use super::*;
use worth_query_host::facade::application_contribution::WorthQueryWorkflowAssessmentPosture;

#[test]
fn retried_assessments_replace_stale_failing_occurrences_at_the_join() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        490,
    )
    .expect("assessment retry definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 491)
        .expect("assessment retry instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 492).expect("proposal must settle");
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            6,
            493,
        )),
        DimensionVerdict::Performed(6)
    );
    for (settlement_key, acceptance_key) in [(494, 495), (496, 497)] {
        let settled = settle_assessment(&application, started.instance().clone(), settlement_key);
        assert_eq!(
            settled.posture(),
            WorthQueryWorkflowAssessmentPosture::Failing
        );
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
    match advance_instance(&application, started.instance().clone(), 498)
        .expect("the failing join must settle its declared retry outcome")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join");
        }
        other => panic!("expected a failed join transition, got {other:?}"),
    }

    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            7,
            499,
        )),
        DimensionVerdict::Performed(7)
    );
    for (settlement_key, acceptance_key, path) in
        [(500, 501, "checks/first"), (502, 503, "checks/second")]
    {
        let settled = settle_assessment(&application, started.instance().clone(), settlement_key);
        assert_eq!(settled.required().node_path(), path);
        assert_eq!(
            settled.posture(),
            WorthQueryWorkflowAssessmentPosture::Passing
        );
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
    match advance_instance(&application, started.instance().clone(), 504)
        .expect("the corrected latest evidence must satisfy the join")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join");
        }
        other => panic!("expected a satisfied join transition, got {other:?}"),
    }
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 505),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
}

#[test]
fn back_navigation_reuses_compatible_evidence_across_new_transition_occurrences() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        540,
    )
    .expect("evidence-reuse definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 541)
        .expect("evidence-reuse instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 542).expect("proposal must settle");
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            6,
            543,
        )),
        DimensionVerdict::Performed(6)
    );
    for (settlement_key, acceptance_key) in [(544, 545), (546, 547)] {
        let settled = settle_assessment(&application, started.instance().clone(), settlement_key);
        assert_eq!(
            settled.posture(),
            WorthQueryWorkflowAssessmentPosture::Failing
        );
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
    match advance_instance(&application, started.instance().clone(), 548)
        .expect("the failing join follows its bounded Back edge")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join");
        }
        other => panic!("expected a failed join transition, got {other:?}"),
    }

    for (key, expected_node) in [(549, "checks/first"), (550, "checks/second")] {
        match advance_instance(&application, started.instance().clone(), key)
            .expect("unchanged evidence remains compatible after Back")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), expected_node);
                assert!(
                    performed.assessment_evidence().is_none(),
                    "evidence reuse must not manufacture a second producer publication"
                );
            }
            other => panic!("expected compatible evidence reuse at {expected_node}, got {other:?}"),
        }
    }
    match advance_instance(&application, started.instance().clone(), 551)
        .expect("reused failing evidence must still settle the declared join result")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join");
        }
        other => panic!("expected a second failed join transition, got {other:?}"),
    }
    match advance_instance(&application, started.instance().clone(), 552)
        .expect("the bounded Back edge must now be exhausted")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "retry-exhausted");
        }
        other => panic!("expected retry exhaustion terminal, got {other:?}"),
    }
}

#[test]
fn evidence_join_requires_new_evidence_after_its_native_source_changes() {
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
    match advance_instance(&application, started.instance().clone(), 488)
        .expect("stale evidence must remain an unmet authored requirement")
    {
        WorkflowProgressOutcome::AwaitingEvidence(required) => {
            assert_eq!(required.required_assessments(), 2);
            assert_eq!(required.completed_assessments(), 0);
        }
        other => panic!("stale evidence incorrectly completed the join: {other:?}"),
    }
}

#[test]
fn evidence_join_requires_new_evidence_after_native_source_aba() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        520,
    )
    .expect("ABA definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 521)
        .expect("ABA instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 522).expect("proposal must settle");
    for (settlement_key, acceptance_key) in [(523, 524), (525, 526)] {
        let settled = settle_assessment(&application, started.instance().clone(), settlement_key);
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

    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            SEED_DIMENSION + 1,
            527,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            started.instance().branch(),
            SEED_DIMENSION,
            528,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION)
    );

    match advance_instance(&application, started.instance().clone(), 529)
        .expect("ABA must leave the old evidence unmet despite equal values")
    {
        WorkflowProgressOutcome::AwaitingEvidence(required) => {
            assert_eq!(required.required_assessments(), 2);
            assert_eq!(required.completed_assessments(), 0);
        }
        other => panic!("ABA evidence incorrectly completed the join: {other:?}"),
    }
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
