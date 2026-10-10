//! A selected branch names the definition it holds current under a workflow
//! identity. A discovered reference grants nothing: a start that names a
//! definition its branch has since superseded or retired is refused with a
//! typed outcome before any effect, and a retry of a recorded start still
//! replays.

use worth_query_host::facade::{
    application_discovery::{
        WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowDefinitionDiscoveryDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowSpec,
        ApplicationWorkflowSpecIdentity,
    },
    product::WorthQueryProductBranch,
};

use super::super::document_retention_model::schema::DocumentRetentionSchema;
use super::fork_definitions::fork_of;
use super::*;

fn discover(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> WorthQueryWorkflowDefinitionDiscovery {
    application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .on_branch(branch)
                .select(&host_phase)
                .expect("the branch selects its exact occurrence")
        })
        .expect("installed host call opens")
        .discover_workflow_definition::<ReviewedDocumentWorkflow>(request_execution, identity)
        .expect("the branch's workflow lineage reads within its bound")
}

fn discovered_current(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> PublishedWorkflowDefinitionRef {
    match discover(request_execution, application, branch, identity) {
        WorthQueryWorkflowDefinitionDiscovery::Current(current) => current,
        other => panic!("the branch must hold a current definition, got {other:?}"),
    }
}

#[test]
fn discovery_names_the_current_definition_and_starts_refuse_any_other() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        WorthQueryWorkflowDefinitionDiscovery::Unpublished,
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
        discovered_current(request_execution, &application, main, &identity),
        first
    );
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
    let current = discovered_current(request_execution, &application, main, &identity);
    assert_eq!(current, second);

    let named = expect_superseded_start(start_instance(&application, first.clone(), 604));
    assert_eq!(named, current, "a refused start names what discovery named");
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
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        WorthQueryWorkflowDefinitionDiscovery::Retired,
    );
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
    assert_eq!(
        discovered_current(request_execution, &application, main, &identity),
        reopened
    );
}

#[test]
fn discovery_on_a_fork_names_the_forks_own_definition() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
    let copied = discovered_current(request_execution, &application, fork, &identity);
    assert_eq!(copied, first.held_on(fork));
    let successor = expect_published(
        "fork successor",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(copied),
            612,
        ),
    );
    assert_eq!(
        discovered_current(request_execution, &application, fork, &identity),
        successor
    );
    assert_eq!(
        discovered_current(request_execution, &application, main, &identity),
        first
    );
    expect_started(start_instance(&application, first, 613));
}

/// A second workflow family over the same schema, which never published the
/// lineage the reviewed-document spec owns.
struct ForeignWorkflow;

impl ApplicationWorkflowSpec for ForeignWorkflow {
    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.certification.foreign-workflow.v1");
}

#[test]
fn discovery_under_another_spec_names_the_lineage_foreign() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    let first = expect_published(
        "owned revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            621,
        ),
    );
    let foreign = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .on_branch(main)
                .select(&host_phase)
                .expect("the branch selects its exact occurrence")
        })
        .expect("installed host call opens")
        .discover_workflow_definition::<ForeignWorkflow>(request_execution, &identity);
    assert_eq!(
        foreign,
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ForeignLineage),
        "a lineage another spec published is named foreign, not unreadable",
    );
    assert_eq!(
        discovered_current(request_execution, &application, main, &identity),
        first
    );
}
