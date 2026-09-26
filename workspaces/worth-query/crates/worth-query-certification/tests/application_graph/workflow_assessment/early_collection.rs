use super::*;

#[test]
fn two_assessments_collect_before_the_cursor_and_replay_without_duplicate_evidence() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        early_assessment_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        600,
    )
    .expect("early assessment definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 601)
        .expect("early assessment instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    assert!(matches!(
        prepare_early_assessment_denial(
            &application,
            started.instance().clone(),
            "checks/second",
            6011,
        ),
        WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(_)
        )
    ));
    propose_instance(&application, started.instance().clone(), 602).expect("proposal must settle");
    assert!(matches!(
        prepare_early_assessment_denial(
            &application,
            started.instance().clone(),
            "checks/join",
            6021,
        ),
        WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(_)
        )
    ));

    for (path, subject, demand_key, acceptance_key) in [
        ("checks/second", PART_IDENTITY, 603, 604),
        ("checks/related", RELATED_PART_IDENTITY, 605, 606),
    ] {
        let settled = settle_early_assessment_for(
            &application,
            started.instance().clone(),
            path,
            demand_key,
            subject,
        );
        assert_eq!(settled.required().node_path(), path);
        match accept_early_assessment(
            &application,
            started.instance().clone(),
            path,
            &settled,
            acceptance_key,
        )
        .expect("early collection must publish evidence")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), path);
                assert!(performed.assessment_evidence().is_some());
            }
            other => panic!("expected performed early collection, got {other:?}"),
        }
    }

    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let before = application.runtime().workflow_instance_progress_counters();
    match advance_instance(&application, started.instance().clone(), 607)
        .expect("cold reconstruction must preserve the unreviewed first requirement")
    {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "checks/first");
            assert_eq!(required.occurrence(), 3);
        }
        other => panic!("early evidence moved the cursor or supplied missing coverage: {other:?}"),
    }
    let after = application.runtime().workflow_instance_progress_counters();
    assert_eq!(after.cold_misses(), before.cold_misses() + 1);

    let first = settle_assessment(&application, started.instance().clone(), 608);
    assert!(matches!(
        accept_assessment(&application, started.instance().clone(), &first, 609),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    {
        use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
        let runtime = application.runtime();
        let scope = super::super::bounded_dimension_model::operator_identity::request_scope();
        let principal =
            super::super::bounded_dimension_model::operator_identity::authenticate_operator(
                runtime.installed_schema(),
                &scope,
            );
        let back = runtime
            .request(&principal, &scope)
            .mutate(
                super::super::bounded_dimension_model::workflow::WorkflowAdvanceIntent {
                    input: super::super::bounded_dimension_model::workflow::WorkflowAdvanceInput {
                        part_identity: PART_IDENTITY.to_owned(),
                    },
                },
            )
            .without_source()
            .idempotency(&6091)
            .prepare_workflow_navigate_back(&application, started.instance().clone())
            .expect("Back must receive fresh admission")
            .execute();
        assert!(back.is_ok());
    }
    match advance_instance(&application, started.instance().clone(), 6092)
        .expect("Back must preserve the first reviewed subject")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/first");
            assert!(performed.assessment_evidence().is_none());
        }
        other => panic!("expected first review reuse after Back, got {other:?}"),
    }
    for (path, key) in [("checks/second", 610), ("checks/related", 611)] {
        match advance_instance(&application, started.instance().clone(), key)
            .expect("compatible early evidence must be consumed")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), path);
                assert!(performed.assessment_evidence().is_none());
            }
            other => panic!("expected evidence reuse at {path}, got {other:?}"),
        }
    }
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 612),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
}
