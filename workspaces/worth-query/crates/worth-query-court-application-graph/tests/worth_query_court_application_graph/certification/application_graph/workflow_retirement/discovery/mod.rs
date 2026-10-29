pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_retirement::discovery::*;

#[test]
fn workflow_actor_contract_disabled_discovery_preserves_native_start_currentness() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    assert_eq!(discover(&application, main, &identity), Err(worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy),);

    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            601,
        ),
    );
    assert_eq!(discover(&application, main, &identity), Err(worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy));
    let recorded = expect_started(start_instance(&application, first.clone(), 602));
    let second = expect_published(
        "second revision",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
            603,
        ),
    );
    let current = second.clone();

    let named = expect_superseded_start(start_instance(&application, first.clone(), 604));
    assert_eq!(
        named, current,
        "a refused start names the actual current publication"
    );
    match start_instance(&application, first.clone(), 602)
        .expect("a recorded start's retry prepares")
    {
        WorkflowInstanceStartOutcome::Started(replay) => {
            assert!(
                replay.replayed(),
                "a superseded definition's recorded start replays"
            );
            assert_eq!(replay.instance(), &recorded);
        }
        other => panic!("a recorded start must replay, got {other:?}"),
    }
    let started = expect_started(start_instance(&application, named, 604));
    assert_ne!(started, recorded, "the refused key recorded nothing");

    expect_retired(retire_definition(&application, second.clone(), 605));
    assert_eq!(discover(&application, main, &identity), Err(worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy),);
    expect_retired_start(start_instance(&application, second, 606));
    let _reopened_definition = expect_published(
        "reopened lineage",
        publish_definition(
            &application,
            terminal_definition("reopened"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            607,
        ),
    );
    assert_eq!(discover(&application, main, &identity), Err(worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy));
}

#[test]
fn workflow_actor_contract_disabled_discovery_preserves_fork_start_control() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    let first = expect_published(
        "main revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            611,
        ),
    );
    let fork = fork_of(&application, main);
    let copied = first.held_on(fork);
    let _fork_successor = expect_published(
        "fork successor",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(copied),
            612,
        ),
    );
    assert_eq!(discover(&application, fork, &identity), Err(worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy));
    assert_eq!(discover(&application, main, &identity), Err(worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy));
    expect_started(start_instance(&application, first, 613));
}
