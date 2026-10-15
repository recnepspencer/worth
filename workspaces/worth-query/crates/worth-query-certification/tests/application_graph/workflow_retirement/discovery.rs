//! A selected branch names the definition it holds current under a workflow
//! identity. A discovered reference grants nothing: a start that names a
//! definition its branch has since superseded or retired is refused with a
//! typed outcome before any effect, and a retry of a recorded start still
//! replays.

use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowDefinitionDiscovery,
    declaration::application_program::ApplicationWorkflowDefinitionIdentity,
    product::WorthQueryProductBranch,
};

use super::fork_definitions::fork_of;
use super::*;

fn discover(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> Result<WorthQueryWorkflowDefinitionDiscovery, ()> {
    super::super::document_retention_model::workflow::discover_definition_metadata(
        application,
        branch,
        identity,
    )
}

#[test]
fn workflow_actor_contract_disabled_discovery_preserves_native_start_currentness() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    assert_eq!(discover(&application, main, &identity), Err(()),);

    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            601,
        ),
    );
    assert_eq!(discover(&application, main, &identity), Err(()));
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
    assert_eq!(discover(&application, main, &identity), Err(()),);
    expect_retired_start(start_instance(&application, second, 606));
    let reopened = expect_published(
        "reopened lineage",
        publish_definition(
            &application,
            terminal_definition("reopened"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            607,
        ),
    );
    assert_eq!(discover(&application, main, &identity), Err(()));
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
    let successor = expect_published(
        "fork successor",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(copied),
            612,
        ),
    );
    assert_eq!(discover(&application, fork, &identity), Err(()));
    assert_eq!(discover(&application, main, &identity), Err(()));
    expect_started(start_instance(&application, first, 613));
}
