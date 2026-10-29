pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::workflow_identity::*;

#[test]
fn equal_content_publications_and_their_instances_never_alias() {
    let (application, first, early) = live_instance_on_first_program(85_500);
    let main = application.current_world();
    match retire_definition(&application, first.clone(), 85_521).expect("retirement prepares") {
        WorkflowDefinitionRetirementOutcome::Retired(_) => {}
        other => panic!("the first publication did not retire: {other:?}"),
    }
    let second = republish(&application, 85_522);
    assert_ne!(
        second.entity_id(),
        first.entity_id(),
        "an equal-content publication is its own definition",
    );
    let late = expect_started(start_instance(&application, second, 85_523));
    assert_ne!(late.entity_id(), early.entity_id());

    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            main,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );

    assert!(matches!(
        advance_instance(&application, early.clone(), 85_530),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match advance_instance(&application, early, 85_531) {
        Ok(WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Stale(_))) => {}
        other => panic!("a completed instance never advances again: {other:?}"),
    }
    assert!(
        matches!(
            advance_instance(&application, late, 85_532),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the equal-content sibling keeps its own progress",
    );
}

#[test]
fn an_advance_prepared_under_the_source_program_is_stale_after_adoption() {
    let (application, _, instance) = live_instance_on_first_program(85_600);
    let main = application.current_world();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&85_621_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("the advance prepares under P0");
    publish_adoption(
        &application,
        prepare_second_program_adoption(
            &application,
            main,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );

    match prepared.execute() {
        WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Denied(_)) => {}
        other => panic!("a P0 binding never commits after the carry: {other:?}"),
    }
    assert!(
        matches!(
            advance_instance(&application, instance, 85_622),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the carried instance progresses only through the P1 binding",
    );
}
