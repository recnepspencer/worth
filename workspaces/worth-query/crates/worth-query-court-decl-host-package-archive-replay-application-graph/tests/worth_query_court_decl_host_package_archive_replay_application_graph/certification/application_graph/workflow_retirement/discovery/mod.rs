use super::super::recorded_workflow_receipt as recorded_receipt;
pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_retirement::discovery::*;

#[test]
fn workflow_actor_contract_disabled_discovery_preserves_native_start_currentness() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    assert_eq!(
        discover(&application, main, &identity),
        Err(WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy),
    );

    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            601,
        ),
    );
    assert_eq!(
        discover(&application, main, &identity),
        Err(WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy)
    );
    let original_start = match start_instance(&application, first.clone(), 602).unwrap() {
        WorkflowInstanceStartOutcome::Started(value) => value,
        other => panic!("the original start must perform: {other:?}"),
    };
    let recorded = original_start.instance().clone();
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
        WorkflowInstanceStartOutcome::RecoveredCommitted(replay) => {
            recorded_receipt::original(&replay, original_start.receipt());
            recorded_receipt::creates(&replay, recorded.entity_id());
        }
        other => panic!("a recorded start must replay, got {other:?}"),
    }
    let started = expect_started(start_instance(&application, named, 604));
    assert_ne!(started, recorded, "the refused key recorded nothing");

    expect_retired(retire_definition(&application, second.clone(), 605));
    assert_eq!(
        discover(&application, main, &identity),
        Err(WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy),
    );
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
    assert_eq!(
        discover(&application, main, &identity),
        Err(WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy)
    );
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
    assert_eq!(
        discover(&application, fork, &identity),
        Err(WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy)
    );
    assert_eq!(
        discover(&application, main, &identity),
        Err(WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy)
    );
    expect_started(start_instance(&application, first, 613));
}
