pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_approval::lineage_step_budget::*;

#[test]
fn a_migration_successor_spends_the_steps_its_source_took() {
    let (application, definition, instance) = after_steps(91_000, 2);
    let target = replace_definition(
        &application,
        definition,
        bounded_retry_definition_with_attempts(63),
        91_010,
    );
    let successor = started(migrate_instance(
        &application,
        instance,
        target.clone(),
        RESUME_AT,
        91_011,
    ))
    .instance()
    .clone();
    take_steps(&application, &successor, 91_020, BUDGET - 2);
    assert_eq!(
        step_denial(propose_authoring_instance(
            &application,
            successor.clone(),
            91_030
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "the successor's own history is short, but the lineage's budget is spent",
    );
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert_eq!(
        step_denial(propose_authoring_instance(
            &application,
            successor.clone(),
            91_031
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "a cold read spends the inherited steps too",
    );

    let next = replace_definition(
        &application,
        target,
        bounded_retry_definition_with_attempts(62),
        91_040,
    );
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            successor.clone(),
            next,
            RESUME_AT,
            91_041,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "another successor cannot restart a spent budget",
    );
    let ended = cancelled(cancel_instance(&application, successor, 91_050));
    assert!(!ended.replayed(), "a spent lineage still cancels");
}

#[test]
fn a_fork_continuation_spends_the_steps_its_copy_took() {
    let (application, definition, instance) = after_steps(91_100, 3);
    let fork = fork_of(&application, instance.branch());
    let successor = started(continue_on_fork(
        &application,
        fork,
        instance.clone(),
        definition.clone(),
        RESUME_AT,
        91_110,
    ))
    .instance()
    .clone();
    take_steps(&application, &successor, 91_120, BUDGET - 3);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert_eq!(
        step_denial(propose_authoring_instance(&application, successor, 91_130)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
    );
    take_steps(&application, &instance, 91_140, BUDGET - 3);
    assert_eq!(
        step_denial(propose_authoring_instance(
            &application,
            instance.clone(),
            91_150
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "the source on its own branch spends its own budget independently",
    );
    let later = fork_of(&application, instance.branch());
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            later,
            instance,
            definition,
            RESUME_AT,
            91_160,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "a later fork cannot restart a spent budget",
    );
}

/// The advance that spends the last step is still answered when retried:
/// its recorded outcome replays, and only a new step is refused.
#[test]
fn a_retry_of_the_step_that_spent_the_budget_replays() {
    let application = budgeted();
    let definition = match publish_definition(
        &application,
        reviewed_document_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        91_200,
    )
    .expect("the reviewed definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the reviewed definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, 91_201)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    proposal::published_proposal(&application, instance.clone(), 91_202);
    for key in [91_203, 91_205] {
        let settlement = settle_assessment(&application, instance.clone(), key);
        match accept_assessment(&application, instance.clone(), &settlement, key + 1) {
            Ok(WorkflowProgressOutcome::Completed(_)) => {}
            other => panic!("expected an accepted assessment, got {other:?}"),
        }
    }
    match advance_instance(&application, instance.clone(), 91_207) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => assert!(!performed.replayed()),
        other => panic!("the last step of the budget did not complete: {other:?}"),
    }
    for state in ["warm", "cold"] {
        match advance_instance(&application, instance.clone(), 91_207) {
            Ok(WorkflowProgressOutcome::Completed(replay)) => {
                assert!(replay.replayed(), "the {state} retry replays")
            }
            other => panic!("the {state} retry of the spending step must replay: {other:?}"),
        }
        application
            .runtime()
            .release_workflow_instance_progress_for_test();
    }
    match advance_instance(&application, instance, 91_208) {
        Ok(WorkflowProgressOutcome::PreparationDenied(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
            "a new step past the spent budget is refused",
        ),
        other => panic!("a new step past the spent budget must be refused: {other:?}"),
    }
}
